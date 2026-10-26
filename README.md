# 客源翠企盤 HakSpring-Kipan

追求強大功能个客話輸入法（Hakfa Input Method for macOS and more to come<!--,Windows, Linux, iOS, Android-->）

![macOS 14+](https://img.shields.io/badge/macOS-14%2B-lightgrey)
<!--![Windows 10+](https://img.shields.io/badge/Windows-10%2B-blue)
![Linux Fcitx5 | IBus](https://img.shields.io/badge/Linux-Fcitx5%20%7C%20IBus-orange)
![iOS 17+](https://img.shields.io/badge/iOS-17%2B-blue)
![Android 9+](https://img.shields.io/badge/Android-9%2B-green)-->
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue)](LICENSE)

---

## 簡介

「客源翠企盤」源於強大的開放原始碼專案「[台語齒盤 TaigiKeyboard](https://github.com/taigikeyboard/taigikeyboard)」截至 2026 年 10 月已開發完成多功能核心引擎及 5 大平台實做，由臺澎族語復興倡議者「Aiuanyu 愛灣語」專為臺灣客語（四縣、海陸等各腔）與拼音體系延伸調整，目標為善用全平台開發先行者的開放心血，達成全平台原生客語輸入法。

本專案旨在為各作業系統提供新的原生現代客話輸入法選擇，提供高雅流暢的無調連打、無分隔連寫美學、自訂創詞學習與全離線安全保障。

---

## 功能說明

### 一、繼承「台語齒盤」核心及各平台實做優勢

- **單一 Rust 核心引擎**：核心音韻、狀態機、詞庫檢索與排序皆由底層高效 Rust 實作，各平台外殼極致輕量原生
- **多音節無縫連續輸入**：支援整句連打，透過 Lattice 與 Viterbi 動態規劃自動切分音節與詞界
- **諸夏字與羅馬字混合輸出**：可隨心輸出諸夏字（漢字）、客語羅馬字或夏羅混寫
- **離線優先與隱私安全**：完全於本機端執行計算與字典檢索，零雲端上傳、不追蹤任何使用者敲鍵隱私
- **本地學習與自訂詞庫**：支援本機自訂詞庫（Custom Dictionary）與手動創詞記憶（Learned Phrases）
    - 手動創詞記憶支援 **hyphen 連接**格式，從常用連打片語升級單詞
- **跨 5 大作業系統原生外殼與 GUI**：直接繼承涵蓋桌面（macOS、Windows、Linux）與行動端（iOS、Android）之原生輸入法外殼與設定面板實作，奠定全平台展開之基石

### 二、適應客話之必要改造

- **客話 HagPinPlus 聲韻調體系**：全新建置 `hakfa_tables.rs` 客話音韻模組，完整支援客家語聲韻配合與入聲塞音判斷 `(v0.0.1)`
- **客話細音顎化聲母相容**：原生支援 `j, q, x` 等客話專屬細音顎化聲母輸入 `(v0.0.1)`
- **選字鍵改造為數字鍵 `1..9`**：調整原版字母選字鍵，適應客拼 `d, f, v, z, x, q` 等聲母 `(v0.0.1)`
- **預設音節「無分隔 (None)」**：呈現 HagPinPlus 自然無分隔連寫風格（如 `fadseu` 輸出 `fàdséu`、`Toivan` 輸出 `Tǒivǎn`） `(v0.0.1)`
- **詞庫調值清理**：清除教典詞庫中殘留的調值數字，根除數字選字鍵衝突 `(v0.0.1)`
- **原生「客」字專屬圖示**：macOS 選單列採用符合官方 Negative Alpha 規範之黑白自適應「客」字鍵帽圖示，於 v0.0.2 全面升級為日系溫潤圓體 `ZenMaruGothic-Bold`（700），並配套產出 46 個多尺寸 App 圖示 `(v0.0.2)`
- **客源翠詞庫管線 (HakSpring Lexicon Pipeline)**：打造自動化抽取與轉檔工具（`extract_and_build.js`），整合四縣腔認證詞彙與教育部辭典，產出 2.2 萬筆標準化二進位詞庫 `(v0.0.1)`
- **套件識別碼設定 (Bundle ID)**：依愛灣語官方網域規範調整為 `net.aiuanyu.inputmethod.HakSpring-Kipan` `(v0.0.2)`
- **手動創詞相容無間隔連寫**：為使自學詞彙深度相容 HagPinPlus 無間隔連詞樣式，確立核心之連字號（hyphen）連接創詞規格（如 `cug-ngi`），確保連打時能精準識別並升級為獨立詞條 `(v0.0.2)`

### 三、擬貢獻回「台語齒盤」核心的新設計

- **手動創詞支援簡拼查出 (Abbreviation Lookup)**：修復 SQLite 查詢語法漏洞（`k.form IN (?, 'abbrev')`），讓手動學習短語（`learned_phrases`）完整支援聲母簡拼縮寫索引，敲 `cng` 即可秒選自創詞「族語」 `(v0.0.2)`

---

## 使用說明（打字與聲調指南）

客源翠企盤採用「**純數字鍵擇字、修飾鍵打調**」設計：常態下純數字鍵 `1～9` 保留給候選字擇字，打調時按住 **`⌃`（Control）＋數字** 即時成調。

可在設定選單中選擇以下聲調方案（詳見 [`docs/hakfa/tone-input.md`](docs/hakfa/tone-input.md)）：

### 1. 符號編號（預設推薦）
直接對應 HagPinPlus 調符字形，跨腔通用、直觀易記：
- `⌃2` → `á`（尖音 / acute）
- `⌃3` → `à`（低音 / grave）
- `⌃5` → `â`（折音 / circumflex）
- `⌃6` → `ǎ`（抑音 / caron）
- `⌃7` → `ā`（平音 / macron）
- **操作要訣**：無標符號不按調號；輸入調號後若要改調，直接敲另一組 `⌃+數字` 即可就地替換；按 `Backspace` 即可清除調號。
- **範例**：海陸打 `sin` 後按 `⌃3` 輸出 `sìn`，候選首選即為「新」、「先」、「薪」。

### 2. 方案一：五度調值
以客語實際調值（五度音高）輸入，由系統依當前腔調動態換算：
- **舒聲**：連續按下兩位數即成調
    - 四縣／南四縣：`⌃2` `⌃4` (24) → `á`、`⌃3` `⌃1` (31) → `à`、`⌃1` `⌃1` (11) → `ǎ`、`⌃5` `⌃5` (55) 清調
    - 海陸：`⌃3` `⌃3` (33) → `ā`、`⌃5` `⌃3` (53) → `à`、`⌃2` `⌃4` (24) → `á`、`⌃1` `⌃1` (11) → `ǎ`、`⌃5` `⌃5` (55) 清調
- **入聲**（韻尾 `b, d, g`）：單鍵一鍵成調（`⌃2` 陰入、`⌃5` 陽入）
- **範例**：海陸打 `sii` 連敲 `⌃3` `⌃3` 即為 `sīi`，候選首選即為「事」、「斯」、「字」。

---

## 建置與安裝 (macOS 先行)

詳細建置指南請參閱 [`docs/BUILDING.md`](docs/BUILDING.md)。

### 快速開始 (macOS)

1. 確認已安裝 Xcode Command Line Tools 與完整 Xcode。
2. 編譯原生 XCFramework 與相依套件：
   ```bash
   make macos-libs
   ```
3. 打包並安裝至系統：
   ```bash
   make -C macos install
   ```
4. 重新載入輸入法：
   ```bash
   make -C macos reload
   ```
   （若選單列圖示未即時更新，可執行 `killall TextInputMenuAgent` 刷新快取）。

---

## 授權條款 (License)

- 本專案程式碼依據 [Apache License 2.0](LICENSE) 開源授權釋出。
- 客話詞彙語料版權歸原製作者及教育部、客家委員會所有，依各該政府資料開放授權規範使用。
