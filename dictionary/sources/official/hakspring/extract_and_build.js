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
const { walkerCost, boostFrequency } = require('./tkdb_v4');

const REPO_ROOT = path.resolve(__dirname, '../../../../');
// HAKSPRING_ROOT overrides the HakSpring checkout, e.g. a worktree whose data is not on main yet.
const HAKSPRING_ROOT = process.env.HAKSPRING_ROOT || path.resolve(REPO_ROOT, '../HakSpring');
const HAKSPRING_DATA = path.join(HAKSPRING_ROOT, 'data');
const FST_BUILDER = path.join(REPO_ROOT, 'engine/target/debug/fst-builder');

// 載入客源翠正宗 variant-parser
const { getLineVariants } = require(path.join(HAKSPRING_ROOT, 'js/variant-parser.js'));

// Dictionary sources (bitmask). Obsidian `dialects` plan: 教典 0, 認證 1, … 客源翠補字典 6.
// Upstream internal names: bit 0 `kautian`, bit 1 `taigitv`, bit 6 `kungge` (soft fork).
const KAUTIAN_BITMASK = 1 << 0;
const CERT_BITMASK = 1 << 1;
const SUPP_BITMASK = 1 << 6;

// Lects (腔). The lect code rides kautian_subtag bits 13-15, which the engine reader
// masks off today (KAUTIAN_SUBTAG_USED_MASK in engine/lexicon/src/dictionary_reader.rs),
// so writing it changes no filtering yet. Code 0 means "no lect" and fails the build.
// 四 ships today; 海 builds to its own output files (see OUT_SUFFIX) until the engine reads the lect code.
const LECT_SHIFT = 13;
const LECTS = {
  '四': { code: 1, name: '四縣', id: 'xiien' },
  '海': { code: 2, name: '海陸', id: 'hoiliug' },
  // 南四縣: its own lect in the dictionary (教典南 + 補字典南, no certification list), so the
  // settings can list it as a lect of its own (Obs IME data-ui R). Code 6; 3-5 stay reserved for
  // 大埔, 饒平, 詔安.
  '南': { code: 6, name: '南四縣', id: 'namxiian' },
};

// （無參數）／`--lect all`  全腔合併成一份詞庫，這就係出貨詞庫：每腔各跑一次
//                 （`--dump-records` 子程序），再合併、排序、編碼，寫去出貨資料夾摎 hakfa-all。
// `--lect 四|海`  單腔，除錯用：寫去自家个 hakfa-<id>，毋蓋出貨檔。
// `--dump-records F`  內部用：萃取完个詞條寫成 JSON 就停。
const ALL_LECTS = 'all';

function parseLectArg(argv) {
  const i = argv.indexOf('--lect');
  const lectKey = i === -1 ? ALL_LECTS : argv[i + 1];
  if (lectKey !== ALL_LECTS && !LECTS[lectKey]) {
    throw new Error(`--lect ${lectKey}: unsupported lect, expected one of ${Object.keys(LECTS).join(' ')} ${ALL_LECTS}`);
  }
  return lectKey;
}
function parseOptArg(argv, flag) {
  const i = argv.indexOf(flag);
  return i === -1 ? null : argv[i + 1];
}
const LECT_KEY = parseLectArg(process.argv.slice(2));
const DUMP_RECORDS_PATH = parseOptArg(process.argv.slice(2), '--dump-records');
const IS_ALL = LECT_KEY === ALL_LECTS;
const LECT = IS_ALL ? { code: 0, name: '全腔', id: 'all' } : LECTS[LECT_KEY];
// 南四縣無自家个認證檔：用四縣个（【南】標音摎附表）
const CERT_LECT_KEY = LECT_KEY === '南' ? '四' : LECT_KEY;
const isShipLect = IS_ALL; // 全腔合併詞庫就係出貨詞庫：覆蓋 assets/dictionaries
const LECT_KEY_BY_CODE = Object.fromEntries(Object.entries(LECTS).map(([k, v]) => [v.code, k]));

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

// 調值轉 HagPinPlus 調符（照客源翠 tone_mapping.json default_tones，入聲 2／5 照韻尾分，免另外標）。
// 四縣：24 á、11 ǎ、31 à、55 無標。海陸：53 à、24 á、11 ǎ、33 ā、55 無標。
const TONE_MARKS = {
  grave: { a: 'à', e: 'è', i: 'ì', o: 'ò', u: 'ù' },
  acute: { a: 'á', e: 'é', i: 'í', o: 'ó', u: 'ú' },
  caron: { a: 'ǎ', e: 'ě', i: 'ǐ', o: 'ǒ', u: 'ǔ' },
  macron: { a: 'ā', e: 'ē', i: 'ī', o: 'ō', u: 'ū' },
  none: { a: 'a', e: 'e', i: 'i', o: 'o', u: 'u' },
};
const LECT_TONES = {
  '四': { '24': 'acute', '11': 'caron', '31': 'grave', '55': 'none', '2': 'grave', '5': 'none' },
  '海': { '53': 'grave', '24': 'acute', '11': 'caron', '33': 'macron', '55': 'none', '2': 'grave', '5': 'none' },
  // 南四縣 has the 四縣 tone values (tone_mapping.json dialect_maps only overrides 31, the same grave).
  '南': { '24': 'acute', '11': 'caron', '31': 'grave', '55': 'none', '2': 'grave', '5': 'none' },
};
const TONE_VALUE_MAP = Object.fromEntries(
  Object.entries(LECT_TONES[LECT_KEY] || {}).map(([tone, mark]) => [tone, TONE_MARKS[mark]]));
// 長調值排前面（53 先於 5），免得 5 吃掉 53 个頭。
const TONE_RE = new RegExp(
  '([a-zA-Z]+)(' + Object.keys(TONE_VALUE_MAP).sort((a, b) => b.length - a.length).join('|') + ')(?=[a-zA-Z]|\\b)', 'g');
