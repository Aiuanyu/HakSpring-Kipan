//! 客話企盤 (HakSpring-Kipan) 全設定打字行為矩陣自動模擬器
//! 
//! 本工具自動驅動真實 Rust 引擎與客話詞庫，遍歷多種設定組合與客話測試語料，
//! 記錄每一步敲鍵的 Preedit、候選字清單、Lattice 切分與最終送字結果，
//! 並產出結構化 JSON 與視覺化 HTML 研究報告。

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use taigi_desktop_core::dictionary_artifacts::DictionaryArtifacts;
use taigi_desktop_core::engine::{self, CommitContinuousArgs, CommitScript};
use taigi_desktop_core::platform::DesktopPlatform;
use taigi_desktop_core::settings::{
    CandidateDisplayMode, EngineSettings, InputMode, SyllableSeparator,
};

const PLATFORM: DesktopPlatform = DesktopPlatform::MacOS;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CandidateSnapshot {
    slot: usize,
    hanji: Option<String>,
    roman: String,
    display_text: String,
    canonical_tl: String,
    consumed_bytes: u32,
    syllable_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StepSnapshot {
    key: String,
    preedit: String,
    candidate_count: usize,
    top_candidates: Vec<CandidateSnapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RunResult {
    setting_id: String,
    setting_name: String,
    test_id: String,
    corpus_text: String,
    input_keys: String,
    steps: Vec<StepSnapshot>,
    final_output: String,
    notes: String,
}

#[derive(Debug, Clone)]
struct SettingProfile {
    id: String,
    name: String,
    description: String,
    settings: EngineSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestCase {
    id: String,
    name: String,
    input: String,
    expected_meaning: String,
}

fn create_profiles() -> Vec<SettingProfile> {
    let base = EngineSettings::default();

    vec![
        SettingProfile {
            id: "hanji_default".into(),
            name: "1. 標準漢羅（漢字優先 + 雙軌並列 + 無分隔 None）".into(),
            description: "客話企盤預設：首選出諸夏字，羅馬字以無分隔 HagPinPlus 連寫，候選窗雙軌並列".into(),
            settings: EngineSettings {
                is_hanji_first: true,
                candidate_display_mode: CandidateDisplayMode::SideBySide,
                syllable_separator: SyllableSeparator::None,
                is_literal_roman_candidate_enabled: false,
                ..base.clone()
            },
        },
        SettingProfile {
            id: "roman_none".into(),
            name: "2. 拼音優先（無分隔 None，HagPinPlus 標準連寫）".into(),
            description: "客拼正宗 HagPinPlus 風格：首選輸出客拼，音節間完全無分隔連寫（如 fàdséu）".into(),
            settings: EngineSettings {
                is_hanji_first: false,
                candidate_display_mode: CandidateDisplayMode::SideBySide,
                syllable_separator: SyllableSeparator::None,
                is_literal_roman_candidate_enabled: false,
                ..base.clone()
            },
        },
        SettingProfile {
            id: "roman_hyphen".into(),
            name: "3. 拼音優先（連字號分隔 + 雙軌並列，教育部風格）".into(),
            description: "教育部詞典標準拼寫：首選輸出帶連字號之客拼（如 fàd-séu）".into(),
            settings: EngineSettings {
                is_hanji_first: false,
                candidate_display_mode: CandidateDisplayMode::SideBySide,
                syllable_separator: SyllableSeparator::Hyphen,
                is_literal_roman_candidate_enabled: false,
                ..base.clone()
            },
        },
        SettingProfile {
            id: "roman_space".into(),
            name: "4. 拼音優先（空格分隔 + 雙軌並列）".into(),
            description: "分音節空格習慣：首選輸出客拼，各音節以空格分隔（如 fàd séu）".into(),
            settings: EngineSettings {
                is_hanji_first: false,
                candidate_display_mode: CandidateDisplayMode::SideBySide,
                syllable_separator: SyllableSeparator::Space,
                is_literal_roman_candidate_enabled: false,
                ..base.clone()
            },
        },
        SettingProfile {
            id: "combined_ruby".into(),
            name: "5. 漢字帶音混排 (Combined / Ruby)".into(),
            description: "學習與教學模式：候選窗將漢字與標音融合成單一複合標籤呈現".into(),
            settings: EngineSettings {
                is_hanji_first: true,
                candidate_display_mode: CandidateDisplayMode::Combined,
                syllable_separator: SyllableSeparator::Hyphen,
                is_literal_roman_candidate_enabled: false,
                ..base.clone()
            },
        },
        SettingProfile {
            id: "literal_first".into(),
            name: "6. 原碼字串置頂 (Show Typed Text First)".into(),
            description: "混打外語模式：第 0 位強制保留英文原文，Space 或 Return 可直接送出英數".into(),
            settings: EngineSettings {
                is_hanji_first: true,
                candidate_display_mode: CandidateDisplayMode::SideBySide,
                syllable_separator: SyllableSeparator::Hyphen,
                is_literal_roman_candidate_enabled: true,
                ..base.clone()
            },
        },
    ]
}

fn create_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            id: "r2_13_phrase1".into(),
            name: "平哥實測句：想法".into(),
            input: "siongfad".into(),
            expected_meaning: "想法".into(),
        },
        TestCase {
            id: "r2_13_phrase2".into(),
            name: "平哥實測句：跈時行".into(),
            input: "tensihang".into(),
            expected_meaning: "跈時行 (趕流行)".into(),
        },
        TestCase {
            id: "r2_13_phrase3".into(),
            name: "平哥實測句：顛顛倒倒".into(),
            input: "diandiandodo".into(),
            expected_meaning: "顛顛倒倒".into(),
        },
        TestCase {
            id: "basic_pronoun".into(),
            name: "客語核心人稱代名詞：𠊎".into(),
            input: "ngai".into(),
            expected_meaning: "𠊎 (我)".into(),
        },
        TestCase {
            id: "basic_verb".into(),
            name: "核心生活詞彙：發燒".into(),
            input: "fadseu".into(),
            expected_meaning: "發燒".into(),
        },
        TestCase {
            id: "project_name".into(),
            name: "專案核心名詞：企盤".into(),
            input: "kipan".into(),
            expected_meaning: "企盤 (輸入法鍵盤)".into(),
        },
        TestCase {
            id: "food".into(),
            name: "日常生活詞彙：食飯".into(),
            input: "siidfan".into(),
            expected_meaning: "食飯 (吃飯)".into(),
        },
        TestCase {
            id: "abbrev_fadseu".into(),
            name: "首字母簡拼測試：fs".into(),
            input: "fs".into(),
            expected_meaning: "發燒 (簡拼)".into(),
        },
        TestCase {
            id: "abbrev_kipan".into(),
            name: "首字母簡拼測試：kp".into(),
            input: "kp".into(),
            expected_meaning: "企盤 (簡拼)".into(),
        },
    ]
}

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let dict_dir = repo_root.join("assets/dictionaries");

    println!("========================================================");
    println!("  客話企盤 (HakSpring-Kipan) — 全設定行為矩陣模擬器");
    println!("========================================================");
    println!("載入辭典: {}", dict_dir.display());

    let artifacts = DictionaryArtifacts::locate(&dict_dir).expect("詞庫檔案存在");
    let stats = engine::lexicon_install(&artifacts, 1).expect("詞庫成功載入引擎");
    println!("✓ 詞庫載入成功！總詞條筆數: {} 筆", stats.dictionary_record_count);

    let profiles = create_profiles();
    let test_cases = create_test_cases();

    println!("▶ 測試矩陣規模: {} 組設定 × {} 條測試語料 = {} 場次模擬",
        profiles.len(), test_cases.len(), profiles.len() * test_cases.len());

    let mut results: Vec<RunResult> = Vec::new();
    let mut gen: u64 = 5000;

    for prof in &profiles {
        println!("\n[測試設定] {}", prof.name);
        for tc in &test_cases {
            gen += 1;
            let mut steps = Vec::new();

            // 1. 逐鍵 append
            let mut final_transition = None;
            for ch in tc.input.chars() {
                let s = ch.to_string();
                let tr = engine::append(&s, &prof.settings, PLATFORM, gen).expect("append 成功");
                
                // 獲取當前候選清單
                let fetch = engine::fetch_at_pos(&prof.settings, PLATFORM, gen, 0)
                    .expect("fetch 成功");
                let cands = fetch.candidates.unwrap_or_default();
                let top: Vec<CandidateSnapshot> = cands.iter().take(5).enumerate().map(|(idx, c)| {
                    CandidateSnapshot {
                        slot: idx,
                        hanji: c.hanji.clone(),
                        roman: c.roman.clone(),
                        display_text: c.display_text.clone(),
                        canonical_tl: c.canonical_tl.clone(),
                        consumed_bytes: c.consumed_span_end,
                        syllable_count: c.syllable_count,
                    }
                }).collect();

                steps.push(StepSnapshot {
                    key: s,
                    preedit: tr.display_text.clone(),
                    candidate_count: cands.len(),
                    top_candidates: top,
                });

                final_transition = Some(tr);
            }

            // 2. 模擬確認選字 (首選 commit)
            let fetch = engine::fetch_at_pos(&prof.settings, PLATFORM, gen, 0).expect("fetch 成功");
            let cands = fetch.candidates.unwrap_or_default();
            
            let final_output = if let Some(first) = cands.first() {
                let commit_res = engine::commit_continuous(
                    &CommitContinuousArgs {
                        script: CommitScript::Lead,
                        roman: &first.roman,
                        canonical_text: &first.display_text,
                        association_tl: &first.canonical_tl,
                        hanji: first.hanji.as_deref(),
                        consumed_bytes: first.consumed_span_end,
                        syllable_count: first.syllable_count,
                    },
                    &prof.settings,
                    PLATFORM,
                    gen,
                );
                match commit_res {
                    Some(c) => c.commit.document_text,
                    None => "(Commit 無結果)".into(),
                }
            } else {
                "(無候選字)".into()
            };

            let notes = if final_output.contains(&tc.expected_meaning) || tc.expected_meaning.contains(&final_output) {
                "命中預期".into()
            } else {
                "非完全匹配".into()
            };

            print!("  • {:<16} => 送出: {:<12}", tc.input, final_output);
            if !steps.is_empty() {
                print!(" (首選候選詞: {})", steps.last().unwrap().top_candidates.first().map(|c| c.display_text.as_str()).unwrap_or("無"));
            }
            println!();

            results.push(RunResult {
                setting_id: prof.id.clone(),
                setting_name: prof.name.clone(),
                test_id: tc.id.clone(),
                corpus_text: tc.name.clone(),
                input_keys: tc.input.clone(),
                steps,
                final_output,
                notes,
            });

            engine::reset(gen);
        }
    }

    // 輸出 JSON
    let output_dir = repo_root.join("output");
    fs::create_dir_all(&output_dir).unwrap();
    let json_path = output_dir.join("matrix_simulation.json");
    let json_str = serde_json::to_string_pretty(&results).unwrap();
    fs::write(&json_path, &json_str).unwrap();
    println!("\n✓ 結構化數據已寫入: {}", json_path.display());

    // 生成 HTML 報告
    let html_content = generate_html_report(&profiles, &test_cases, &results);
    let docs_dir = repo_root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    let html_path = docs_dir.join("kipan_behavior_study.html");
    fs::write(&html_path, &html_content).unwrap();
    println!("🎉 互動式 HTML 整體研究報告已成功生成: {}", html_path.display());
}

