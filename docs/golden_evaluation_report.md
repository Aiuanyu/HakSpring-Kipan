# 客話企盤 (HakSpring-Kipan) 黃金語料標準答案回歸測試報告

- **測試時間**：2026-10-23
- **執行環境**：**【完整沙盒模式 (SQLite Active)】：已連線 SQLite，真實載入自訂詞與學習紀錄**

## 一、 核心評測量化指標總覽 (Executive Summary)

| 評測指標分流 | 題數規模 | 整句全對率 (Sentence Acc) | 標音全對率 (Roman Acc) | 字錯誤率 (CER) | 指標核心意義 |
|---|:---:|:---:|:---:|:---:|---|
| **純詞庫連打基準線** | 45 題 | **57.78%** (26/45) | **60.00%** (27/45) | **12.46%** (35/281) | 衡量通用詞庫、連打切分與排序的核心能力（排除個人環境干擾） |
| **個人自訂／學詞驗證集** | 5 題 | **60.00%** (3/5) | **40.00%** (2/5) | **12.50%** (5/40) | 驗證個人自訂詞組（瀏、瑋、江瑋平、劉源翠、歐陽秋月）是否被引擎成功採納 |
| **總體語料全集** | 50 題 | **58.00%** (29/50) | **58.00%** (29/50) | **12.46%** (40/321) | 30 句客觀回歸基準線總指標 |

## 二、 逐句標準對照評測表 (Expected vs. Actual)

### 1. 純詞庫通用連打標準集（25 題）

