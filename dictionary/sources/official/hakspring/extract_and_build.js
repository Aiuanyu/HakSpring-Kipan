#!/usr/bin/env node
/**
 * HakSpring Dictionary Pipeline for HakSpring-Kipan (客話企盤)
 * 
 * 1. 讀取客源翠四縣認證（初基中中高中）與教典四縣語料
 *    plus the HakSpring character supplement (data/supp/, step 3)
 * 2. 調用客源翠 js/variant-parser.js (getLineVariants) 展開所有區域音字變體
 * 3. 展開同義詞括號，剔除所有【】污染，確保（諸夏字, 讀音）一對一精確對應
 * 4. 進行零聲母簡拼與無調轉換
 * 5. 輸出 hakspring.csv、dictionary.bin (TKDB v4)、dictionary.fst、syllables.fst
 */

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { walkerCost } = require('./tkdb_v4');

const REPO_ROOT = path.resolve(__dirname, '../../../../');
// HAKSPRING_ROOT overrides the HakSpring checkout, e.g. a worktree whose data is not on main yet.
const HAKSPRING_ROOT = process.env.HAKSPRING_ROOT || '/Users/gjrobert/NAS/projects/HakSpring';
const HAKSPRING_DATA = path.join(HAKSPRING_ROOT, 'data');
const FST_BUILDER = path.join(REPO_ROOT, 'engine/target/debug/fst-builder');

// 載入客源翠正宗 variant-parser
const { getLineVariants } = require(path.join(HAKSPRING_ROOT, 'js/variant-parser.js'));

// 聲調與母音映射表
const VOWEL_MAP = {
  'ā': 'a', 'á': 'a', 'ǎ': 'a', 'â': 'a', 'à': 'a',
  'ē': 'e', 'é': 'e', 'ě': 'e', 'ê': 'e', 'è': 'e',
  'ī': 'i', 'í': 'i', 'ǐ': 'i', 'î': 'i', 'ì': 'i',
  'ō': 'o', 'ó': 'o', 'ǒ': 'o', 'ô': 'o', 'ò': 'o',
  'ū': 'u', 'ú': 'u', 'ǔ': 'u', 'û': 'u', 'ù': 'u',
  'n̄': 'n', 'ń': 'n', 'ň': 'n', 'n̂': 'n', 'ǹ': 'n',
  'm̄': 'm', 'ḿ': 'm', 'm̌': 'm', 'm̂': 'm', 'm̀': 'm'
};

function removeTone(syll) {
  let s = syll.toLowerCase();
  let res = '';
  for (const ch of s) {
    res += VOWEL_MAP[ch] || ch;
  }
  return res.replace(/[^a-z]/g, '');
}

// 四縣調值轉 HagPinPlus 調符
const TONE_VALUE_MAP = {
  '24': { a: 'á', e: 'é', i: 'í', o: 'ó', u: 'ú' },
  '11': { a: 'ǎ', e: 'ě', i: 'ǐ', o: 'ǒ', u: 'ǔ' },
  '31': { a: 'à', e: 'è', i: 'ì', o: 'ò', u: 'ù' },
  '55': { a: 'a', e: 'e', i: 'i', o: 'o', u: 'u' },
  '2':  { a: 'à', e: 'è', i: 'ì', o: 'ò', u: 'ù' },
  '5':  { a: 'a', e: 'e', i: 'i', o: 'o', u: 'u' },
};
const VOWEL_ORDER = ['a', 'e', 'o', 'u', 'i'];

function convertToneValuesToDiacritics(str) {
  return str.replace(/([a-zA-Z]+)(24|11|31|55|[25])(?=[a-zA-Z]|\b)/g, (match, syll, tone) => {
    const toneMap = TONE_VALUE_MAP[tone];
    if (!toneMap) return syll;
    let converted = syll;
    for (const v of VOWEL_ORDER) {
      if (converted.includes(v)) {
        converted = converted.replace(v, toneMap[v]);
        break;
      }
    }
    return converted + ' ';
  }).replace(/\s+/g, ' ').trim();
}

/**
 * 清理並切分包含中文讀音註記的標音字串
 * 例如：ha sù 又讀2.há sù -> ['ha sù', 'há sù']
 * 例如：文1.qiěn sii 白qiěn se -> ['qiěn sii', 'qiěn se']
 */
