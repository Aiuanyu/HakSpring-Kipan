#!/usr/bin/env python3
"""
Translates in-app content (i18n/content/features.json and i18n/content/faq.json)
'hanji' field to Hakfa (Hakka), and syncs to Android and iOS bundle locations.
"""

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

FAQ_UPDATES = {
    "installIssue": {
        "title": "企盤安裝好之後無出現",
        "paragraphs": [
            {
                "text": "1. 先檢查企盤敢有照「啟用方法」設定好。",
                "attachment_text": "點這位看「啟用方法」",
            },
            {
                "text": "2. 重開你當在用个 App，分 App 重新捉著企盤。",
            },
            {
                "text": "3. 在做得打字个位所，長撳地球圖示就做得切換去客源翠企盤。",
            },
        ],
    },
    "feedback": {
        "title": "回報問題無消息",
        "paragraphs": [
            {
                "text": "若回報之後還吂收到回覆，請過送 1 擺，還係直接聯絡𠊎兜。",
                "attachment_text": "點這位看「關於」",
            },
        ],
    },
    "samsungSwitch": {
        "title": "三星手機企盤切換",
        "paragraphs": [
            {
                "text": "若三星企盤左手片下背無地球撳鈕，做得摎麥克風撳鈕改做企盤切換撳鈕。",
            },
            {
                "text": "1. 尋著三星企盤左手片下背个麥克風撳鈕，佢預設係語音輸入。",
            },
            {
                "text": "2. 長撳麥克風撳鈕，摎佢改做「輸入法」。",
            },
            {
                "text": "3. 撳鈕會變做企盤圖示，過撳一擺就做得切換企盤。",
            },
        ],
    },
}

