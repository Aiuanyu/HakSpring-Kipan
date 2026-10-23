#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hakfa Unigram Corpus & Language Model Extractor (客話語言模型語料萃取器)

本工具為「客源翠 × 企盤」核心語言模型建置管線：
從客源翠教典（16,653 句道地例句）與認證詞庫（7,222 句日常例句）中，
精確提取近 90 萬字純客語標準文本，並以 23,071 筆客話詞庫為詞彙表，
透過正向最大匹配 (Forward Maximum Match, FMM) 建立客話 Unigram 詞頻分佈。

產出成果：
- dictionary/sources/official/hakspring/data/hakfa_unigrams.json
- 供 extract_and_build.js 與 tkdb_v4.js 進行 walker_cost (Lidstone add-alpha) 與 frequency boost。
"""

import csv
import glob
import json
import os
import re
from collections import Counter
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
HAKSPRING_ROOT = Path(os.environ.get("HAKSPRING_ROOT", REPO_ROOT.parent / "HakSpring-main"))
HAKSPRING_DATA = HAKSPRING_ROOT / "data"

DICT_CSV = REPO_ROOT / "dictionary" / "sources" / "official" / "hakspring" / "data" / "hakspring.csv"
OUTPUT_JSON = REPO_ROOT / "dictionary" / "sources" / "official" / "hakspring" / "data" / "hakfa_unigrams.json"


def load_lexicon(dict_csv_path):
    """載入客話詞庫所有詞條，建立詞表集合與長度索引"""
    words = set()
    records = []
    with open(dict_csv_path, "r", encoding="utf-8") as f:
        reader = csv.reader(f)
        header = next(reader)
        for row in reader:
            if not row:
                continue
            word = row[0].strip()
            tl = row[1].strip()
            tl_notone = row[3].strip() if len(row) > 3 else ""
            words.add(word)
            records.append((word, tl, tl_notone))
    print(f"📖 載入客話詞庫：{len(records)} 筆記錄，{len(words)} 個相異詞目")
    return words, records


def extract_gip_sentences(gip_csv_path):
    """從教典中提取純客語例句（精準過濾華語翻譯與標籤）"""
    sentences = []
    if not gip_csv_path.exists():
        print(f"⚠️ 找不到教典檔案：{gip_csv_path}")
        return sentences

    with open(gip_csv_path, "r", encoding="utf-8") as f:
        reader = csv.reader(f)
        next(reader, None)
        for row in reader:
            if len(row) <= 7 or not row[7].strip():
                continue
            raw = row[7].strip()

            # 情況 1：含有括號「例：客語(華語) 例：客語(華語)」
            if "(" in raw or "（" in raw:
                # 剔除括號內的華語翻譯
                clean = re.sub(r"\([^)]*\)", "", raw)
                clean = re.sub(r"（[^）]*）", "", clean)
                # 濾除所有字典標籤（例如：、例句：、例：、如：）並分割
                parts = [p.strip() for p in re.split(r"(?:例如|例句|例|如)\s*[:：]", clean) if p.strip()]
                for p in parts:
                    if p:
                        sentences.append(p)
            else:
                # 情況 2：無括號，格式多為「例：客語句。 華語翻譯句。」（空白分隔）
                parts = re.split(r"\s+", raw)
                if parts and parts[0].strip():
                    first_part = parts[0].strip()
                    # 濾除開頭之標籤
                    first_clean = re.sub(r"^(?:例如|例句|例|如)\s*[:：]", "", first_part).strip()
                    if first_clean:
                        sentences.append(first_clean)

    print(f"📚 教典提取純客語例句：{len(sentences)} 句")
    return sentences


def extract_cert_sentences(cert_dir_path):
    """從認證題庫提取純客語例句"""
    sentences = []
    for cert_file in sorted(cert_dir_path.glob("113四*.csv")):
        with open(cert_file, "r", encoding="utf-8") as f:
            reader = csv.reader(f)
            next(reader, None)
            for row in reader:
                # 認證例句欄位在第 5 欄 (index 4)
                if len(row) > 4 and row[4].strip():
                    raw = row[4].strip()
                    # 去除內部括號同音字註記，例：洩（射）-> 洩
                    clean = re.sub(r"（[^）]*）", "", raw)
                    clean = re.sub(r"\([^)]*\)", "", clean)
                    # 處理換行 <br> 分隔之多句
                    subs = clean.split("<br>")
                    for s in subs:
                        s = s.strip()
                        # 濾除認證中高級與高級中大量出現之開頭標籤「例如：」
                        s = re.sub(r"^(?:例如|例句|例|如)\s*[:：]", "", s).strip()
                        if s:
                            sentences.append(s)

    print(f"📚 認證提取純客語例句：{len(sentences)} 句")
    return sentences


def tokenize_corpus(sentences, word_set):
    """
    正向最大匹配分詞 (Forward Maximum Match)
    長詞優先匹配，精確統計每個詞目之出現次數。
    """
    max_len = max(len(w) for w in word_set) if word_set else 8
    unigram_counts = Counter()
    total_tokens = 0
    total_chars = sum(len(s) for s in sentences)

    # 漢字區間正則表達式
    cjk_pattern = re.compile(r"[\u4e00-\u9fff\u3400-\u4dbf\U00020000-\U0002A6DF]+")

    for text in sentences:
        chunks = cjk_pattern.findall(text)
        for chunk in chunks:
            i = 0
            n = len(chunk)
            while i < n:
                matched = False
                for l in range(min(max_len, n - i), 0, -1):
                    sub = chunk[i:i + l]
                    if sub in word_set:
                        unigram_counts[sub] += 1
                        total_tokens += 1
                        i += l
                        matched = True
                        break
                if not matched:
                    # 詞庫未收錄字，步進 1
                    i += 1

    return unigram_counts, total_tokens, total_chars


def main():
    print("==================================================")
    print("🚀 客話語言模型語料萃取器 (Hakfa Unigrams)")
    print(f"📁 客源翠資料根目錄：{HAKSPRING_DATA}")
    print("==================================================")

    # 1. 載入詞庫
    word_set, records = load_lexicon(DICT_CSV)

    # 2. 蒐集語料
    sentences = []
    sentences.extend(extract_gip_sentences(HAKSPRING_DATA / "gip" / "20250630-四.csv"))
    sentences.extend(extract_cert_sentences(HAKSPRING_DATA / "cert"))

    print(f"\n📊 語料庫彙整完成：共 {len(sentences)} 句客話標準文本")

    # 3. 分詞與詞頻統計
    print("⏳ 開始執行正向最大匹配分詞與詞頻統計...")
    counts, total_tokens, total_chars = tokenize_corpus(sentences, word_set)

    print(f"\n✅ 分詞計數完成！")
    print(f"  - 語料總字數：{total_chars:,} 字")
    print(f"  - 總 Token 數：{total_tokens:,} 詞次")
    print(f"  - 被計數之詞庫詞目：{len(counts)} / {len(word_set)} ({len(counts)/len(word_set)*100:.1f}%)")

    # 4. 輸出檢驗
    print("\n【Top 30 高頻詞目】")
    for w, c in counts.most_common(30):
        print(f"  {w:8s}: {c:6d} 次")

    print("\n【關鍵高頻對決指標】")
    pairs = [
        ("个", "該"),
        ("摎", "落"),
        ("請", "青"),
        ("又有", "幼幼"),
        ("個人", "个人"),
    ]
    for w1, w2 in pairs:
        c1, c2 = counts.get(w1, 0), counts.get(w2, 0)
        winner = w1 if c1 > c2 else (w2 if c2 > c1 else "平手")
        print(f"  {w1:6s} ({c1:5d} 次) vs {w2:6s} ({c2:5d} 次) ➔ 【{winner}】勝出！")

    # 5. 輸出 JSON 成果
    OUTPUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "metadata": {
            "description": "客源翠教典與認證純客語例句 Unigram 詞頻分佈",
            "total_sentences": len(sentences),
            "total_chars": total_chars,
            "total_tokens": total_tokens,
            "vocabulary_size": len(word_set),
            "counted_words": len(counts),
        },
        "unigrams": dict(counts)
    }

    with open(OUTPUT_JSON, "w", encoding="utf-8") as f:
        json.dump(payload, f, ensure_ascii=False, indent=2)

    print(f"\n🎉 成果已成功儲存至：{OUTPUT_JSON}")


if __name__ == "__main__":
    main()