function splitReadingVariants(phonetic) {
  if (!phonetic) return [];
  let cleaned = convertToneValuesToDiacritics(phonetic);
  const parts = cleaned.split(/(?:又讀|又俗音|俗音|又音|又|文|白)\d*\.?/g)
    .map(s => s.trim())
    .filter(Boolean);
  return parts.length > 0 ? parts : [cleaned];
}

/**
 * 展開同義詞括號（例如 屁（屁卵） / pi (pi lòn)）
 */
function expandParens(word, phonetic) {
  const res = [];
  const mWord = word.match(/^(.*?)[（\(](.*?)[）\)](.*?)$/);
  const mPhon = phonetic.match(/^(.*?)[（\(](.*?)[）\)](.*?)$/);

  if (mWord && mPhon) {
    const w1 = (mWord[1] + mWord[3]).trim();
    const w2 = mWord[2].trim();
    const p1 = (mPhon[1] + mPhon[3]).trim();
    const p2 = mPhon[2].trim();
    if (w1 && p1) res.push({ word: w1, phonetic: p1 });
    if (w2 && p2) res.push({ word: w2, phonetic: p2 });
  } else if (mWord && !mPhon) {
    const w1 = (mWord[1] + mWord[3]).trim();
    const w2 = mWord[2].trim();
    if (w1 && phonetic) res.push({ word: w1, phonetic });
    if (w2 && phonetic) res.push({ word: w2, phonetic });
  } else {
    const cleanW = word.replace(/[（\(\)）]/g, '').trim();
    const cleanP = phonetic.replace(/[（\(\)）]/g, '').trim();
    if (cleanW && cleanP) res.push({ word: cleanW, phonetic: cleanP });
  }
  return res;
}

function processWordPhoneticPairs(word, rawPhonetic) {
  const phonetics = splitReadingVariants(rawPhonetic);
  const pairs = [];
  for (const ph of phonetics) {
    pairs.push(...expandParens(word, ph));
  }
  return pairs;
}

/**
 * 客話 HagPinPlus 聲母提取（長度優先，涵蓋多字母聲母）
 * 包含 zh, ch, sh, rh, ng, bb 等複合聲母
 * 與 engine/phonetics/src/hakfa_tables.rs::HAKFA_ABBREV_INITIALS 保持 100% 同步
 */
const HAKFA_ABBREV_INITIALS = [
  'zh', 'ch', 'sh', 'rh', 'bb', 'ng',
  'b', 'p', 'm', 'f', 'v', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'z', 'c', 's', 'j', 'q', 'x'
];

function extractLeadingInitial(syllable) {
  const lowered = syllable.toLowerCase();
  for (const init of HAKFA_ABBREV_INITIALS) {
    if (lowered.startsWith(init)) {
      return init;
    }
  }
  // 零聲母（元音開頭），取第 1 個字元
  return lowered[0] || '';
}

function deriveHakfaAbbrev(notoneSylls) {
  if (notoneSylls.length < 2) {
    return '';
  }
  return notoneSylls.map(extractLeadingInitial).join('');
}

console.log('▶ [1/4] 開始萃取客源翠語料並展開變體...');

const map = new Map(); // key: word + '|' + fusedNotone -> entry

// 1. 處理認證五級（按級別賦予詞頻）
const certFiles = [
  { file: '113四基.json', freq: 50000, name: '四縣基礎級' },
  { file: '113四初.json', freq: 45000, name: '四縣初級' },
  { file: '113四中.json', freq: 40000, name: '四縣中級' },
  { file: '113四中高.json', freq: 35000, name: '四縣中高級' },
  { file: '113四高.json', freq: 30000, name: '四縣高級' }
];