const VOWEL_ORDER = ['a', 'e', 'o', 'u', 'i'];

function convertToneValuesToDiacritics(str) {
  return str.replace(TONE_RE, (match, syll, tone) => {
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
// 讀音欄个註記，照客源翠 scripts/build_supp_dict.py（READING_NOTES、DROP_NOTES）
// 摎 data/annotations.md 第三節。
// 另一隻正式讀音：拆開、全部收（「特」：教典特殊音，177 條，例 長 特diǒng）
const READING_NOTES = ['又俗音', '又讀', '又音', '俗音', '特殊音', '特', '又', '文', '白'];
// 合音、變調後个讀法，毋係字本身个讀音：拆開、丟忒
// 教典「特」：特殊讀音。摎主讀音淨差聲調也愛另外留一筆，毋好併
const SPECIAL_NOTES = ['特殊音', '特'];
const DROP_NOTES = ['小稱變調讀本調為', '小稱變調讀', '為後字變調', '後字變調讀', '後字變調',
  '本調為', '合音讀', '合音'];
const NOTE_SPLIT = new RegExp(
  '(' + [...READING_NOTES, ...DROP_NOTES].sort((a, b) => b.length - a.length).join('|') + ')\\s*\\d*\\s*\\.?');

function splitReadingVariants(phonetic) {
  if (!phonetic) return [];
  // 方音符號調號 ˊˇˋ 係原始資料零星混入个雜訊（起屋 `hi ˋ vùg`，查詢欄正確係 hi55 vug2），拿忒
  phonetic = phonetic.replace(/[\s\u00a0]*[ˊˇˋ]/g, '');
  // 調值數字同音節中央隔空白（海陸 `ciam 33 shi`、`ngiag 5`），接返去
  phonetic = phonetic.replace(/([a-zA-Z]) (\d{1,2})(?=\s|$)/g, '$1$2');
  const cleaned = convertToneValuesToDiacritics(phonetic);
  // split 有捕捉群組：[頭前, 註記, 讀音, 註記, 讀音, …]
  const parts = cleaned.split(new RegExp(NOTE_SPLIT.source, 'g'));
  const readings = [{ phonetic: parts[0], special: false }];
  for (let i = 1; i + 1 < parts.length; i += 2) {
    if (!DROP_NOTES.includes(parts[i])) {
      readings.push({ phonetic: parts[i + 1], special: SPECIAL_NOTES.includes(parts[i]) });
    }
  }
  // 無註記就係原讀音；有註記就淨收拆出來个（一開頭就係丟忒个讀法就無讀音）
  if (parts.length === 1) return cleaned.trim() ? [{ phonetic: cleaned.trim(), special: false }] : [];
  return readings
    .map(r => ({ phonetic: r.phonetic.trim(), special: r.special }))
    .filter(r => r.phonetic);
}

/**
 * 展開同義詞括號（例如 屁（屁卵） / pi (pi lòn)）
 */
function expandParens(word, phonetic) {
  const res = [];
  // 詞目無括號、讀音有括號：括號係讀音个另一讀（照客源翠 scripts/build_supp_dict.py expand_parens）。
  // 黏等音節个（𠊎等 ngǎi dén(nén)、饒平 má(ma)）係該隻音節个另一讀；前背有空白个，
  // 用音節數分：內容音節數＝字數係整詞換（ngǎi dén (ngǎi nén)），一隻音節係換前一隻（ngǎi dén (nén)）。Obs IME C8-6
  if (!/[（(]/.test(word) && /[（(]/.test(phonetic)) {
    const SYL = '[A-Za-z\u00C0-\u024F\u0300-\u036F]+';
    const tight = new RegExp(`(${SYL})[（(]([^）)\\s]+)[）)]`, 'g');
    if (tight.test(phonetic)) {
      const outer = phonetic.replace(tight, '$1');
      const inner = phonetic.replace(tight, '$2');
      return [{ word, phonetic: outer }, { word, phonetic: inner }];
    }
    const spaced = phonetic.match(/^(.*?)\s+[（(]([^）)]+)[）)](.*)$/);
    const n = headwordLength(word);
    if (spaced && n > 0 && !/[（(]/.test(spaced[3])) {
      const [, pre, content, post] = spaced;
      const outer = `${pre} ${post}`.trim();
      const preToks = readingTokens(pre);
      if (readingTokens(outer).length === n) {
        if (readingTokens(content).length === n) {
          return [{ word, phonetic: outer }, { word, phonetic: content.trim() }];
        }
        if (readingTokens(content).length === 1 && preToks.length > 0) {
          const inner = [...preToks.slice(0, -1), content.trim(), post.trim()].filter(Boolean).join(' ');
          return [{ word, phonetic: outer }, { word, phonetic: inner }];
        }
      }
    }
  }
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
    // 讀音無括號（多半係【】變體）：音節數對得著哪隻詞目就分哪隻，
    // 毋好兩隻都分（調羹仔（湯匙）【tiǎu/těu gáng è】、帖（帖仔）【tiàb è】）。Obs IME C8-6
    // 有一邊無算得着个字（數字詞 8（八））照舊兩邊都分，數字詞收毋收等 C8 第 5 項
    const m = readingTokens(phonetic).length;
    const n1 = headwordLength(w1), n2 = headwordLength(w2);
    const fits1 = n1 === m;
    const fits2 = n2 === m;
    const both = fits1 === fits2 || n1 === 0 || n2 === 0;
    if (w1 && phonetic && (both || fits1)) res.push({ word: w1, phonetic });
    if (w2 && phonetic && (both || fits2)) res.push({ word: w2, phonetic });
  } else {
    const cleanW = word.replace(/[（\(\)）]/g, '').trim();
    const cleanP = phonetic.replace(/[（\(\)）]/g, '').trim();
    if (cleanW && cleanP) res.push({ word: cleanW, phonetic: cleanP });
  }
  return res;
}

// 讀音欄看毋懂、毋好猜个，建置尾印出來分人工看
const readingProblems = [];

/** 詞目个字數：漢字、□、注音……都算，標點、空白、ASCII 毋算。 */
function headwordLength(word) {
  return Array.from(word).filter(c => !/[\s\x00-\x7f、，。…－—（）()「」]/.test(c)).length;
}

function readingTokens(phonetic) {
  return phonetic.trim().split(/[\s\-]+/).filter(Boolean);
}

/** 每隻音節个 a/b 展開做全部組合 */
function expandSyllableSlashes(tokens) {
  return tokens.reduce((acc, tok) => {
    const opts = tok.split('/').filter(Boolean);
    return acc.flatMap(prefix => opts.map(o => [...prefix, o]));
  }, [[]]).map(t => t.join('-'));
}

/**
 * 讀音欄个 `/` 有兩種意思（認證寫个時節淨顧人目）：單音節換（開始 kói-cìi/sìi）
 * 抑係整詞換（溜走 tiu-zèu/liu-zèu）。用「音節數＝詞目字數」判斷。
 * 另外有用 `-` 串兩隻整詞讀音个（佢等 ǐ-dén-ǐ-nén、哪仔 nai-è-lai/nai-é）：
 * 音節數係字數兩倍，切做兩隻整詞再展開。
 * 客源翠教典顯示欄个 ` / `（前後有空白）一定係整詞換。Obs IME C8-5、C8-6。
 */
function expandReadingAlternatives(word, phonetic, quiet = false) {
  if (/\s\/\s/.test(phonetic)) {
    return phonetic.split(/\s+\/\s+/).flatMap(p => expandReadingAlternatives(word, p, quiet));
  }
  const n = headwordLength(word);
  const toks = readingTokens(phonetic);
  if (n === 0 || toks.length === 0) return [phonetic];
  const hasSlash = phonetic.includes('/');
  if (toks.length === n) return hasSlash ? expandSyllableSlashes(toks) : [phonetic];
  if (hasSlash) {
    const whole = phonetic.split('/').map(p => p.trim());
    if (whole.every(p => readingTokens(p).length === n)) return whole;
  }
  if (toks.length === 2 * n) {
    return [toks.slice(0, n), toks.slice(n)].flatMap(expandSyllableSlashes);
  }
  if (hasSlash || toks.length === 2 * n) {
    if (!quiet) readingProblems.push(`${word}\t${phonetic}`);
    return hasSlash ? [] : [phonetic];
  }
  return [phonetic];
}

/**
 * 、 並列个詞目摎讀音照順序配（今下、現下 gím/gín-ha、hien-ha）。
 * 數量無共樣、抑係音節數對毋著个，印出來分人工看。
 * 詞目有「，」个係一句話（俗諺），毋拆，淨拿忒讀音裡个標點。Obs IME C8-8
 */
function splitListedPair(word, phonetic) {
  if (/[，,；;]/.test(word)) {
    return [{ word, phonetic: phonetic.replace(/[、，,；;]/g, ' ').replace(/\s+/g, ' ').trim() }];
  }
  if (!word.includes('、')) return [{ word, phonetic }];
  const words = word.split('、').map(w => w.trim()).filter(Boolean);
  const phons = phonetic.split('、').map(p => p.trim()).filter(Boolean);
  if (words.length === phons.length) {
    return words.map((w, i) => ({ word: w, phonetic: phons[i] }));
  }
  readingProblems.push(`${word}\t${phonetic}`);
  return [];
}

function processWordPhoneticPairs(word, rawPhonetic) {
  const readings = splitReadingVariants(rawPhonetic);
  const pairs = [];
  for (const r of readings) {
    for (const parenPair of expandParens(word, r.phonetic)) {
      for (const pair of splitListedPair(parenPair.word, parenPair.phonetic)) {
        for (const phonetic of expandReadingAlternatives(pair.word, pair.phonetic)) {
          // 調值轉過還有數字：調值表無收（教典海陸 fan31、han1155 voi5533、du324），印出來分人工看，毋收
          // 也擋拉丁字母以外个記號（海陸教典 ˋ、「詞目刪除」、全形標點），毋擋會污染音節庫。
          if (/[0-9]/.test(phonetic) || !/^[\p{Script=Latin}\p{M}\s\-.…]+$/u.test(phonetic)) {
            readingProblems.push(`${pair.word}\t${phonetic}`);
            continue;
          }
          pairs.push({ ...pair, phonetic, special: r.special });
        }
      }
    }
  }
  return pairs;
}

/** 拿忒調符、空白摎連字號个讀音，比相像用 */
function tonelessKey(phonetic) {
  return readingTokens(phonetic).map(removeTone).join('');
}

function editDistance(a, b) {
  const d = Array.from({ length: a.length + 1 }, (_, i) => [i]);
  for (let j = 1; j <= b.length; j++) d[0][j] = j;
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
    }
  }
  return d[a.length][b.length];
}

/**
 * 【】變體淨有讀音（南四縣 zùi mí）：係同一隻詞个另一種腔口讀法，所以分「音節數共樣、
 * 主讀音上相像」个詞目。盡尾（第一尾、最尾）【zùi mí】→ 最尾，毋係盡尾；
 * 本來（原本、原來）【ngiǎn bùn、ngiǎn lǒi】→ 原本、原來。Obs IME C8-8
 */
function assignVariantReadings(mainPairs, variantPhonetic, lineWord) {
  const out = [];
  for (const r of splitReadingVariants(variantPhonetic)) {
    // 讀音个 、 摎括號拆做一隻一隻讀音
    // 先拆括號，正拆 、（括號肚裡可能有 、；variant-parser 也會將兩組【】合做幾組括號：
    // 個零月 ge lǎng ngiad (ge nǎng ngiad) (ge go ngiad、zàg go ngiad)）
    const groups = [...r.phonetic.matchAll(/[（(]([^）)]+)[）)]/g)].map(g => g[1].trim());
    const outerText = r.phonetic.replace(/\s*[（(][^）)]+[）)]/g, ' ').trim();
    const parts = [outerText, ...groups];
    const pieces = parts.flatMap(p => p.split('、')).map(p => p.trim()).filter(Boolean);
    for (const piece of pieces) {
      let best = [];
      let bestDist = Infinity;
      for (const t of mainPairs) {
        // 音節數愛啱啱好（逐音節 / 抑係整詞 /），毋用「兩倍切兩隻」該條：帖（帖仔）【tiàb è】係帖仔个
        const n = headwordLength(t.word);
        const fits = readingTokens(piece).length === n
          || (piece.includes('/') && piece.split('/').every(p => readingTokens(p).length === n));
        if (!fits) continue;
        const alts = expandReadingAlternatives(t.word, piece, true);
        if (alts.length === 0) continue;
        const dist = editDistance(tonelessKey(alts[0]), tonelessKey(t.phonetic));
        if (dist < bestDist) { bestDist = dist; best = [t.word]; }
        else if (dist === bestDist && !best.includes(t.word)) best.push(t.word);
      }
      // 共樣相像、主讀音也共樣（屋下（屋家）vùg ká【lùg ká】）：兩隻都分
      if (best.length > 1) {
        const mains = new Set(best.map(w => mainPairs.filter(t => t.word === w).map(t => tonelessKey(t.phonetic)).sort().join('|')));
        if (mains.size === 1) {
          for (const w of best) for (const phonetic of expandReadingAlternatives(w, piece, true)) out.push({ word: w, phonetic, special: r.special });
          continue;
        }
      }
      if (best.length !== 1) {
        readingProblems.push(`${lineWord}【${piece}】\t${best.length ? '相像个有 ' + best.join('／') : '無詞目對得著'}`);
        continue;
      }
      for (const phonetic of expandReadingAlternatives(best[0], piece)) {
        out.push({ word: best[0], phonetic, special: r.special });
      }
    }
  }
  return out;
}

