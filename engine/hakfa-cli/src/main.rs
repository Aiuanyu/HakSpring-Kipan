use std::io::{self, Write};
use std::path::Path;
use lexicon::dictionary_reader::DictionaryReader;
use lexicon::prefix_index::PrefixIndex;

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let fst_path = repo_root.join("output/hakfa/dictionary.fst");
    let bin_path = repo_root.join("output/hakfa/dictionary.bin");

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
    println!("輸入 'q' 或 'exit' 離開。");
    println!("--------------------------------------------------------");

    loop {
        print!("客話拼音 > ");
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

        println!("  找到 {} 筆候選詞（顯示前 10 筆）：", candidates.len());
        for (i, (freq, hanzi, tl)) in candidates.iter().take(10).enumerate() {
            println!("    {}. {} [{}] (詞頻: {})", i + 1, hanzi, tl, freq);
        }
        println!();
    }
}