for (const cf of certFiles) {
  const p = path.join(HAKSPRING_DATA, 'cert', cf.file);
  if (!fs.existsSync(p)) continue;
  const raw = fs.readFileSync(p, 'utf8');
  const data = JSON.parse(raw);
  const lines = data.content.split('\n');
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;
    const parts = line.split(',');
    if (parts.length < 4) continue;
    const row = {
      客家語: parts[1],
      客語標音_顯示: parts[2],
      客語標音_查詢: parts[3],
      sourceName: cf.name
    };

    // 透過客源翠 variant-parser 展開變體
    const parsed = getLineVariants(row);
    const pairs = [];
    if (parsed.main && parsed.main.word && parsed.main.phonetic) {
      pairs.push(...processWordPhoneticPairs(parsed.main.word, parsed.main.phonetic));
    }
    for (const v of parsed.variants) {
      const w = v.word || (parsed.main ? parsed.main.word : '');
      if (w && v.phonetic) {
        pairs.push(...processWordPhoneticPairs(w, v.phonetic));
      }
    }

    for (const pair of pairs) {
      const tokens = pair.phonetic.trim().split(/[\s\-]+/);
      const notoneSylls = tokens.map(removeTone).filter(Boolean);
      if (notoneSylls.length === 0) continue;
      const fusedNotone = notoneSylls.join('');
      const hyphenatedPhonetic = tokens.join('-');
      // 客話聲母縮寫（多字母聲母長度優先，單音節詞留空）
      const fusedAbbrev = deriveHakfaAbbrev(notoneSylls);
      const key = pair.word + '|' + fusedNotone;

      if (!map.has(key) || map.get(key).freq < cf.freq) {
        map.set(key, {
          word: pair.word,
          tl: hyphenatedPhonetic,
          tl_num: hyphenatedPhonetic,
          tl_notone: fusedNotone,
          tl_abbrev: fusedAbbrev,
          freq: cf.freq,
          syllable_count: Math.min(notoneSylls.length, 4),
          syllables: notoneSylls
        });
      }
    }
  }
}

console.log(`  ✓ 認證詞彙（展開變體後）: ${map.size} 筆`);

// 2. 處理教典四縣（讀取 20250630-四.json，取得正宗 HagPinPlus 顯示欄位）
const gipJsonPath = path.join(HAKSPRING_DATA, 'gip', '20250630-四.json');
if (fs.existsSync(gipJsonPath)) {
  const rawGip = fs.readFileSync(gipJsonPath, 'utf8');
  const gipData = JSON.parse(rawGip);
  const lines = gipData.content.split('\n');
  const gipDefaultFreq = 20000;

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;
    const parts = line.split(',');
    if (parts.length < 4) continue;
    const row = {
      客家語: parts[1],
      客語標音_顯示: parts[2],
      客語標音_查詢: parts[3],
      sourceName: '教典四縣'
    };

    const parsed = getLineVariants(row);
    const pairs = [];
    if (parsed.main && parsed.main.word && parsed.main.phonetic) {
      pairs.push(...processWordPhoneticPairs(parsed.main.word, parsed.main.phonetic));
    }
    for (const v of parsed.variants) {
      const w = v.word || (parsed.main ? parsed.main.word : '');
      if (w && v.phonetic) {
        pairs.push(...processWordPhoneticPairs(w, v.phonetic));
      }
    }

    for (const pair of pairs) {
      const tokens = pair.phonetic.trim().split(/[\s\-]+/);
      const notoneSylls = tokens.map(removeTone).filter(Boolean);
      if (notoneSylls.length === 0) continue;
      const fusedNotone = notoneSylls.join('');
      const hyphenatedPhonetic = tokens.join('-');
      // 客話聲母縮寫（多字母聲母長度優先，單音節詞留空）
      const fusedAbbrev = deriveHakfaAbbrev(notoneSylls);
      const key = pair.word + '|' + fusedNotone;
      const freq = Math.max(100, gipDefaultFreq - pair.word.length * 1000);

      if (!map.has(key)) {
        map.set(key, {
          word: pair.word,
          tl: hyphenatedPhonetic,
          tl_num: hyphenatedPhonetic,
          tl_notone: fusedNotone,
          tl_abbrev: fusedAbbrev,
          freq: freq,
          syllable_count: Math.min(notoneSylls.length, 4),
          syllables: notoneSylls
        });
      }
    }
  }
}