/** 一行資料（主讀音＋【】變體）个全部（詞目, 讀音） */
function linePairs(parsed) {
  const pairs = [];
  const mainPairs = (parsed.main && parsed.main.word && parsed.main.phonetic)
    ? processWordPhoneticPairs(parsed.main.word, parsed.main.phonetic) : [];
  pairs.push(...mainPairs);
  for (const v of parsed.variants) {
    if (!v.phonetic) continue;
    if (v.word) {
      pairs.push(...processWordPhoneticPairs(v.word, v.phonetic));
    } else if (mainPairs.length) {
      pairs.push(...assignVariantReadings(mainPairs, v.phonetic, parsed.main.word));
    }
  }
  // 詞目有「…」个（…兜、拜…）係用法說明，毋做候選；歇後語个「－－」、阿拉伯數字詞先留
  return pairs.filter(p => !p.word.includes('…'));
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

/**
 * 四縣認證个【】變體分腔 (Obs IME dialects D6)：認證四縣个【】係南四縣音，【（）】係美濃音。
 * 回傳 { main: 北四縣, south: 南四縣變體, other: 其他（美濃等） }，同樣濾掉「…」。
 * 四縣只用 main；南四縣用 south（無就用附表）。美濃另外做成輸出選項，先毋收。
 */
function linePairsSplit(parsed) {
  const clean = pairs => pairs.filter(p => !p.word.includes('…'));
  const main = (parsed.main && parsed.main.word && parsed.main.phonetic)
    ? processWordPhoneticPairs(parsed.main.word, parsed.main.phonetic) : [];
  const south = [];
  const other = [];
  for (const v of parsed.variants) {
    if (!v.phonetic) continue;
    const pairs = v.word
      ? processWordPhoneticPairs(v.word, v.phonetic)
      : (main.length ? assignVariantReadings(main, v.phonetic, parsed.main.word) : []);
    (v.label === '南' ? south : other).push(...pairs);
  }
  return { main: clean(main), south: clean(south), other: clean(other) };
}

/**
 * 南四縣附表（客源翠 data/sub-dialects-fill/南四縣.json）：認證四縣無標【南】个詞條，官方漏標，
 * 毋係「南北共樣」，程式推導个南四縣音。鍵 `來源檔|編號`。教典南四縣層以外个推導音詞頻打折；
 * `待確認`（單字詞，教典音摎推導無共樣）先毋收，推毋出个（待審）本來就毋在附表。
 */
const SOUTH_DERIVED_FREQ_FACTOR = 0.8;
function loadSouthFill() {
  const fillPath = path.join(HAKSPRING_DATA, 'sub-dialects-fill', '南四縣.json');
  const fill = new Map();
  if (!fs.existsSync(fillPath)) return fill;
  for (const e of JSON.parse(fs.readFileSync(fillPath, 'utf8')).entries) {
    if (e['待確認']) continue;
    fill.set(`${e['來源檔']}|${e['編號']}`, e);
  }
  return fill;
}

/**
 * 推毋出个（`待審-南四縣.json`，有音節無南四縣證據，附表毋收）：用戶打字時「北四縣有，南四縣就無」
 * 比「南四縣讀音可能略有差」更傷（20261025 維護者：繼續）。所以暫時用北四縣音權充，詞頻打對折，
 * 有真个南四縣音就排在伊前面。人工審音後會進附表，到時自然取代。SOUTH_INCLUDE_PENDING = false 就毋收。
 */
const SOUTH_INCLUDE_PENDING = true;
const SOUTH_PENDING_FREQ_FACTOR = 0.5;
// 美濃開關開咧時，美濃讀音排南四縣同詞个普通讀音前面（開這個開關个人就係愛打美濃）
const MEINONG_FREQ_FACTOR = 1.02;
/** 南四縣整隻詞都缺、抄北四縣音个詞頻倍數（--lect all 合併時做） */
const SOUTH_STAND_IN_FREQ_FACTOR = 0.5;
/** 美濃變體个 subtag 位元（引擎 KAUTIAN_SUBTAG_MEINONG_BIT，借上游腔口位元 3） */
const MEINONG_SUBTAG = (1 << 0) | (1 << 3);
function loadSouthPending() {
  const pendingPath = path.join(HAKSPRING_DATA, 'sub-dialects-fill', '待審-南四縣.json');
  const pending = new Map();
  if (!SOUTH_INCLUDE_PENDING || !fs.existsSync(pendingPath)) return pending;
  for (const e of JSON.parse(fs.readFileSync(pendingPath, 'utf8')).entries) {
    pending.set(`${e['來源檔']}|${e['編號']}`, e);
  }
  return pending;
}

/**
 * 南四縣音推導（頂著北四縣音時用，維護者 20261025 准）：
 * 1. 規則：舌根音（g、k、ng）、喉音（h）、零聲母 + -ien/-ied → -ian/-iad，調號挪去 a
 *    （kiěn→kiǎn、ngièd→ngiàd、ien→ian）。客源翠附表个 `rule-ien`，卓桃音另外
 * 2. 字級對照：由南北都有、音節數＝字數个詞，學「這隻字北讀 X、南讀 Y」，出現 ≥2 擺、佔 ≥8 成正用
 * 毋確定个音節就維持北四縣音。
 */
const DIACRITIC_MARKS = new Set(['\u0300', '\u0301', '\u0302', '\u0304', '\u030c']);
function splitToneMark(syllable) {
  let mark = '';
  let base = '';
  for (const ch of syllable.normalize('NFD')) {
    if (DIACRITIC_MARKS.has(ch)) mark = ch; else base += ch;
  }
  return { base: base.normalize('NFC'), mark };
}
function ruleIen(syllable) {
  const { base, mark } = splitToneMark(syllable);
  const m = /^(ng|g|k|h|)(ie)([nd])$/.exec(base);
  if (!m) return syllable;
  // 調號擺 a 頂（a 優先）：i + a + 調號 + n/d
  return (m[1] + 'ia' + mark + m[3]).normalize('NFC');
}
const SOUTH_TABLE_MIN_COUNT = 2;
const SOUTH_TABLE_MIN_RATIO = 0.8;
function buildSouthTable(records) {
  const NORTH = LECTS['四'].code;
  const SOUTH = LECTS['南'].code;
  const readings = { [NORTH]: new Map(), [SOUTH]: new Map() };
  for (const r of records) {
    const lect = r.subtag >> LECT_SHIFT;
    if (lect !== NORTH && lect !== SOUTH) continue;
    if (r.subtag & MEINONG_SUBTAG & ~1) continue;
    const m = readings[lect];
    if (!m.has(r.word)) m.set(r.word, new Set());
    m.get(r.word).add(r.tl);
  }
  const table = new Map(); // `${字}|${北音節}` -> Map(南音節 -> 次數)
  const words = [];
  for (const [word, northSet] of readings[NORTH]) {
    const southSet = readings[SOUTH].get(word);
    if (!southSet || northSet.size !== 1 || southSet.size !== 1) continue;
    const chars = Array.from(word);
    const north = [...northSet][0].split('-');
    const south = [...southSet][0].split('-');
    if (chars.length !== north.length || north.length !== south.length) continue;
    words.push({ chars, north, south });
    chars.forEach((c, i) => {
      const key = `${c}|${north[i]}`;
      if (!table.has(key)) table.set(key, new Map());
      const counts = table.get(key);
      counts.set(south[i], (counts.get(south[i]) || 0) + 1);
    });
  }
  return { table, words };
}
/** 一隻音節个南四縣音：字級對照（扣掉 exclude 自家貢獻个）→ 規則 → 維持北四縣音 */
function deriveSouthSyllable(table, char, northSyl, exclude) {
  const counts = char ? table.get(`${char}|${northSyl}`) : null;
  if (counts) {
    let total = 0, top = null, topN = 0;
    for (const [s, n0] of counts) {
      const n = n0 - (exclude && exclude === s ? 1 : 0);
      total += n;
      if (n > topN) { topN = n; top = s; }
    }
    if (top && total >= SOUTH_TABLE_MIN_COUNT && topN / total >= SOUTH_TABLE_MIN_RATIO) return top;
  }
  return ruleIen(northSyl);
}
function deriveSouthReading(table, word, northTl, exclude) {
  const chars = Array.from(word);
  const north = northTl.split('-');
  const aligned = chars.length === north.length;
  return north.map((s, i) => deriveSouthSyllable(table, aligned ? chars[i] : null, s, exclude ? exclude[i] : null)).join('-');
}

function printReadingProblems() {
  if (readingProblems.length) {
    console.log(`  ⚠ 讀音看毋懂、毋好猜个 ${readingProblems.length} 筆（Obs IME C8-6，愛人工看）：`);
    for (const p of [...new Set(readingProblems)]) console.log(`    ${p}`);
  }
}

/** 單腔萃取：讀客源翠資料，回傳這一腔个詞條（腔號已寫入 subtag）。 */
function buildLectRecords() {
console.log(`▶ [1/4] 開始萃取客源翠語料並展開變體（${LECT.name}）...`);

const map = new Map(); // key: word + '|' + fusedNotone -> entry

// 1. 處理認證五級（按級別賦予詞頻）
const certFiles = [
  { level: '基', freq: 50000, levelName: '基礎級' },
  { level: '初', freq: 45000, levelName: '初級' },
  { level: '中', freq: 40000, levelName: '中級' },
  { level: '中高', freq: 35000, levelName: '中高級' },
  { level: '高', freq: 30000, levelName: '高級' }
].map(c => ({ file: `113${CERT_LECT_KEY}${c.level}.json`, freq: c.freq, name: `${LECT.name}${c.levelName}` }));
const southFill = LECT_KEY === '南' ? loadSouthFill() : new Map();
const southPending = LECT_KEY === '南' ? loadSouthPending() : new Map();
let southFromPending = 0;
let southMeinong = 0;
let southFromVariant = 0;
let southFromFill = 0;
let southNoReading = 0;

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
    // 四縣：淨收北四縣主讀音（【】係南四縣／美濃，毋係四縣）。南四縣：【南】變體；無標个用附表
    let pairs;
    if (LECT_KEY === '四') {
      pairs = linePairsSplit(parsed).main.map(p => ({ ...p, scale: 1 }));
    } else if (LECT_KEY === '南') {
      const split = linePairsSplit(parsed);
      const stem = cf.file.replace(/\.json$/, '');
      const fillEntry = southFill.get(`${stem}|${parts[0]}`);
      if (split.south.length) {
        southFromVariant++;
        // 標个南四縣音排頭前；北四縣主讀音南四縣人亦有講，留起來但詞頻略低
        pairs = [
          ...split.south.map(p => ({ ...p, scale: 1 })),
          ...split.main.map(p => ({ ...p, scale: 0.98 })),
        ];
      } else if (fillEntry) {
        southFromFill++;
        const scale = fillEntry['推導層'] === '教典南四縣' ? 1 : SOUTH_DERIVED_FREQ_FACTOR;
        pairs = processWordPhoneticPairs(fillEntry['詞目'], fillEntry['讀音'])
          .filter(p => !p.word.includes('…'))
          .map(p => ({ ...p, scale }));
      } else if (southPending.has(`${stem}|${parts[0]}`)) {
        southFromPending++;
        const pending = southPending.get(`${stem}|${parts[0]}`);
        pairs = processWordPhoneticPairs(pending['詞目'], pending['讀音'])
          .filter(p => !p.word.includes('…'))
          .map(p => ({ ...p, scale: SOUTH_PENDING_FREQ_FACTOR }));
      } else {
        southNoReading++;
        pairs = [];
      }
    } else {
      pairs = linePairs(parsed).map(p => ({ ...p, scale: 1 }));
    }
    // 美濃【（）】音：南四縣个子開關（預設關），毋係獨立个腔。標个美濃音做南四縣腔个變體記錄，
    // subtag 帶美濃位元，引擎淨在美濃開關開咧時正出。詞頻略低，毋搶南四縣自家个音
    if (LECT_KEY === '南') {
      const meinong = linePairsSplit(parsed).other.map(p => ({ ...p, scale: MEINONG_FREQ_FACTOR, meinong: true }));
      southMeinong += meinong.length;
      pairs = [...pairs, ...meinong];
    }

    for (const pair of pairs) {
      const tokens = pair.phonetic.trim().split(/[\s\-]+/);
      const notoneSylls = tokens.map(removeTone).filter(Boolean);
      if (notoneSylls.length === 0) continue;
      const fusedNotone = notoneSylls.join('');
      const hyphenatedPhonetic = tokens.join('-');
      // 客話聲母縮寫（多字母聲母長度優先，單音節詞留空）
      const fusedAbbrev = deriveHakfaAbbrev(notoneSylls);
      let word = pair.word;
      // 客話正字規範：個人 ge-ngǐn -> 个人 ge-ngǐn，保留個人 gòg-ngǐn
      if (word === '個人' && fusedNotone === 'gengin') {
        word = '个人';
      }
      // 美濃變體另外一格：同詞同無調音（淨調號無共樣）也毋會被主讀音擋忒
      const key = word + '|' + fusedNotone + (pair.meinong ? '|美濃' : '');

      const boostedFreq = boostFrequency(word, Math.round(cf.freq * pair.scale));
      if (!map.has(key) || map.get(key).freq < boostedFreq) {
        map.set(key, {
          word: word,
          tl: hyphenatedPhonetic,
          tl_num: hyphenatedPhonetic,
          tl_notone: fusedNotone,
          tl_abbrev: fusedAbbrev,
          freq: boostedFreq,
          syllable_count: Math.min(notoneSylls.length, 4),
          syllables: notoneSylls,
          bitmask: CERT_BITMASK,
          ...(pair.meinong ? { subtag: MEINONG_SUBTAG } : {})
        });
      }
    }
  }
}

