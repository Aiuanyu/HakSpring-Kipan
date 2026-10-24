use std::io::{self, Write};
use std::path::Path;
use lexicon::dictionary_reader::{DictionaryReader, Filter, WIRE_LECT_SHIFT};
use lexicon::prefix_index::PrefixIndex;

/// Lect names for the prompt and the candidate list (code → Hakka self-name; 0 = every lect).
fn lect_name(lect: u32) -> &'static str {
    match lect {
        0 => "全腔",
        1 => "四縣 xiien",
        2 => "海陸 hoiliug",
        _ => "（未建）",
    }
}

/// `ToggleLect`: the same one-key round trip the desktop shells bind (四縣 ↔ 海陸), from
/// "every lect" it starts at 四縣.
fn next_lect(lect: u32) -> u32 {
    if lect == 1 { 2 } else { 1 }
}

/// Source bits 0-11 and the variant bit on, wanted lect on wire bits 27-29 — the same
/// shape `compute_filters` produces when the platform sends `DictionarySourceToggles.lect`.
fn lect_filter(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(0x0FFF | (1 << 12) | (lect << WIRE_LECT_SHIFT))
}

fn main() {
    // `--dict <dir>` picks the dictionary folder (default output/hakfa, the shipped Sixian one;
    // `dictionary/output/hakfa-all` is the merged all-lect build), `--lect N` the starting lect.
    let args: Vec<String> = std::env::args().collect();
    let arg_after = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
    let mut lect: u32 = arg_after("--lect").and_then(|v| v.parse().ok()).unwrap_or(0);
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let dict_dir = repo_root.join(arg_after("--dict").map(String::as_str).unwrap_or("output/hakfa"));
    let fst_path = dict_dir.join("dictionary.fst");
    let bin_path = dict_dir.join("dictionary.bin");

    println!("========================================================");
    println!("  客話企盤 (HakSpring-Kipan) — 客話打字 CLI 概念驗證 (PoC)");
    println!("========================================================");
    println!("載入辭典: {}", bin_path.display());

    let prefix_index = match PrefixIndex::open(&fst_path) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("無法開啟 FST 索引: {}", e);
            return;
        }
    };

    let dict = match DictionaryReader::open(&bin_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("無法開啟辭典二進位檔: {}", e);
            return;
        }
    };

    println!("詞庫總筆數: {} 筆", dict.record_count());
    println!("FST 索引鍵數: {} 條", prefix_index.entry_count());
    println!("--------------------------------------------------------");
    println!("提示：輸入客話拼音（例如：fadseu、ngiho、ioge、fls、或部分拼音），按 Enter 候選！");
    println!("輸入 ':l' 切換腔頭（四縣 ↔ 海陸），':l 0' 全腔，':l 1' 四縣，':l 2' 海陸。");
    println!("輸入 'q' 或 'exit' 離開。");
    println!("--------------------------------------------------------");

    loop {
        print!("[{}] 客話拼音 > ", lect_name(lect));
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let input = line.trim().to_lowercase();
        if input == "q" || input == "exit" {
            println!("再會！(Zai voi!)");
            break;
        }
        if input.is_empty() {
            continue;
        }
        if let Some(cmd) = input.strip_prefix(":l") {
            lect = match cmd.trim().parse::<u32>() {
                Ok(n) if n <= 7 => n,
                _ if cmd.trim().is_empty() => next_lect(lect),
                _ => {
                    println!("  [腔號愛 0 到 7，或淨打 :l 切換]");
                    continue;
                }
            };
            println!("  → 目前腔：{}", lect_name(lect));
            continue;
        }
        let filter = lect_filter(lect);

        // Try searching in different key families:
        // 1. Full/partial romanization: "tl:" + input
        // 2. Abbreviation: "tl-abbrev:" + input
        // 3. Hanzi: "hanzi:" + input
        let query_tl = format!("tl:{}", input);
        let mut rowids = prefix_index.lookup_prefix(&query_tl);
        if rowids.is_empty() {
            let query_abbrev = format!("tl-abbrev:{}", input);
            rowids = prefix_index.lookup_prefix(&query_abbrev);
        }
        if rowids.is_empty() {
            let query_hanzi = format!("hanzi:{}", input);
            rowids = prefix_index.lookup_prefix(&query_hanzi);
        }

        if rowids.is_empty() {
            println!("  [無相符詞彙]");
            continue;
        }

        // Collect matching records and sort by frequency (descending)
        let mut candidates = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for rowid in rowids {
            if let Some(record) = dict.record(rowid) {
                if !DictionaryReader::passes_filter(record.bitmask, record.kautian_subtag, &filter) {
                    continue;
                }
                if let Some(hanzi) = &record.hanji {
                    let key = (hanzi.clone(), record.tl.clone());
                    if !seen.contains(&key) {
                        seen.insert(key);
                        candidates.push((record.frequency, hanzi.clone(), record.tl.clone()));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        println!("  找到 {} 筆候選詞（{}，顯示前 10 筆）：", candidates.len(), lect_name(lect));
        for (i, (freq, hanzi, tl)) in candidates.iter().take(10).enumerate() {
            println!("    {}. {} [{}] (詞頻: {})", i + 1, hanzi, tl, freq);
        }
        println!();
    }
}
