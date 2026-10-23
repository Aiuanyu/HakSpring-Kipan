#!/usr/bin/env node
/**
 * TKDB v4 for Kipan: the `walker_cost` field upstream added in v4, priced from
 * Kipan's own frequency.
 *
 * Upstream prices `walker_cost` from a Taigi corpus unigram model
 * (`dictionary/build/walker_lm.py`). Kipan has no Hakfa corpus yet, so it keeps
 * the cost its walker used before v4 (`engine/composing/src/lattice/cost.rs`
 * up to v3.6.13): `ln(CORPUS_TOTAL_FREQ) − ln(1 + frequency)`, in milli-nats.
 * Replace this with a Hakfa corpus model once one exists.
 *
 *   node tkdb_v4.js <dictionary.bin>   upgrade a v3 file to v4 in place
 */

const fs = require('fs');
const path = require('path');

const UNIGRAMS_PATH = path.join(__dirname, 'data/hakfa_unigrams.json');
let hakfaUnigrams = null;
let totalTokens = 255177;
let vocabSize = 21653;

if (fs.existsSync(UNIGRAMS_PATH)) {
  try {
    const raw = JSON.parse(fs.readFileSync(UNIGRAMS_PATH, 'utf8'));
    hakfaUnigrams = raw.unigrams || {};
    if (raw.metadata) {
      totalTokens = raw.metadata.total_tokens || totalTokens;
      vocabSize = raw.metadata.vocabulary_size || vocabSize;
    }
  } catch (e) {
    console.warn('載入 hakfa_unigrams.json 失敗，回退至 baseline：', e);
  }
}

const CORPUS_TOTAL_FREQ = 13056588;
const MILLI_NATS_PER_NAT = 1000;
const V3_FIXED_PREFIX = 11; // bitmask u16, frequency u32, hanzi_len u8, tl_len u8, syllable_count u8, kautian_subtag u16
const HEADER_SIZE = 16;

/**
 * 依據客話 Unigram 語言模型（以 Lidstone add-alpha Informed Prior 平滑）計算 walker_cost
 * 若無語料則回退至原頻率公式。
 */
function walkerCost(word, frequency) {
  if (hakfaUnigrams) {
    let prior = 0.02;
    if (frequency >= 50000) prior = 1.0;
    else if (frequency >= 45000) prior = 0.6;
    else if (frequency >= 40000) prior = 0.3;
    else if (frequency >= 35000) prior = 0.15;
    else if (frequency >= 30000) prior = 0.05;
    else if (frequency < 10000) prior = 0.001; // 補字典

    const count = (hakfaUnigrams[word] || 0) + prior;
    const prob = count / (totalTokens + 0.5 * vocabSize);
    const cost = Math.round(-Math.log(prob) * MILLI_NATS_PER_NAT);
    return Math.max(0, Math.min(0xfffe, cost));
  }

  // Fallback baseline
  const cost = Math.round((Math.log(CORPUS_TOTAL_FREQ) - Math.log(1 + frequency)) * MILLI_NATS_PER_NAT);
  if (!(cost >= 0 && cost < 0xffff)) {
    throw new Error(`walker_cost ${cost} for frequency ${frequency} is outside u16 (0xffff is "unpriced")`);
  }
  return cost;
}

/**
 * 依據客話語料真實出現次數，提供候選詞頻率加成 (LM Frequency Boost)
 */
function boostFrequency(word, baseFreq) {
  if (!hakfaUnigrams || !hakfaUnigrams[word]) return baseFreq;
  const count = hakfaUnigrams[word];
  const boost = Math.min(8000, Math.round(Math.log2(1 + count) * 550));
  return baseFreq + boost;
}

/** v3 record → v4 record: insert walker_cost u16 after kautian_subtag. */
function recordV4(v3) {
  const frequency = v3.readUInt32LE(2);
  const out = Buffer.alloc(v3.length + 2);
  v3.copy(out, 0, 0, V3_FIXED_PREFIX);
  out.writeUInt16LE(walkerCost(frequency), V3_FIXED_PREFIX);
  v3.copy(out, V3_FIXED_PREFIX + 2, V3_FIXED_PREFIX);
  return out;
}

function upgradeFile(path) {
  const bin = fs.readFileSync(path);
  if (bin.toString('ascii', 0, 4) !== 'TKDB') throw new Error(`${path}: not a TKDB file`);
  const version = bin.readUInt32LE(4);
  if (version !== 3) throw new Error(`${path}: version ${version}, expected 3`);
  const count = bin.readUInt32LE(8);
  const offsets = [];
  for (let i = 0; i < count; i++) offsets.push(bin.readUInt32LE(HEADER_SIZE + i * 4));
  offsets.push(bin.length);
  const records = [];
  for (let i = 0; i < count; i++) records.push(recordV4(bin.subarray(offsets[i], offsets[i + 1])));
  const header = Buffer.from(bin.subarray(0, HEADER_SIZE));
  header.writeUInt32LE(4, 4);
  const table = Buffer.alloc(count * 4);
  let at = HEADER_SIZE + table.length;
  records.forEach((r, i) => {
    table.writeUInt32LE(at, i * 4);
    at += r.length;
  });
  fs.writeFileSync(path, Buffer.concat([header, table, ...records]));
  console.log(`  ✓ ${path}: TKDB v3 → v4 (${count} records)`);
}

module.exports = { walkerCost, boostFrequency, V3_FIXED_PREFIX };

if (require.main === module) {
  if (process.argv.length !== 3) {
    console.error('usage: node tkdb_v4.js <dictionary.bin>');
    process.exit(2);
  }
  upgradeFile(process.argv[2]);
}