console.log(`  ✓ 認證詞彙（展開變體後）: ${map.size} 筆`);
if (LECT_KEY === '南') {
  console.log(`    南四縣認證：標【南】${southFromVariant} 條、附表推導 ${southFromFill} 條、待審以北四縣音權充 ${southFromPending} 條、完全無（毋收）${southNoReading} 條；美濃音（子開關）${southMeinong} 筆`);
}

// 2. 處理教典（讀取 20250630-<腔>.json，取得正宗 HagPinPlus 顯示欄位）
const gipJsonPath = path.join(HAKSPRING_DATA, 'gip', `20250630-${LECT_KEY}.json`);
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
      sourceName: `教典${LECT.name}`
    };

    const parsed = getLineVariants(row);
    const pairs = linePairs(parsed);

    for (const pair of pairs) {
      const tokens = pair.phonetic.trim().split(/[\s\-]+/);
      const notoneSylls = tokens.map(removeTone).filter(Boolean);
      if (notoneSylls.length === 0) continue;
      const fusedNotone = notoneSylls.join('');
      const hyphenatedPhonetic = tokens.join('-');
      // 客話聲母縮寫（多字母聲母長度優先，單音節詞留空）
      const fusedAbbrev = deriveHakfaAbbrev(notoneSylls);
      let word = pair.word;
      // 客話正字規範：個人 ge-ngǐn -> 个人 ge-ngǐn，保留個人 gòg-ngǐn
      if (word === '個人' && fusedNotone === 'gengin') {
        word = '个人';
      }
      // 「特」讀音：淨差聲調也另外一筆（鍵加帶調讀音）；共一隻帶調讀音就併
      const plainKey = word + '|' + fusedNotone;
      const samePlain = map.has(plainKey) && map.get(plainKey).tl.normalize('NFC') === hyphenatedPhonetic.normalize('NFC');
      const key = pair.special && !samePlain
        ? word + '|' + fusedNotone + '|' + hyphenatedPhonetic.normalize('NFC')
        : plainKey;
      const baseFreq = Math.max(100, gipDefaultFreq - word.length * 1000);
      const freq = boostFrequency(word, baseFreq);

      if (map.has(key)) {
        // Also in the certification lists: the certification reading and frequency stay,
        // the row just gains the MOE source bit.
        map.get(key).bitmask |= KAUTIAN_BITMASK;
      } else {
        map.set(key, {
          word: word,
          tl: hyphenatedPhonetic,
          tl_num: hyphenatedPhonetic,
          tl_notone: fusedNotone,
          tl_abbrev: fusedAbbrev,
          freq: freq,
          syllable_count: Math.min(notoneSylls.length, 4),
          syllables: notoneSylls,
          bitmask: KAUTIAN_BITMASK
        });
      }
    }
  }
}