FEATURES_UPDATES = {
    "inputModes": {
        "title": "輸入系統",
        "paragraphs": [
            {
                "text": "用企盤頂背个工具列切換客語拼音（HagPinPlus）摎英文模式。",
            },
            {
                "text": "客語拼音方案包含教育部臺灣客語拼音方案（HagPinPlus），無論係四縣、南四縣、海陸、大埔、饒平抑係詔安腔，全部做得順暢打字。",
            },
            {
                "text": "客源翠企盤未來乜會陸續加入客語白話字（PFS）等更多書寫系統支援。",
            },
            {
                "text": "若想愛打英文，工具列頂背乜有英文模式。",
            },
        ],
    },
    "romanInput": {
        "title": "6 種打字方式",
        "paragraphs": [
            {
                "text": "聲調毋使打，音節乜做得打一半，抑係單淨打頭字母。hag2fa55、hag2fa、hagfa、hagf、hf 全部尋得著客話。",
            },
            {
                "text": "規句共下打乜做得，企盤會自家斷音節。打 ngaioisiidfan 會尋著「𠊎愛食飯」。",
            },
            {
                "text": "想愛自家斷音節，就打連字符。像 zui 會尋著「最」摎「追」，打做 zu-i 正會尋著「注意」摎「主意」。",
            },
            {
                "text": "打好字母正打聲調數字，企盤會換做聲調符號。聲調打毋著，按一下刪除鍵過打一擺就做得。",
            },
            {
                "text": "支援數字調號 1 2 3 4 5 6 7 與五度調值等多元調號輸入方案。",
            },
        ],
    },
    "tpsAutoCorrect": {
        "title": "方音符號自動校正",
        "paragraphs": [
            {
                "text": "打方音符號个時節，企盤會自動修正符號。",
            },
            {
                "text": "齒音在 ㄧ 抑係 ㆪ 頭前會變顎化音，ㄗ、ㄘ、ㄙ、ㆡ 會變做 ㄐ、ㄑ、ㄒ、ㆢ。像打 ㄗㄧㄨ 會變做 ㄐㄧㄨ（就）。",
            },
            {
                "text": "ㄇ、ㄋ、ㄫ 摎 ㄅ、ㄉ、ㄍ、ㄏ 若擺在音節尾，會自家變做韻尾 ㆬ、ㄣ、ㆭ 摎 ㆴ、ㆵ、ㆻ、ㆷ。像打 ㄙㄧㄇ 會變做 ㄒㄧㆬ（心）。",
            },
            {
                "text": "ㄧ 後背打 ㆮ 會換做 ㆯ。",
            },
        ],
    },
    "hanloDesign": {
        "title": "羅馬字轉換器",
        "paragraphs": [
            {
                "text": "空白鍵右邊个轉換鍵，做得切換漢字模式摎羅馬字模式。預設係漢字模式，選候選詞會送出漢字。",
            },
            {
                "text": "標點乜會跈等換，漢字模式用全形，羅馬字模式用半形。",
            },
            {
                "text": "這係客源翠企盤為母語打字者精心設計个雙書寫切換機制，分打漢字摎羅馬字都當順手。",
            },
        ],
    },
    "caseSwitch": {
        "title": "自動大寫",
        "summary": "「自動大寫」決定一句个頭一个字母敢會自動變大寫。連續按 Shift 兩下係 Caps Lock。",
        "paragraphs": [
            {
                "text": "打開「自動大寫」，一句个頭一个字母會自動變大寫，Shift 鍵會變色反白。",
            },
            {
                "text": "關起來就全部打細寫，想愛大寫就自家按 Shift。",
            },
            {
                "text": "連續按 Shift 兩下係 Caps Lock，會一直打大寫，過按一下就解除。",
            },
            {
                "text": "大寫狀態下，調符摎字母輸入依然保持規則一致。",
            },
        ],
    },
    "oneHanded": {
        "title": "單手模式",
        "paragraphs": [
            {
                "text": "長按工具列个企盤圖示，選左手抑係右手，企盤就會移去該邊。",
            },
            {
                "text": "邊脣个按鈕做得「換邊」，抑係轉去「一般」。",
            },
        ],
    },
    "displayModes": {
        "title": "3 種候選詞顯示",
        "paragraphs": [
            {
                "text": "在「設定」>「候選詞顯示」選候選詞愛仰般顯示。",
            },
            {
                "text": "漢羅對應：漢字摎羅馬字共下顯示，用轉換鍵決定送出哪一个。",
            },
            {
                "text": "漢羅合用：漢字摎羅馬字分做兩格，選哪一格就送出哪一種。",
            },
            {
                "text": "羅馬字：單淨顯示羅馬字，送出羅馬字。",
            },
            {
                "text": "漢羅合用摎羅馬字模式支援客語各腔拼音顯示。",
            },
        ],
    },
    "typingSettings": {
        "title": "打字設定",
        "summary": "「雙書寫標註」摎漢字摎羅馬字共下送出。「自動加入空白」送出羅馬字之後自動加空格。",
        "paragraphs": [
            {
                "text": "打開「當在打个字放第一格」，候選詞頭一格就係你打个羅馬字，按一下就送出。",
            },
            {
                "text": "有兜詞辭典無收、尋無候選詞个時節，這格做得直接送出羅馬字。",
            },
            {
                "text": "關起來个時節，乜做得按 Enter 送出打个羅馬字。",
            },
            {
                "text": "打開「雙書寫標註」，漢字摎羅馬字會共下送出，像客話 (Hag-fa)。",
            },
            {
                "text": "「音節分隔」做得選連字符、空白抑係無隔開，像 hag-fa、hag fa、hagfa。",
            },
            {
                "text": "打開「自動加入空白」，送出羅馬字之後會自動加空格。",
            },
        ],
    },
    "nextWord": {
        "title": "連續建議詞",
        "paragraphs": [
            {
                "text": "選一隻詞之後，企盤會建議後一隻：選「恁」，「早」就會出現（恁早！）。",
            },
            {
                "text": "辭典無收个詞句，打過一擺就會記下來，下擺會出現在建議肚底。",
            },
        ],
    },
    "dictionaries": {
        "title": "辭典管理摎尋詞",
        "paragraphs": [
            {
                "text": "候選詞來自公開个辭典。預設包含教育部臺灣客語常用詞辭典、客家委員會認證詞彙等豐富語料。",
            },
            {
                "text": "「詞庫」>「辭典管理」提供各腔調辭典來源，逐隻都做得打開抑係關忒。",
            },
            {
                "text": "候選詞傷多个時節，就關忒無在用个辭典。若你單淨用一種腔，做得關忒其他腔。",
            },
            {
                "text": "詞典支援各腔調發音對應。",
            },
            {
                "text": "「辭典管理」頁个「尋字」，做得用漢字抑係羅馬字查辭典有收麼个詞。",
            },
            {
                "text": "看著有問題个詞，請透過回報管道通知𠊎兜。",
            },
        ],
    },
    "customDictionary": {
        "title": "自訂詞庫",
        "paragraphs": [
            {
                "text": "辭典無个詞，做得自家加：「詞庫」>「自訂詞庫」按 ＋，輸入羅馬字摎漢字。",
            },
            {
                "text": "乜做得匯入 CSV：第 1 欄羅馬字，第 2 欄漢字，毋使標題。",
            },
        ],
    },
    "learningRecords": {
        "title": "學習紀錄",
        "paragraphs": [
            {
                "text": "企盤會記你選過个詞，乜會記你一段一段打出个詞句。輒輒用个會排較頭前。",
            },
            {
                "text": "在詞庫个學習紀錄，做得看詞頻紀錄摎詞句紀錄，照最常用抑係最近用排序。",
            },
            {
                "text": "數量做得自家改，超過 40 排序就全部共樣。無愛个紀錄做得刪除。",
            },
            {
                "text": "想愛摎學過个詞加入自訂詞庫，iOS 摎該項紀錄向右邊滑，Android 按該項紀錄个 ＋。有漢字个紀錄正做得加。",
            },
        ],
    },
    "appearance": {
        "title": "字型摎外觀",
        "paragraphs": [
            {
                "text": "字型有多種選擇，支援客語特殊用字，系統字型有兜字可能顯示毋出。做得在「設定」>「字型設定」更換。",
            },
            {
                "text": "「主題」頁做得調色水、按鍵大小摎形狀，逐項都做得恢復預設。",
            },
        ],
    },
    "dataManagement": {
        "title": "資料摎隱私",
        "paragraphs": [
            {
                "text": "自訂詞摎學習紀錄全部放在手機裡肚，毋會送去網路。",
            },
            {
                "text": "刪除 App，資料會無忒。換手機進前，在「詞庫」>「備份復原」匯出；iOS 摎 Android 都做得匯入。",
            },
            {
                "text": "資料改過就重新匯出。分享備份檔進前，先檢查有無私密資料。",
            },
        ],
    },
    "taigiConverter": {
        "title": "客語通用轉換器",
        "paragraphs": [
            {
                "text": "線上工具：客語拼音、羅馬字相互轉換，聲調數字一擺轉做符號。",
            },
            {
                "text": "摎企盤用共一个核心，結果共樣。看著有毋著，請同𠊎講。",
            },
        ],
    },
}