fn generate_html_report(
    profiles: &[SettingProfile],
    test_cases: &[TestCase],
    results: &[RunResult],
) -> String {
    let mut rows_html = String::new();

    for tc in test_cases {
        rows_html.push_str(&format!(r#"
        <tr class="corpus-row">
            <td class="corpus-header" colspan="{}">
                <div class="corpus-title">
                    <span class="badge badge-accent">測試語料</span>
                    <strong>{}</strong> <code>{}</code>
                    <span class="corpus-meaning">（預期詞義：{}）</span>
                </div>
            </td>
        </tr>
        "#, profiles.len() + 1, tc.name, tc.input, tc.expected_meaning));

        rows_html.push_str("<tr>\n");
        rows_html.push_str(r#"<td class="cell-label">首位送出結果 (Commit Output)</td>"#);
        for prof in profiles {
            let res = results.iter().find(|r| r.test_id == tc.id && r.setting_id == prof.id);
            let out = res.map(|r| r.final_output.as_str()).unwrap_or("-");
            rows_html.push_str(&format!(
                r#"<td class="cell-result"><span class="output-pill">{}</span></td>"#,
                out
            ));
        }
        rows_html.push_str("</tr>\n");

        rows_html.push_str("<tr>\n");
        rows_html.push_str(r#"<td class="cell-label">最終候選字清單 (Top 3 Candidates)</td>"#);
        for prof in profiles {
            let res = results.iter().find(|r| r.test_id == tc.id && r.setting_id == prof.id);
            let cands_html = if let Some(r) = res {
                if let Some(last_step) = r.steps.last() {
                    let mut list = String::new();
                    for (i, c) in last_step.top_candidates.iter().take(3).enumerate() {
                        let hanji = c.hanji.as_deref().unwrap_or("");
                        list.push_str(&format!(
                            r#"<div class="cand-item"><span class="cand-num">{}.</span> <span class="cand-hanji">{}</span> <span class="cand-roman">{}</span></div>"#,
                            i + 1, hanji, c.display_text
                        ));
                    }
                    list
                } else {
                    "-".into()
                }
            } else {
                "-".into()
            };
            rows_html.push_str(&format!(r#"<td class="cell-cands">{}</td>"#, cands_html));
        }
        rows_html.push_str("</tr>\n");
    }

    let mut profile_headers = String::new();
    for prof in profiles {
        profile_headers.push_str(&format!(
            r#"<th><div class="prof-header"><strong>{}</strong><small>{}</small></div></th>"#,
            prof.name, prof.description
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>客話企盤 (HakSpring-Kipan) — 全設定行為矩陣深度研究報告</title>
    <style>
        :root {{
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-card: #334155;
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --accent: #38bdf8;
            --accent-glow: rgba(56, 189, 248, 0.15);
            --border: #475569;
            --success: #4ade80;
            --code-bg: #0b1120;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            padding: 2.5rem 1.5rem;
            line-height: 1.6;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        header {{
            margin-bottom: 2.5rem;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1.5rem;
        }}
        h1 {{
            font-size: 2.2rem;
            color: var(--accent);
            margin-bottom: 0.5rem;
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }}
        .subtitle {{
            color: var(--text-secondary);
            font-size: 1.05rem;
        }}
        .meta-cards {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 1rem;
            margin: 1.5rem 0 2.5rem 0;
        }}
        .card {{
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 10px;
            padding: 1.25rem;
        }}
        .card h3 {{
            font-size: 0.85rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            margin-bottom: 0.4rem;
        }}
        .card .value {{
            font-size: 1.6rem;
            font-weight: 700;
            color: var(--accent);
        }}
        .table-wrap {{
            overflow-x: auto;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.4);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 0.95rem;
            text-align: left;
        }}
        th, td {{
            padding: 1rem 1.2rem;
            border-bottom: 1px solid var(--border);
            vertical-align: top;
        }}
        th {{
            background: #111e33;
            color: var(--text-primary);
            position: sticky;
            top: 0;
            z-index: 10;
        }}
        .prof-header strong {{
            display: block;
            color: var(--accent);
            font-size: 1rem;
            margin-bottom: 0.25rem;
        }}
        .prof-header small {{
            color: var(--text-secondary);
            font-weight: normal;
            font-size: 0.8rem;
            line-height: 1.3;
            display: block;
        }}
        .corpus-row {{
            background: #1e3a5f !important;
        }}
        .corpus-header {{
            padding: 0.8rem 1.2rem;
            border-top: 2px solid var(--accent);
        }}
        .corpus-title {{
            display: flex;
            align-items: center;
            gap: 0.8rem;
            font-size: 1.1rem;
        }}
        .badge {{
            padding: 0.2rem 0.6rem;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }}
        .badge-accent {{
            background: var(--accent);
            color: #0f172a;
        }}
        .corpus-meaning {{
            color: #bae6fd;
            font-size: 0.9rem;
            font-weight: normal;
        }}
        code {{
            background: var(--code-bg);
            color: #38bdf8;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 0.9em;
            border: 1px solid #1e293b;
        }}
        .cell-label {{
            font-weight: 600;
            color: var(--text-secondary);
            width: 180px;
            background: rgba(15, 23, 42, 0.4);
        }}
        .output-pill {{
            display: inline-block;
            background: #064e3b;
            color: #34d399;
            padding: 0.35rem 0.75rem;
            border-radius: 6px;
            font-weight: 700;
            font-size: 1.05rem;
            border: 1px solid #059669;
        }}
        .cand-item {{
            margin-bottom: 0.35rem;
            font-size: 0.85rem;
        }}
        .cand-num {{ color: var(--text-secondary); margin-right: 0.2rem; }}
        .cand-hanji {{ color: #fbbf24; font-weight: 600; margin-right: 0.3rem; }}
        .cand-roman {{ color: var(--text-secondary); font-size: 0.8rem; }}
        footer {{
            margin-top: 3rem;
            text-align: center;
            color: var(--text-secondary);
            font-size: 0.85rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>客話企盤 (HakSpring-Kipan) 全設定行為矩陣深度研究報告</h1>
            <p class="subtitle">自動化模擬 Rust 引擎無頭執行：設定切換 × 客話語料 × 候選切分 × 送字形態對照全覽</p>
        </header>

        <div class="meta-cards">
            <div class="card">
                <h3>測試設定組合</h3>
                <div class="value">{} 組</div>
            </div>
            <div class="card">
                <h3>測試語料規模</h3>
                <div class="value">{} 條</div>
            </div>
            <div class="card">
                <h3>總計執行場次</h3>
                <div class="value">{} 場</div>
            </div>
            <div class="card">
                <h3>辭典詞庫版本</h3>
                <div class="value">22,294 筆</div>
            </div>
        </div>

        <div class="table-wrap">
            <table>
                <thead>
                    <tr>
                        <th style="width: 180px;">測試向度</th>
                        {}
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>

        <footer>
            <p>客源翠 HakSpring 團隊 — 客話企盤 (HakSpring-Kipan) 自動化架構研究工坊產出</p>
        </footer>
    </div>
</body>
</html>"#,
        profiles.len(),
        test_cases.len(),
        profiles.len() * test_cases.len(),
        profile_headers,
        rows_html
    )
}