// 3. HakSpring character supplement: single-character readings for characters
//    that MOE and certification data only have inside words, aligned by HakSpring.
//    Its own source: bit 6 (upstream's `kungge`, relabelled and locked on).
//    Produced by HakSpring `python3 process_all_data.py --only-supp`; see docs/hakfa/supp-dict.md.
const suppJsonPath = path.join(HAKSPRING_DATA, 'supp', `補字典-${LECT_KEY}.json`);
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
  console.log(`  ✓ 客源翠補字典（${LECT.name}）: ${added} 筆`);
}

const records = Array.from(map.values());
for (const r of records) {
  // kautian_subtag: MOE and certification rows have the main bit (1); the supplement has 0.
  // The lect code goes on top in bits 13-15.
  r.subtag = (r.subtag ?? 1) | (LECT.code << LECT_SHIFT);
}
// 斷言：嚴格確保所有詞條的標音 tl 都不包含任何數字
for (const r of records) {
  if (/[0-9]/.test(r.tl)) {
    throw new Error(`[Assertion Error] 詞條 "${r.word}" 拼音仍然包含數字: "${r.tl}"`);
  }
}
// 斷言：every row has a known source and a lect; a MOE row needs a subcollection bit,
// or the engine drops it once subcollection filtering is active (dictionary_reader.rs passes_filter).
const KNOWN_SOURCES = KAUTIAN_BITMASK | CERT_BITMASK | SUPP_BITMASK;
for (const r of records) {
  if (!r.bitmask || (r.bitmask & ~KNOWN_SOURCES) !== 0) {
    throw new Error(`[Assertion Error] 詞條 "${r.word}" 來源位元不對: ${r.bitmask}`);
  }
  const lectCode = r.subtag >> LECT_SHIFT;
  if (lectCode === 0) {
    throw new Error(`[Assertion Error] 詞條 "${r.word}" 無腔`);
  }
  if ((r.bitmask & KAUTIAN_BITMASK) && (r.subtag & ((1 << LECT_SHIFT) - 1)) === 0) {
    throw new Error(`[Assertion Error] 教典詞條 "${r.word}" 無子集位元`);
  }
}
// 依照詞頻排序（客語第一助詞「个」因被賦予 55000 特級頻率，自然名列前茅）
records.sort((a, b) => b.freq - a.freq);
console.log(`  ✓ 總計詞庫筆數 (認證 + 教典): ${records.length} 筆`);
return records;
}