def update_content():
    # 1. Update faq.json
    faq_path = REPO_ROOT / "i18n/content/faq.json"
    with open(faq_path, "r", encoding="utf-8") as f:
        faq_doc = json.load(f)

    for entry in faq_doc["faqs"]:
        eid = entry["id"]
        if eid in FAQ_UPDATES:
            u = FAQ_UPDATES[eid]
            entry["title"]["hanji"] = u["title"]
            for i, p in enumerate(entry["paragraphs"]):
                pu = u["paragraphs"][i]
                p["text"]["hanji"] = pu["text"]
                if "attachment_text" in pu and "attachment" in p:
                    p["attachment"]["text"]["hanji"] = pu["attachment_text"]

    faq_canonical = json.dumps(faq_doc, ensure_ascii=False, indent=2)
    faq_path.write_text(faq_canonical, encoding="utf-8")

    # 2. Update features.json
    feat_path = REPO_ROOT / "i18n/content/features.json"
    with open(feat_path, "r", encoding="utf-8") as f:
        feat_doc = json.load(f)

    for entry in feat_doc["features"]:
        eid = entry["id"]
        if eid in FEATURES_UPDATES:
            u = FEATURES_UPDATES[eid]
            entry["title"]["hanji"] = u["title"]
            if "summary" in u:
                entry["summary"]["hanji"] = u["summary"]
            for i, p in enumerate(entry["paragraphs"]):
                pu = u["paragraphs"][i]
                p["text"]["hanji"] = pu["text"]

    feat_canonical = json.dumps(feat_doc, ensure_ascii=False, indent=2)
    feat_path.write_text(feat_canonical, encoding="utf-8")

    # 3. Sync to Android and iOS
    android_assets = REPO_ROOT / "android/app/src/main/assets"
    ios_tabs = REPO_ROOT / "ios/Sources/TaigiKeyboard/App/Tabs/Home"

    for path in [
        android_assets / "faq.json",
        ios_tabs / "faq.json",
    ]:
        path.write_text(faq_canonical, encoding="utf-8")

    for path in [
        android_assets / "features.json",
        ios_tabs / "features.json",
    ]:
        path.write_text(feat_canonical, encoding="utf-8")

    print("Successfully translated and synced faq.json and features.json!")


if __name__ == "__main__":
    update_content()
