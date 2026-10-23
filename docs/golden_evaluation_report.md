# 客話企盤 (HakSpring-Kipan) 黃金語料標準答案回歸測試報告

- **測試時間**：2026-10-23
- **執行環境**：**【完整沙盒模式 (SQLite Active)】：已連線 SQLite，真實載入自訂詞與學習紀錄**

## 一、 核心評測量化指標總覽 (Executive Summary)

| 評測指標分流 | 題數規模 | 整句全對率 (Sentence Acc) | 標音全對率 (Roman Acc) | 字錯誤率 (CER) | 指標核心意義 |
|---|:---:|:---:|:---:|:---:|---|
| **純詞庫連打基準線** | 25 題 | **52.00%** (13/25) | **52.00%** (13/25) | **21.14%** (26/123) | 衡量通用詞庫、連打切分與排序的核心能力（排除個人環境干擾） |
| **個人自訂／學詞驗證集** | 5 題 | **0.00%** (0/5) | **40.00%** (2/5) | **20.00%** (8/40) | 驗證個人自訂詞組（瀏、瑋、江瑋平、劉源翠、歐陽秋月）是否被引擎成功採納 |
| **總體語料全集** | 30 題 | **43.33%** (13/30) | **50.00%** (15/30) | **20.86%** (34/163) | 30 句客觀回歸基準線總指標 |

## 二、 逐句標準對照評測表 (Expected vs. Actual)

### 1. 純詞庫通用連打標準集（25 題）