// 3. HakSpring character supplement (Sixian): single-character readings for characters
//    that MOE and certification data only have inside words, aligned by HakSpring.
//    Its own source: bit 6 (upstream's `kungge`, relabelled and locked on).
//    Produced by HakSpring `python3 process_all_data.py --only-supp`; see docs/hakfa/supp-dict.md.
const SUPP_BITMASK = 1 << 6;
const suppJsonPath = path.join(HAKSPRING_DATA, 'supp', '補字典-四.json');
if (!fs.existsSync(suppJsonPath)) {
  throw new Error(`${suppJsonPath} not found: run python3 process_all_data.py --only-supp in HakSpring first`);
}
{
  const suppLines = JSON.parse(fs.readFileSync(suppJsonPath, 'utf8')).content.split('\n');
  let added = 0;
  for (let i = 1; i < suppLines.length; i++) {
    const line = suppLines[i];
    if (!line.trim()) continue;
    // The first four columns (id, word, display and query readings) never hold commas or quotes.
    const parts = line.split(',');
    const word = parts[1];
    const syll = parts[2];
    if (!word || !syll) continue;
    const notone = removeTone(syll);
    const key = word + '|' + notone;
    if (map.has(key)) continue;
    // Frequency grows with how often the character occurs in words, but stays below every real entry.
    const count = Number((line.match(/出現 (\d+) 擺/) || [])[1] || 1);
    map.set(key, {
      word,
      tl: syll,
      tl_num: syll,
      tl_notone: notone,
      tl_abbrev: '',
      freq: Math.min(9000, 1000 + 100 * count),
      syllable_count: 1,
      syllables: [notone],
      bitmask: SUPP_BITMASK,
      subtag: 0, // not a kautian row (engine/lexicon dictionary_reader.rs passes_filter)
    });
    added++;
  }
  console.log(`  ✓ 客源翠補字典（四縣）: ${added} 筆`);
}

const records = Array.from(map.values());
// 斷言：嚴格確保所有詞條的標音 tl 都不包含任何數字
for (const r of records) {
  if (/[0-9]/.test(r.tl)) {
    throw new Error(`[Assertion Error] 詞條 "${r.word}" 拼音仍然包含數字: "${r.tl}"`);
  }
}
// 依照詞頻排序
records.sort((a, b) => b.freq - a.freq);
console.log(`  ✓ 總計詞庫筆數 (認證 + 教典): ${records.length} 筆`);

// 輸出 CSV
const csvOutputDir = path.join(REPO_ROOT, 'dictionary/sources/official/hakspring/data');
fs.mkdirSync(csvOutputDir, { recursive: true });
const csvPath = path.join(csvOutputDir, 'hakspring.csv');
const csvLines = ['word,tl,tl_num,tl_notone,tl_abbrev,frequency,syllable_count,bitmask'];
for (const r of records) {
  csvLines.push(`"${r.word}","${r.tl}","${r.tl_num}","${r.tl_notone}","${r.tl_abbrev}",${r.freq},${r.syllable_count},${r.bitmask ?? 1}`);
}
fs.writeFileSync(csvPath, csvLines.join('\n'), 'utf8');
console.log(`  ✓ 已輸出 CSV 至 ${csvPath}`);

// -------------------------------------------------------------
// [2/4] 建置 dictionary.bin (TKDB v4)
// -------------------------------------------------------------
console.log('\n▶ [2/4] 建置 dictionary.bin (TKDB v4)...');
const outDirs = [
  path.join(REPO_ROOT, 'output/hakfa'),
  path.join(REPO_ROOT, 'dictionary/output/hakfa'),
  path.join(REPO_ROOT, 'assets/dictionaries')
];
for (const d of outDirs) {
  fs.mkdirSync(d, { recursive: true });
}

const encodedRecords = [];
const allSyllables = new Set();
const fstLines = [];