let records;
if (IS_ALL) {
  // 每腔跑一次（腔个調值表摎檔名是模組載入時定个，所以用子程序）
  const tmpDir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'hakfa-all-'));
  records = [];
  for (const key of Object.keys(LECTS)) {
    const dump = path.join(tmpDir, `${LECTS[key].id}.json`);
    const res = spawnSync(process.execPath, [__filename, '--lect', key, '--dump-records', dump], { stdio: 'inherit' });
    if (res.status !== 0) {
      throw new Error(`--lect ${key} 萃取失敗（exit ${res.status}）`);
    }
    records.push(...JSON.parse(fs.readFileSync(dump, 'utf8')));
  }
  fs.rmSync(tmpDir, { recursive: true, force: true });
  // 美濃讀音摎南四縣同詞个普通讀音完全共樣，就毋使再標「美」：丟忒美濃那筆，免得候選重複
  {
    const SOUTH = LECTS['南'].code;
    const isMeinong = r => (r.subtag >> LECT_SHIFT) === SOUTH && (r.subtag & (1 << 3));
    const plain = new Set(records.filter(r => (r.subtag >> LECT_SHIFT) === SOUTH && !(r.subtag & (1 << 3))).map(r => `${r.word}|${r.tl}`));
    const before = records.length;
    for (let i = records.length - 1; i >= 0; i--) {
      if (isMeinong(records[i]) && plain.has(`${records[i].word}|${records[i].tl}`)) records.splice(i, 1);
    }
    if (records.length !== before) console.log(`  ✓ 美濃音摎南四縣普通讀音共樣，合併 ${before - records.length} 筆`);
  }
  // 南四縣缺个，用北四縣个頂著（維護者原則，20261025）：北四縣有、南四縣「整隻詞」都無个
  // （無任何一種讀音），抄北四縣个讀音入南四縣，詞頻 ×0.5。南四縣若有該詞（毋論讀音同北四縣
  // 有無共樣）就毋抄——那係南北个差異，毋係缺；抄過去會同南四縣自家个資料衝突。
  // 補字典个列（單字補音）毋抄，由補字典線自家處理。
  {
    const SOUTH = LECTS['南'].code;
    const NORTH = LECTS['四'].code;
    const lectOf = r => r.subtag >> LECT_SHIFT;
    const southWords = new Set(records.filter(r => lectOf(r) === SOUTH).map(r => r.word));
    const { table, words: trainWords } = buildSouthTable(records);
    // 推導準確率（leave-one-out）：南北都有个詞，扣掉自家貢獻，看推導个南四縣音對毋對
    let hit = 0, syllableHit = 0, syllableAll = 0;
    for (const w of trainWords) {
      const pred = w.chars.map((c, i) => deriveSouthSyllable(table, c, w.north[i], w.south[i]));
      pred.forEach((s, i) => { syllableAll++; if (s === w.south[i]) syllableHit++; });
      if (pred.every((s, i) => s === w.south[i])) hit++;
    }
    const baseline = trainWords.filter(w => w.north.every((s, i) => s === w.south[i])).length;
    console.log(`  ✓ 南四縣推導驗證（${trainWords.length} 隻南北都有个詞）：整詞全對 ${(100 * hit / trainWords.length).toFixed(1)}%（毋推導、照北四縣音：${(100 * baseline / trainWords.length).toFixed(1)}%）；音節 ${(100 * syllableHit / syllableAll).toFixed(1)}%`);
    const standIns = records
      // 補字典（單字補音、次方言補音）有自家處理摎推導原則（客源翠 dict-augmentation），毋在這抄：
      // 南四縣補字典缺个字，由補字典線決定，毋用北四縣頂著
      .filter(r => lectOf(r) === NORTH && !(r.bitmask & SUPP_BITMASK) && !southWords.has(r.word))
      .map(r => {
        const tl = deriveSouthReading(table, r.word, r.tl);
        return {
          ...r,
          tl,
          tl_num: tl,
          // 無調音節、縮寫拼法愛跟讀音走：ien→ian 會改拼寫
          tl_notone: tl.split('-').map(removeTone).join(''),
          tl_abbrev: deriveHakfaAbbrev(tl.split('-').map(removeTone)),
          syllables: tl.split('-').map(removeTone),
          subtag: (r.subtag & ((1 << LECT_SHIFT) - 1)) | (SOUTH << LECT_SHIFT),
          freq: Math.max(100, Math.round(r.freq * SOUTH_STAND_IN_FREQ_FACTOR)),
          derived: tl !== r.tl,
        };
      });
    records.push(...standIns);
    console.log(`  ✓ 南四縣缺个詞，用北四縣音頂著: ${new Set(standIns.map(r => r.word)).size} 隻詞 / ${standIns.length} 筆，其中推導過（音有變）${standIns.filter(r => r.derived).length} 筆`);
  }
  // 全腔合併排序：詞頻高个在前，同分四縣先（Array.sort 穩定）
  records.sort((a, b) => b.freq - a.freq);
  console.log(`\n  ✓ 全腔合併詞庫筆數: ${records.length} 筆`);
} else {
  records = buildLectRecords();
  if (DUMP_RECORDS_PATH) {
    fs.writeFileSync(DUMP_RECORDS_PATH, JSON.stringify(records), 'utf8');
    printReadingProblems();
    process.exit(0);
  }
}