| # | 類別 | 語料說明 | 輸入碼 | 標準答案 (Expected) | 實際輸出 (Actual) | 結果 | CER | 標音比對 | 錯字分類與診斷 |
|---|---|---|---|---|---|:---:|---:|---|---|
| 1 | problem_case | 連打个出該問題（常用連詞个 vs 該） | `zuizuieuge` | 最主要个 | 最主要該 | ❌ 有錯 | 25.0% | 預期: `zui zù-ieu ge`<br>實際: `zùi zù'ieu ge` | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」 |
| 2 | problem_case | 連詞摎 láu 被動詞落踩過之排序問題 | `ngailaugi` | 𠊎摎佢 | 𠊎落佢 | ❌ 有錯 | 33.3% | 預期: `ngǎi láu gǐ`<br>實際: `ngǎi lau gǐ` | `排序偏後 (Ranking)`: 預期字 '摎' 已收錄於詞庫，首選輸出 '落' 權重較高或搶先結巴 |
| 3 | historical_bug | R5-1 跨界連打壓制修復驗證句 | `iaheduioigefibo` | 這係對愛个回報 | 也係隊愛該回煲 | ❌ 有錯 | 57.1% | 預期: `Ià he dui oi ge fǐ-bo`<br>實際: `iahe dui oi ge fǐ bó` | `排序偏後 (Ranking)`: 預期字 '這' 已收錄於詞庫，且在非首選候選列中出現<br>`排序偏後 (Ranking)`: 預期字 '對' 已收錄於詞庫，首選輸出 '隊' 權重較高或搶先結巴<br>`同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「回報」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 4 | problem_case | 補字典學詞連打 | `busiidien` | 補字典 | 補字典 | ✅ 全對 | 0.0% | 預期: `bù-sii-dièn`<br>實際: `busiidien` | — |
| 5 | problem_case | 調羹仔名詞連打 | `tiaugange` | 調羹仔 | 調羹仔 | ✅ 全對 | 0.0% | `tiǎu-gáng-è` (吻合) | — |
| 6 | classical_text | 世界人權宣言第一條前段（正字个，非個人） | `soiugenginsangloiciiiu` | 所有个人生來自由 | 所有個人生來自由 | ❌ 有錯 | 12.5% | `sò-iú ge ngǐn sáng-lǒi cii-iǔ` (吻合) | `字形規範 (Orthography)`: 客文正字應為「个」，輸出包含非規範字「個」 |
| 7 | classical_text | 世界人權宣言第一條後段（連詞摎，非落） | `tezunngiamlaukianlisongidludpinden` | 在尊嚴摎權利上一律平等 | 在尊拈落權利上一㪐平等 | ❌ 有錯 | 27.3% | 預期: `té zún-ngiǎm láu kiǎn-li song ìd-lud pǐn-dèn`<br>實際: `té zún ngiám lau kiǎnli song ìd lùd pǐndèn` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「尊嚴」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`排序偏後 (Ranking)`: 預期字 '摎' 已收錄於詞庫，首選輸出 '落' 權重較高或搶先結巴<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「一律」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 8 | classical_text | 現代客語聖經愛篇：有愛心就會有吞忍㧯慈悲 | `iuoiximqiuvoiutunngiunlauciibi` | 有愛心就會有吞忍摎慈悲 | 有愛心就唩有吞銀落慈悲 | ❌ 有錯 | 27.3% | 預期: `iú oi-xím qiu voi iú tún-ngiún láu cǐi-bí`<br>實際: `iú oixím qiu vò iú tún ngiǔn lau cǐibí` | `排序偏後 (Ranking)`: 預期字 '會' 已收錄於詞庫，首選輸出 '唩' 權重較高或搶先結巴<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「吞忍」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`排序偏後 (Ranking)`: 預期字 '摎' 已收錄於詞庫，首選輸出 '落' 權重較高或搶先結巴 |
| 9 | classical_text | 現代客語聖經愛篇：有愛心就毋會嫉妒 | `iuoiximqiumvoiqiddu` | 有愛心就毋會嫉妒 | 有愛心就毋會七在 | ❌ 有錯 | 25.0% | `iú oi-xím qiu m̌-voi qìd-du` (吻合) | `缺詞（詞組 OOV-Phrase）`: 預期詞組「嫉妒」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「嫉妒」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 10 | classical_text | 現代客語聖經愛篇：毋會誇口毋會驕傲 | `mvoikuakieumvoigieungau` | 毋會誇口毋會驕傲 | 毋會誇口毋會噭咬 | ❌ 有錯 | 25.0% | 預期: `m̌-voi kuá-kièu m̌-voi giéu-ngau`<br>實際: `m̌voi kuákièu m̌voi gieu ngáu` | `缺詞（單字 OOV-Char）`: 詞庫真查確認無收錄單字 '驕'<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「驕傲」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 11 | classical_text | 現代客語聖經愛篇：愛心永遠存在 | `oiximiuniencuncai` | 愛心永遠存在 | 愛心永遠伸在 | ❌ 有錯 | 16.7% | 預期: `oi-xím iùn-ièn cǔn-cai`<br>實際: `oixím iùn'ièn cún cai` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「存在」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 12 | daily_life | 日常生活問候：食飽無 | `siidbaumang` | 食飽吂 | 食飽吂 | ✅ 全對 | 0.0% | `siid-bàu-mǎng` (吻合) | — |
| 13 | daily_life | 日常生活句：轉屋下 | `zonvugka` | 轉屋下 | 轉屋下 | ✅ 全對 | 0.0% | `zòn-vùg-ká` (吻合) | — |
| 14 | daily_life | 客家待客句：請來食茶 | `qiangloisiidca` | 請來食茶 | 青來食茶 | ❌ 有錯 | 25.0% | 預期: `qiàng lǒi siid-cǎ`<br>實際: `qiáng lǒi siidcǎ` | `排序偏後 (Ranking)`: 預期字 '請' 已收錄於詞庫，且在非首選候選列中出現 |
| 15 | daily_life | 日常生活問候：你愛去哪 | `ngioihinai` | 你愛去哪 | 你愛去哪 | ✅ 全對 | 0.0% | 預期: `ň oi hi nài`<br>實際: `ngǐ oihinai` | — |
| 16 | daily_life | 日常生活問候：恁多謝 | `anziise` | 恁仔細 | 恁仔細 | ✅ 全對 | 0.0% | `àn zìi-se` (吻合) | — |
| 17 | daily_life | 日常生活問候：恁早 | `anzo` | 恁早 | 恁早 | ✅ 全對 | 0.0% | `àn-zò` (吻合) | — |
| 18 | daily_life | 日常生活句：打拚 | `dabiang` | 打拚 | 打拚 | ✅ 全對 | 0.0% | `dà-biang` (吻合) | — |
| 19 | daily_life | 日常生活句：去街仔 | `higiaie` | 去街仔 | 起居阿姨仔 | ❌ 有錯 | 133.3% | 預期: `hi giái-è`<br>實際: `hìgí á'ǐ'è` | `斷詞切分 (Segmentation)`: 輸出多出贅字 '起'<br>`斷詞切分 (Segmentation)`: 輸出多出贅字 '居'<br>`排序偏後 (Ranking)`: 預期字 '去' 已收錄於詞庫，且在非首選候選列中出現<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「街仔」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 20 | daily_life | 核心族語句：大家共下講客話 | `taigakiunghagonghagfa` | 大家共下講客話 | 大家共下講客話 | ✅ 全對 | 0.0% | `tai-gá kiung-ha gòng hàg-fa` (吻合) | — |
| 21 | idiom | 常用名詞：想法（四縣 xiongfab） | `xiongfab` | 想法 | 想法 | ✅ 全對 | 0.0% | `xiòng-fàb` (吻合) | — |
| 22 | idiom | 客語特色熟語：跈時行 | `tensiihang` | 跈時行 | 跈時行 | ✅ 全對 | 0.0% | `těn-sǐi-hǎng` (吻合) | — |
| 23 | idiom | 客語疊字熟語：顛顛倒倒 | `diendiendodo` | 顛顛倒倒 | 顛顛倒倒 | ✅ 全對 | 0.0% | `dién-dién-do-do` (吻合) | — |
| 24 | idiom | 客語經典熟語：跌斷腳骨顛倒勇 | `diedtongiogguddiendoiung` | 跌斷腳骨顛倒勇 | 跌斷腳骨顛倒勇 | ✅ 全對 | 0.0% | `dièd-tón-giòg-gùd-dién-do-iùng` (吻合) | — |
| 25 | problem_case | 平哥 20261023 回報有問題：有更新吔 | `iugangxine` | 有更新吔 | 有粳信仔 | ❌ 有錯 | 75.0% | 預期: `iú gáng xín è`<br>實際: `iú gáng xin'è` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「更新」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「更新」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`排序偏後 (Ranking)`: 預期字 '吔' 已收錄於詞庫，首選輸出 '仔' 權重較高或搶先結巴 |