| # | 類別 | 語料說明 | 輸入碼 | 標準答案 (Expected) | 實際輸出 (Actual) | 結果 | CER | 標音比對 | 錯字分類與診斷 |
|---|---|---|---|---|---|:---:|---:|---|---|
| 1 | problem_case | 連打个出該問題（常用連詞个 vs 該） | `zuizuieuge` | 最主要个 | 注意主要个 | ❌ 有錯 | 50.0% | `zui zù-ieu ge` (吻合) | `斷詞切分 (Segmentation)`: 輸出多出贅字 '注'<br>`排序偏後 (Ranking)`: 預期字 '最' 已收錄於詞庫，且在非首選候選列中出現 |
| 2 | problem_case | 連詞摎 láu 被動詞落踩過之排序問題 | `ngailaugi` | 𠊎摎佢 | 𠊎摎佢 | ✅ 全對 | 0.0% | `ngǎi láu gǐ` (吻合) | — |
| 3 | historical_bug | R5-1 跨界連打壓制修復驗證句 | `iaheduioigefibo` | 這係對愛个回報 | 抑係在愛个會煲 | ❌ 有錯 | 57.1% | 預期: `Ià he dui oi ge fǐ-bo`<br>實際: `iahe dui oi ge fi bó` | `排序偏後 (Ranking)`: 預期字 '這' 已收錄於詞庫，且在非首選候選列中出現<br>`排序偏後 (Ranking)`: 預期字 '對' 已收錄於詞庫，首選輸出 '在' 權重較高或搶先結巴<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「回報」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「回報」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 4 | problem_case | 補字典學詞連打 | `busiidien` | 補字典 | 補字典 | ✅ 全對 | 0.0% | 預期: `bù-sii-dièn`<br>實際: `busiidien` | — |
| 5 | problem_case | 調羹仔名詞連打 | `tiaugange` | 調羹仔 | 調羹仔 | ✅ 全對 | 0.0% | `tiǎu-gáng-è` (吻合) | — |
| 6 | classical_text | 世界人權宣言第一條前段（正字个，非個人） | `soiugenginsangloiciiiu` | 所有个人生來自由 | 所有个人生來自由 | ✅ 全對 | 0.0% | `sò-iú ge ngǐn sáng-lǒi cii-iǔ` (吻合) | — |
| 7 | classical_text | 世界人權宣言第一條後段（連詞摎，非落） | `tezunngiamlaukianlisongidludpinden` | 在尊嚴摎權利上一律平等 | 在準拈摎權利上一㪐平等 | ❌ 有錯 | 27.3% | 預期: `té zún-ngiǎm láu kiǎn-li song ìd-lud pǐn-dèn`<br>實際: `té zùn ngiám láu kiǎnli song ìd lùd pǐndèn` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「尊嚴」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「尊嚴」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「一律」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 8 | classical_text | 現代客語聖經愛篇：有愛心就會有吞忍㧯慈悲 | `iuoiximqiuvoiutunngiunlauciibi` | 有愛心就會有吞忍摎慈悲 | 有愛心就和有吞銀摎慈悲 | ❌ 有錯 | 18.2% | 預期: `iú oi-xím qiu voi iú tún-ngiún láu cǐi-bí`<br>實際: `iú oixím qiu vǒ iú tún ngiǔn láu cǐibí` | `排序偏後 (Ranking)`: 預期字 '會' 已收錄於詞庫，首選輸出 '和' 權重較高或搶先結巴<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「吞忍」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 9 | classical_text | 現代客語聖經愛篇：有愛心就毋會嫉妒 | `iuoiximqiumvoiqiddu` | 有愛心就毋會嫉妒 | 有愛心就毋會七在 | ❌ 有錯 | 25.0% | `iú oi-xím qiu m̌-voi qìd-du` (吻合) | `缺詞（詞組 OOV-Phrase）`: 預期詞組「嫉妒」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「嫉妒」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 10 | classical_text | 現代客語聖經愛篇：毋會誇口毋會驕傲 | `mvoikuakieumvoigieungau` | 毋會誇口毋會驕傲 | 毋會誇口毋會噭咬 | ❌ 有錯 | 25.0% | 預期: `m̌-voi kuá-kièu m̌-voi giéu-ngau`<br>實際: `m̌voi kuákièu m̌voi gieu ngáu` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「驕傲」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「驕傲」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 11 | classical_text | 現代客語聖經愛篇：愛心永遠存在 | `oiximiuniencuncai` | 愛心永遠存在 | 愛心永遠伸在 | ❌ 有錯 | 16.7% | 預期: `oi-xím iùn-ièn cǔn-cai`<br>實際: `oixím iùn'ièn cún cai` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「存在」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 12 | daily_life | 日常生活問候：食飽無 | `siidbaumang` | 食飽吂 | 食飽吂 | ✅ 全對 | 0.0% | `siid-bàu-mǎng` (吻合) | — |
| 13 | daily_life | 日常生活句：轉屋下 | `zonvugka` | 轉屋下 | 轉屋下 | ✅ 全對 | 0.0% | `zòn-vùg-ká` (吻合) | — |
| 14 | daily_life | 客家待客句：請來食茶 | `qiangloisiidca` | 請來食茶 | 請來食茶 | ✅ 全對 | 0.0% | `qiàng lǒi siid-cǎ` (吻合) | — |
| 15 | daily_life | 日常生活問候：你愛去哪 | `ngioihinai` | 你愛去哪 | 你愛去哪 | ✅ 全對 | 0.0% | 預期: `ň oi hi nài`<br>實際: `ngǐ oihinai` | — |
| 16 | daily_life | 日常生活問候：恁多謝 | `anziise` | 恁仔細 | 恁仔細 | ✅ 全對 | 0.0% | `àn zìi-se` (吻合) | — |
| 17 | daily_life | 日常生活問候：恁早 | `anzo` | 恁早 | 恁做 | ❌ 有錯 | 50.0% | 預期: `àn-zò`<br>實際: `àn zo` | `排序偏後 (Ranking)`: 預期字 '早' 已收錄於詞庫，且在非首選候選列中出現 |
| 18 | daily_life | 日常生活句：打拚 | `dabiang` | 打拚 | 打拚 | ✅ 全對 | 0.0% | `dà-biang` (吻合) | — |
| 19 | daily_life | 日常生活句：去街仔 | `higiaie` | 去街仔 | 去界仔 | ❌ 有錯 | 33.3% | 預期: `hi giái-è`<br>實際: `hi giai è` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「街仔」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 20 | daily_life | 核心族語句：大家共下講客話 | `taigakiunghagonghagfa` | 大家共下講客話 | 大家共下講客話 | ✅ 全對 | 0.0% | `tai-gá kiung-ha gòng hàg-fa` (吻合) | — |
| 21 | idiom | 常用名詞：想法（四縣 xiongfab） | `xiongfab` | 想法 | 想法 | ✅ 全對 | 0.0% | `xiòng-fàb` (吻合) | — |
| 22 | idiom | 客語特色熟語：跈時行 | `tensiihang` | 跈時行 | 跈時行 | ✅ 全對 | 0.0% | `těn-sǐi-hǎng` (吻合) | — |
| 23 | idiom | 客語疊字熟語：顛顛倒倒 | `diendiendodo` | 顛顛倒倒 | 顛顛倒倒 | ✅ 全對 | 0.0% | `dién-dién-do-do` (吻合) | — |
| 24 | idiom | 客語經典熟語：跌斷腳骨顛倒勇 | `diedtongiogguddiendoiung` | 跌斷腳骨顛倒勇 | 跌斷腳骨顛倒勇 | ✅ 全對 | 0.0% | `dièd-tón-giòg-gùd-dién-do-iùng` (吻合) | — |
| 25 | problem_case | 使用者實測回報有問題：有更新吔 | `iugangxine` | 有更新吔 | 有耕信仔 | ❌ 有錯 | 75.0% | 預期: `iú gáng xín è`<br>實際: `iú gáng xin'è` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「更新」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「更新」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`排序偏後 (Ranking)`: 預期字 '吔' 已收錄於詞庫，首選輸出 '仔' 權重較高或搶先結巴 |
| 26 | daily_colloquial | 平哥審定日常口語：日常生活、天氣 | `gimbungidtiensiiiho` | 今晡日天時異好 | 今晡日天時異好 | ✅ 全對 | 0.0% | `Gímbú'ngìd tiénsǐi i hò` (吻合) | — |
| 27 | daily_colloquial | 平哥審定日常口語：口語時間詞「下晝」、問句「哪位」 | `nhazuoihinaivi` | 妳下晝愛去哪位 | 你下晝愛去哪位 | ❌ 有錯 | 14.3% | 預期: `Ň házu oi hi naivi`<br>實際: `ň házu oi hinaivi` | `排序偏後 (Ranking)`: 預期字 '妳' 已收錄於詞庫，且在非首選候選列中出現 |
| 28 | daily_colloquial | 平哥審定日常口語：現代科技詞「手機仔」、句尾虛詞「吔」 | `ngasugiemotiene` | 吾手機仔無電吔 | 吾手機仔無天仔 | ❌ 有錯 | 28.6% | 預期: `Ngá sùgí'è mǒ tien è`<br>實際: `ngá sùgí'è mǒ tién è` | `排序偏後 (Ranking)`: 預期字 '電' 已收錄於詞庫，首選輸出 '天' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '吔' 已收錄於詞庫，首選輸出 '仔' 權重較高或搶先結巴 |
| 29 | daily_colloquial | 平哥審定日常口語：疊字副詞「遽遽」、晚餐「暗晡夜」 | `giaggiagzonhisiidia` | 遽遽轉去食夜 | 遽遽轉去食夜 | ✅ 全對 | 0.0% | `Giàggiàg zònhi siid'ia` (吻合) | — |
| 30 | daily_colloquial | 平哥審定日常口語：軟體生活句、程度副詞「當」 | `kipandasiidongsunsu` | 企盤打字當順手 | 棋盤打字當順手 | ❌ 有錯 | 14.3% | 預期: `Kípǎn dàsii dóng sunsù`<br>實際: `kǐpǎn dàsii dóng sunsù` | `排序偏後 (Ranking)`: 預期字 '企' 已收錄於詞庫，且在非首選候選列中出現 |
| 31 | daily_colloquial | 平哥審定日常口語：時間詞「昨晡日」、天氣動詞「落大雨」 | `cobungidambulogtaii` | 昨晡日暗晡落大雨 | 昨晡日暗晡落大雨 | ✅ 全對 | 0.0% | `Cóbú'ngìd ambú logtai'ì` (吻合) | — |
| 32 | daily_colloquial | 平哥審定日常口語：示性詞「這隻」、形容詞「單純」 | `iakiensiiqinmoandansun` | 這件事情無恁單純 | 這件事情無恁單純 | ✅ 全對 | 0.0% | `Ià kien siiqǐn mǒ àn dánsǔn` (吻合) | — |
| 33 | daily_colloquial | 平哥審定日常口語：複數人稱「𠊎等」、連動句「想辦法」 | `ngaideukiunghaloixiongpanfab` | 𠊎兜共下來想辦法 | 𠊎兜共下來想辦法 | ✅ 全對 | 0.0% | `Ngǎidéu kiungha lǒi xiòng panfàb` (吻合) | — |
| 34 | daily_colloquial | 平哥審定日常口語：現代科技詞「電腦檔案」、「備份」 | `tiennodongonoigidedpifun` | 電腦檔案愛記得備份 | 電腦檔案愛記得皮分 | ❌ 有錯 | 22.2% | 預期: `Tiennò dòng'on oi gidèd pifun`<br>實際: `tiennò dòng'on oi gidèd pǐ fún` | `排序偏後 (Ranking)`: 預期字 '備' 已收錄於詞庫，首選輸出 '皮' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '份' 已收錄於詞庫，首選輸出 '分' 權重較高或搶先結巴 |
| 35 | daily_colloquial | 平哥審定日常口語：現代交通詞「高鐵」、比較句「過遽」 | `cogotiedbicocaehangogiag` | 坐高鐵比坐車仔還過遽 | 坐高鐵比坐車仔還過遽 | ✅ 全對 | 0.0% | `Có gótièd bì có cáè hǎn'go giàg` (吻合) | — |
| 36 | daily_colloquial | 平哥審定日常口語：常用名詞「新聞」、疑問句尾「無」 | `ngiiukondogetiauxinvunmo` | 你有看著該條新聞無 | 你有看著个條新聞無 | ❌ 有錯 | 11.1% | `Ngǐ iú kondò ge tiǎu xínvǔn mǒ` (吻合) | `同分決勝 (TieBreaking)`: 詞庫中「个」與「該」權重同分 (50000)，同分決勝規則偏向「該」 |
| 37 | daily_colloquial | 平哥審定日常口語：社會議題「假消息」、告誡語「千祈莫」 | `gaseuxidqienvanmogloncon` | 假消息千萬莫亂傳 | 加消息千萬莫亂賺 | ❌ 有錯 | 25.0% | 預期: `Gà séuxìd qiénvan mog lon cǒn`<br>實際: `gá séuxìd qiénvan mog lon con` | `排序偏後 (Ranking)`: 預期字 '假' 已收錄於詞庫，且在非首選候選列中出現<br>`排序偏後 (Ranking)`: 預期字 '傳' 已收錄於詞庫，首選輸出 '賺' 權重較高或搶先結巴 |
| 38 | daily_colloquial | 平哥審定日常口語：否定助詞「毋好」、動詞「相打爭論」 | `mhotungnginxiongdaxiongau` | 毋好同人相打相詏 | 毋好同人相打相詏 | ✅ 全對 | 0.0% | `M̌hò tǔng ngǐn xióngdà xióng'au` (吻合) | — |
| 39 | daily_colloquial | 平哥審定日常口語：問候語「屋下人」、健康「健康無」 | `vugkanginsiintiiukongkienmo` | 屋下人身體有康健無 | 屋下人身體有康健無 | ✅ 全對 | 0.0% | `Vùgká'ngǐn síintì iú kóngkien mǒ` (吻合) | — |
| 40 | daily_colloquial | 平哥審定日常口語：處置句「摎...」、科技動詞「儲存」 | `lauiabidziiliaucucunhiloi` | 摎這筆資料儲存起來 | 摎這筆資料儲存起來 | ✅ 全對 | 0.0% | `Láu ià bìd zíiliau cǔcǔn hìlǒi` (吻合) | — |
| 41 | daily_colloquial | 平哥審定日常口語：現代名詞「網路速度」、副詞「恁慢」 | `mionglusugtuanman` | 網路速度恁慢 | 網路速度恁慢 | ✅ 全對 | 0.0% | `Miònglu sùgtu àn man` (吻合) | — |
| 42 | daily_colloquial | 平哥審定日常口語：口語疑問詞「仰會」、程度詞「恁多」 | `ngiongvoiandomunti` | 仰會恁多問題 | 仰會恁到問題 | ❌ 有錯 | 16.7% | 預期: `Ngiòng voi àndó muntǐ`<br>實際: `ngiòngvoi àn do muntǐ` | `排序偏後 (Ranking)`: 預期字 '多' 已收錄於詞庫，首選輸出 '到' 權重較高或搶先結巴 |
| 43 | daily_colloquial | 平哥審定日常口語：時間詞「這下」、專有名詞「客源翠」 | `ngaiiahadihagngiencuikonsu` | 𠊎這下在客源翠看書 | 𠊎這下在客源翠看守 | ❌ 有錯 | 11.1% | 預期: `Ngǎi iàha di Hàgngiěncui kon sú`<br>實際: `ngǎi iàha di hàg ngiěn cui konsù` | `排序偏後 (Ranking)`: 預期字 '書' 已收錄於詞庫，首選輸出 '守' 權重較高或搶先結巴 |
| 44 | daily_colloquial | 平哥審定日常口語：功能詞「換字型」、「介面當靚」 | `vonsiihingoheugiemiendongjiang` | 換字型過後介面當靚 | 換事興過後雞面當靚 | ❌ 有錯 | 33.3% | 預期: `Von siihǐn goheu giemien dóng jiáng`<br>實際: `von sii hín goheu gié mien dóngjiáng` | `排序偏後 (Ranking)`: 預期字 '字' 已收錄於詞庫，首選輸出 '事' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '型' 已收錄於詞庫，首選輸出 '興' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '介' 已收錄於詞庫，首選輸出 '雞' 權重較高或搶先結巴 |
| 45 | daily_colloquial | 平哥審定日常口語：傳聞句「聽講」、時間詞「聽日」 | `tanggongtiengongngidvoilogsong` | 聽講天光日會落霜 | 聽講天光日會落霜 | ✅ 全對 | 0.0% | `Tánggòng tién'góngngìd voi logsóng` (吻合) | — |