// 輸出 CSV
const csvOutputDir = path.join(REPO_ROOT, 'dictionary/sources/official/hakspring/data');
fs.mkdirSync(csvOutputDir, { recursive: true });
// 四縣照舊覆蓋出貨檔；別的腔寫自己的檔，免蓋掉出貨詞庫。
const csvPath = path.join(csvOutputDir, isShipLect ? 'hakspring.csv' : `hakspring-${LECT.id}.csv`);
const csvLines = ['word,tl,tl_num,tl_notone,tl_abbrev,frequency,syllable_count,bitmask,lect'];
for (const r of records) {
  csvLines.push(`"${r.word}","${r.tl}","${r.tl_num}","${r.tl_notone}","${r.tl_abbrev}",${r.freq},${r.syllable_count},${r.bitmask},${LECT_KEY_BY_CODE[r.subtag >> LECT_SHIFT]}`);
}
fs.writeFileSync(csvPath, csvLines.join('\n'), 'utf8');
printReadingProblems();
console.log(`  ✓ 已輸出 CSV 至 ${csvPath}`);

// -------------------------------------------------------------
// [2/4] 建置 dictionary.bin (TKDB v4)
// -------------------------------------------------------------
console.log('\n▶ [2/4] 建置 dictionary.bin (TKDB v4)...');
const outDirs = isShipLect ? [
  path.join(REPO_ROOT, 'output/hakfa'),
  path.join(REPO_ROOT, 'dictionary/output/hakfa'),
  path.join(REPO_ROOT, 'dictionary/output/hakfa-all'),
  path.join(REPO_ROOT, 'assets/dictionaries')
] : [path.join(REPO_ROOT, `dictionary/output/hakfa-${LECT.id}`)];
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

  recBuf.writeUInt16LE(r.bitmask, 0);      // bitmask: MOE 1 << 0, certification 1 << 1, supplement 1 << 6
  recBuf.writeUInt32LE(r.freq, 2);          // frequency
  recBuf.writeUInt8(hanziBuf.length, 6);    // hanzi_len
  recBuf.writeUInt8(tlBuf.length, 7);       // tl_len
  recBuf.writeUInt8(r.syllable_count, 8);   // syllable_count
  recBuf.writeUInt16LE(r.subtag, 9);       // kautian_subtag: 1 (KAUTIAN_SUBTAG_MAIN_BIT: has_main = true); supplement 0; lect in bits 13-15
  recBuf.writeUInt16LE(walkerCost(r.word, r.freq), 11); // walker_cost (v4), see tkdb_v4.js
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