### 2. 個人化自訂詞與學習詞驗證集（5 題）

| # | 類別 | 語料說明 | 輸入碼 | 標準答案 (Expected) | 實際輸出 (Actual) | 結果 | CER | 標音比對 | 自訂詞生效診斷與錯字分類 |
|---|---|---|---|---|---|:---:|---:|---|---|
| 1 | custom_word | 子區間自訂詞瀏 liǔ 參與連打合成瀏覽器 | `liulamhi` | 瀏覽器 | 瀏㜮去 | ❌ 有錯 | 66.7% | 預期: `liǔ-lam-hi`<br>實際: `liu lám hi` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「瀏覽器」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「瀏覽器」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 2 | personal_name | 平哥姓名介紹句（含自訂單字瑋 Ví 與學習詞江瑋平） | `ngagemiangeonzogongvipin` | 吾个名仔安做江瑋平 | 吾該名仔安做江瑋平 | ❌ 有錯 | 11.1% | `Ngá ge miǎng-è ón-zo Góng Vípǐn` (吻合) | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」 |
| 3 | personal_name | 劉源翠姓名介紹句 | `ngagemiangeonzoliungiencui` | 吾个名仔安做劉源翠 | 吾該名仔安做劉源翠 | ❌ 有錯 | 11.1% | `Ngá ge miǎng-è ón-zo Liǔ Ngiěncui` (吻合) | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」 |
| 4 | personal_name | 歐陽秋月姓名介紹句（測試四字複姓學詞連打） | `ngabungionggemiangeonzoeuiongqiungied` | 吾餔娘个名仔安做歐陽秋月 | 吾餔娘該名仔安做歐陽秋月 | ❌ 有錯 | 8.3% | 預期: `Ngá bú-ngiǒng ge miǎng-è ón-zo Éu-iǒng Qiú-ngied`<br>實際: `ngábú'ngiǒng ge miǎng'è ónzo euiongqiungied` | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」 |
| 5 | custom_word | 補字典沈 Sìim 參與連打 | `ngigesiizongsiimbagiong` | 你个市長沈伯洋 | 你該市長沈百樣 | ❌ 有錯 | 42.9% | 預期: `Ngǐ ge sii-zòng Sìim Bàg'iǒng`<br>實際: `ngǐ ge siizòng sìim bàg'iong` | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」<br>`排序偏後 (Ranking)`: 預期字 '伯' 已收錄於詞庫，首選輸出 '百' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '洋' 已收錄於詞庫，首選輸出 '樣' 權重較高或搶先結巴 |

## 三、 錯字根本原因分類統計 (Root Cause Analysis)

| 錯誤類型 | 出現頻次 | 改善方向與對應 Queue |
|---|:---:|---|
| 排序偏後 (Ranking) | 11 次 | 連打排序與權重調校（Queue 4），加權高頻連詞或接續詞（如「摎」、「請」） |
| 斷詞切分 (Segmentation) | 2 次 | Lattice 斷詞邊界判定與貪婪切分策略調校（如 `higiaie` 誤切贅字） |
| 同分決勝 (TieBreaking) | 6 次 | 同分 tie-breaking 規則重構（Queue 4），「个」與「該」權重同分時偏好正字「个」 |
| 字形規範 (Orthography) | 1 次 | 客文正字規則校正，避免非規範字或異體字侵蝕（如「個」侵蝕「个」） |
| 缺詞（單字 OOV-Char） | 1 次 | 客源翠補字典增補單字（如「驕」等單字條目） |
| 缺詞（詞組 OOV-Phrase） | 10 個詞組 | 客源翠補字典增補多音節詞組（缺失詞組共 10 個：一律、吞忍、回報、嫉妒、存在、尊嚴、更新、瀏覽器、街仔、驕傲） |