for (let i = 0; i < records.length; i++) {
  const r = records[i];
  const rowid = i + 1; // 1-based

  // 蒐集音節到 inventory
  for (const s of r.syllables) {
    allSyllables.add(s);
  }

  // FST keys
  if (r.tl_notone) {
    fstLines.push(`TL\ttl:${r.tl_notone}\t${rowid}\n`);
  }
  if (r.tl_num && r.tl_num !== r.tl_notone) {
    // 移除空白與連字符後的連寫
    const numClean = r.tl_num.replace(/[\s\-]+/g, '').toLowerCase();
    fstLines.push(`TL\ttl:${numClean}\t${rowid}\n`);
  }
  if (r.tl_abbrev) {
    fstLines.push(`TL\ttl-abbrev:${r.tl_abbrev}\t${rowid}\n`);
  }
  if (r.word) {
    fstLines.push(`TL\thanzi:${r.word}\t${rowid}\n`);
  }

  // 二進位打包
  const hanziBuf = Buffer.from(r.word, 'utf8');
  const tlBuf = Buffer.from(r.tl, 'utf8');
  const recBuf = Buffer.alloc(13 + hanziBuf.length + tlBuf.length);

  recBuf.writeUInt16LE(r.bitmask ?? 1, 0); // bitmask: 1 (MOE + certification); supplement 1 << 6
  recBuf.writeUInt32LE(r.freq, 2);          // frequency
  recBuf.writeUInt8(hanziBuf.length, 6);    // hanzi_len
  recBuf.writeUInt8(tlBuf.length, 7);       // tl_len
  recBuf.writeUInt8(r.syllable_count, 8);   // syllable_count
  recBuf.writeUInt16LE(r.subtag ?? 1, 9);  // kautian_subtag: 1 (KAUTIAN_SUBTAG_MAIN_BIT: has_main = true); supplement 0
  recBuf.writeUInt16LE(walkerCost(r.freq), 11); // walker_cost (v4), see tkdb_v4.js
  hanziBuf.copy(recBuf, 13);
  tlBuf.copy(recBuf, 13 + hanziBuf.length);

  encodedRecords.push(recBuf);
}

// 寫入二進位檔案
const count = encodedRecords.length;
const headerSize = 16;
const offsetTableSize = count * 4;
let currentOffset = headerSize + offsetTableSize;
const offsets = [];
for (const rec of encodedRecords) {
  offsets.push(currentOffset);
  currentOffset += rec.length;
}

const binBuf = Buffer.alloc(currentOffset);
binBuf.write('TKDB', 0, 4, 'ascii');
binBuf.writeUInt32LE(4, 4);         // version 4
binBuf.writeUInt32LE(count, 8);     // count
binBuf.writeUInt32LE(20261007, 12); // build_ts

for (let i = 0; i < count; i++) {
  binBuf.writeUInt32LE(offsets[i], 16 + i * 4);
}

let writePos = headerSize + offsetTableSize;
for (const rec of encodedRecords) {
  rec.copy(binBuf, writePos);
  writePos += rec.length;
}

for (const d of outDirs) {
  const p = path.join(d, 'dictionary.bin');
  fs.writeFileSync(p, binBuf);
  console.log(`  ✓ 成功產出 ${p} (${count} 筆, ${(binBuf.length / 1024 / 1024).toFixed(2)} MB)`);
}

// -------------------------------------------------------------
// [3/4] 建置 dictionary.fst 前綴自動機
// -------------------------------------------------------------
console.log('\n▶ [3/4] 建置 dictionary.fst 前綴自動機...');
if (!fs.existsSync(FST_BUILDER)) {
  console.error(`錯誤: 找不到 fst-builder (${FST_BUILDER})，請先執行 cargo build -p fst-builder`);
  process.exit(1);
}

const fstPayload = fstLines.join('');
for (const d of outDirs) {
  const p = path.join(d, 'dictionary.fst');
  const res = spawnSync(FST_BUILDER, ['build', p], {
    input: fstPayload,
    encoding: 'utf8'
  });
  if (res.status !== 0) {
    console.error(`fst-builder 失敗: ${res.stderr}`);
    process.exit(1);
  }
  const stat = fs.statSync(p);
  console.log(`  ✓ 成功產出 ${p} (${fstLines.length} 條鍵, ${(stat.size / 1024 / 1024).toFixed(2)} MB)`);
}

// -------------------------------------------------------------
// [4/4] 建置 syllables.fst 合法客話音節庫
// -------------------------------------------------------------
console.log('\n▶ [4/4] 建置 syllables.fst 音節庫 (fst::Set)...');
const syllableKeys = Array.from(allSyllables)
  .sort()
  .map(s => `tl:${s}`);

for (const d of outDirs) {
  const p = path.join(d, 'syllables.fst');
  const res = spawnSync(FST_BUILDER, ['build-set', p], {
    input: syllableKeys.join('\n') + '\n',
    encoding: 'utf8'
  });
  if (res.status !== 0) {
    console.error(`fst-builder build-set 失敗: ${res.stderr}`);
    process.exit(1);
  }
  const stat = fs.statSync(p);
  console.log(`  ✓ 成功產出 ${p} (${syllableKeys.length} 個音節, ${(stat.size / 1024).toFixed(1)} KB)`);
}

console.log('\n🎉 客話詞庫管線全數建置完成！');