### 2. 個人化自訂詞與學習詞驗證集（5 題）

| # | 類別 | 語料說明 | 輸入碼 | 標準答案 (Expected) | 實際輸出 (Actual) | 結果 | CER | 標音比對 | 自訂詞生效診斷與錯字分類 |
|---|---|---|---|---|---|:---:|---:|---|---|
| 1 | custom_word | 子區間自訂詞瀏 liǔ 參與連打合成瀏覽器 | `liulamhi` | 瀏覽器 | 瀏㜮去 | ❌ 有錯 | 66.7% | 預期: `liǔ-lam-hi`<br>實際: `liu lám hi` | `缺詞（詞組 OOV-Phrase）`: 預期詞組「瀏覽器」未收錄於詞庫，導致單字組詞潰散（需補字典）<br>`缺詞（詞組 OOV-Phrase）`: 預期詞組「瀏覽器」未收錄於詞庫，導致單字組詞潰散（需補字典） |
| 2 | personal_name | 江瑋平姓名介紹句（含自訂單字瑋 Ví 與學習詞江瑋平） | `ngagemiangeonzogongvipin` | 吾个名仔安做江瑋平 | 吾个名仔安做江瑋平 | ✅ 全對 | 0.0% | `Ngá ge miǎng-è ón-zo Góng Vípǐn` (吻合) | — |
| 3 | personal_name | 劉源翠姓名介紹句 | `ngagemiangeonzoliungiencui` | 吾个名仔安做劉源翠 | 吾个名仔安做劉源翠 | ✅ 全對 | 0.0% | `Ngá ge miǎng-è ón-zo Liǔ Ngiěncui` (吻合) | — |
| 4 | personal_name | 歐陽秋月姓名介紹句（測試四字複姓學詞連打） | `ngabungionggemiangeonzoeuiongqiungied` | 吾餔娘个名仔安做歐陽秋月 | 吾餔娘个名仔安做歐陽秋月 | ✅ 全對 | 0.0% | 預期: `Ngá bú-ngiǒng ge miǎng-è ón-zo Éu-iǒng Qiú-ngied`<br>實際: `ngábú'ngiǒng ge miǎng'è ónzo euiongqiungied` | — |
| 5 | custom_word | 補字典沈 Sìim 參與連打 | `ngigesiizongsiimbagiong` | 你个市長沈伯洋 | 你个市長審百樣 | ❌ 有錯 | 42.9% | 預期: `Ngǐ ge sii-zòng Sìim Bàg'iǒng`<br>實際: `ngǐ ge siizòng sìim bàg'iong` | `排序偏後 (Ranking)`: 預期字 '沈' 已收錄於詞庫，首選輸出 '審' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '伯' 已收錄於詞庫，首選輸出 '百' 權重較高或搶先結巴<br>`排序偏後 (Ranking)`: 預期字 '洋' 已收錄於詞庫，首選輸出 '樣' 權重較高或搶先結巴 |

## 三、 錯字根本原因分類統計 (Root Cause Analysis)

| 錯誤類型 | 出現頻次 | 改善方向與對應 Queue |
|---|:---:|---|
| 缺詞（詞組 OOV-Phrase） | 10 個詞組 | 客源翠補字典增補多音節詞組（缺失詞組共 10 個：一律、吞忍、回報、嫉妒、存在、尊嚴、更新、瀏覽器、街仔、驕傲） |
| 同分決勝 (TieBreaking) | 1 次 | 同分 tie-breaking 規則重構（Queue 4），「个」與「該」權重同分時偏好正字「个」 |
| 排序偏後 (Ranking) | 22 次 | 連打排序與權重調校（Queue 4），加權高頻連詞或接續詞（如「摎」、「請」） |
| 斷詞切分 (Segmentation) | 1 次 | Lattice 斷詞邊界判定與貪婪切分策略調校（如 `higiaie` 誤切贅字） |
