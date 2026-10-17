#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hakfa Association Builder for HakSpring-Kipan.
Generates `association.bin` (TKWA v2) directly from HakSpring dictionary and common Hakfa bigrams.

Replaces the inherited Taigi `association.bin` to eliminate "台語齒盤" next-word recommendations
and provide authentic Hakfa next-word prediction (e.g., 大家 -> 好, 承蒙 -> 你, 客源 -> 翠).
Ensures 100% of association entries have valid Hakfa romanization (next_tl).
"""

import csv
import re
import struct
import sys
import time
from collections import defaultdict
from pathlib import Path

MAGIC = b"TKWA"
VERSION = 2

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent
HAKSPRING_CSV = REPO_ROOT / "dictionary" / "sources" / "official" / "hakspring" / "data" / "hakspring.csv"
OUTPUT_DIR = REPO_ROOT / "output" / "hakfa"
ASSETS_DIR = REPO_ROOT / "assets" / "dictionaries"
OUTPUT_FILE = OUTPUT_DIR / "association.bin"


def is_cjk(ch: str) -> bool:
    if len(ch) != 1:
        return False
    cp = ord(ch)
    return (
        0x4E00 <= cp <= 0x9FFF
        or 0x3400 <= cp <= 0x4DBF
        or 0x20000 <= cp <= 0x2A6DF
        or 0x2A700 <= cp <= 0x2B73F
        or 0x2B740 <= cp <= 0x2B81F
        or 0x2B820 <= cp <= 0x2CEAF
        or 0x2CEB0 <= cp <= 0x2EBBF
        or 0x30000 <= cp <= 0x3134F
        or 0x31350 <= cp <= 0x323AF
        or 0xF900 <= cp <= 0xFAFF
    )


class AssocEntry:
    __slots__ = ("next_word", "next_tl", "count")

    def __init__(self, next_word: str, next_tl: str = "", count: int = 100):
        self.next_word = next_word
        self.next_tl = next_tl
        self.count = count

    def encode(self) -> bytes:
        nw_bytes = self.next_word.encode("utf-8")
        tl_bytes = self.next_tl.encode("utf-8")
        bitmask = 1  # main source bit
        return struct.pack(
            f"<HIBB{len(nw_bytes)}s{len(tl_bytes)}s",
            bitmask,
            self.count,
            len(nw_bytes),
            len(tl_bytes),
            nw_bytes,
            tl_bytes,
        )


def main():
    print("====================================================")
    print("  建置 HakSpring-Kipan 客話接續詞模型 (association.bin)")
    print("====================================================")

    if not HAKSPRING_CSV.exists():
        print(f"錯誤：找不到 {HAKSPRING_CSV}")
        sys.exit(1)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    dict_word_to_tl = {}
    dict_char_to_tl = {}
    char_freq = defaultdict(int)

    # 1. 建立詞目與單字發音對照庫
    print(f"▶ 建立詞目與字音反查庫：{HAKSPRING_CSV}")
    raw_records = []
    with open(HAKSPRING_CSV, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            word = row.get("word", "").strip()
            tl = row.get("tl", "").strip()
            try:
                freq = int(row.get("frequency", "50000"))
            except ValueError:
                freq = 50000

            if not word or not tl:
                continue

            raw_records.append((word, tl, freq))

            # 記錄完整詞的讀音
            if word not in dict_word_to_tl:
                dict_word_to_tl[word] = tl

            # 單字直接收錄
            if len(word) == 1 and is_cjk(word):
                if freq > char_freq[word]:
                    dict_char_to_tl[word] = tl
                    char_freq[word] = freq

            # 切分記錄單字字音
            cjk_chars = [c for c in word if is_cjk(c)]
            tl_parts = [p.strip() for p in tl.split("-") if p.strip()]
            if len(cjk_chars) == len(tl_parts):
                for c, p in zip(cjk_chars, tl_parts):
                    if freq > char_freq[c]:
                        dict_char_to_tl[c] = p
                        char_freq[c] = freq

    print(f"  ✓ 已建立字音庫：{len(dict_char_to_tl)} 個單字，{len(dict_word_to_tl)} 個詞條")

    grouped = defaultdict(list)
    pair_counts = defaultdict(int)
    pair_tl = {}

    def resolve_tl(word_str: str, candidate_tl: str = "") -> str:
        if candidate_tl:
            return candidate_tl
        if word_str in dict_word_to_tl:
            return dict_word_to_tl[word_str]
        if len(word_str) == 1 and word_str in dict_char_to_tl:
            return dict_char_to_tl[word_str]
        # 逐字拼合
        parts = [dict_char_to_tl.get(c, "") for c in word_str if is_cjk(c)]
        if all(parts) and len(parts) == len(word_str):
            return "-".join(parts)
        return ""

    # 2. 提取詞條內的相鄰字詞關係
    for word, tl, freq in raw_records:
        # 若有多個同義詞（以頓號、斜線隔開），分別拆解
        sub_words = re.split(r"[,、/，]", word)
        sub_tls = re.split(r"[,、/，]", tl)
        pairs_to_process = []
        if len(sub_words) == len(sub_tls):
            for sw, stl in zip(sub_words, sub_tls):
                pairs_to_process.append((sw.strip(), stl.strip()))
        else:
            pairs_to_process.append((word, tl))

        for w_item, tl_item in pairs_to_process:
            hanzi_chars = [c for c in w_item if is_cjk(c)]
            if len(hanzi_chars) < 2:
                continue

            tl_parts = [p.strip() for p in tl_item.split("-") if p.strip()]
            tl_valid = (len(tl_parts) == len(hanzi_chars))

            # 相鄰字關係 (prev_c -> next_c)
            for i in range(len(hanzi_chars) - 1):
                prev_c = hanzi_chars[i]
                next_c = hanzi_chars[i + 1]
                pair_key = (prev_c, next_c)
                pair_counts[pair_key] += freq
                if pair_key not in pair_tl:
                    cand = tl_parts[i + 1] if tl_valid else ""
                    resolved = resolve_tl(next_c, cand)
                    if resolved:
                        pair_tl[pair_key] = resolved

            # 片語關聯 (prev_c -> rem_phrase)
            if 3 <= len(hanzi_chars) <= 4:
                prev_c = hanzi_chars[0]
                rem_phrase = "".join(hanzi_chars[1:])
                cand = "-".join(tl_parts[1:]) if tl_valid else ""
                phrase_key = (prev_c, rem_phrase)
                pair_counts[phrase_key] += freq
                if phrase_key not in pair_tl:
                    resolved = resolve_tl(rem_phrase, cand)
                    if resolved:
                        pair_tl[phrase_key] = resolved

    # 3. 注入常用客家口語問候與高頻搭配詞對
    extra_pairs = [
        ("大", "家", "gá", 100000),
        ("大家", "好", "hó", 95000),
        ("食", "飽", "báu", 90000),
        ("食飽", "吂", "mǎng", 85000),
        ("承", "蒙", "mùng", 90000),
        ("承蒙", "你", "ngǐ", 85000),
        ("恁", "仔細", "zii-se", 80000),
        ("客", "話", "fa", 90000),
        ("客話", "先生", "sien-sáng", 75000),
        ("客", "源翠", "ngièn-cui", 90000),
        ("客源", "翠", "cui", 85000),
        ("企", "盤", "pàn", 80000),
        ("客源翠", "企盤", "ki-pàn", 75000),
        ("早", "安", "on", 70000),
        ("平", "安", "on", 70000),
        ("請", "坐", "co", 65000),
        ("請", "食茶", "siid-cà", 65000),
    ]

    for prev_w, next_w, ntl, weight in extra_pairs:
        k = (prev_w, next_w)
        pair_counts[k] += weight
        pair_tl[k] = ntl

    # 4. 整理至 grouped，強制要求 100% 具有羅馬字
    skipped_count = 0
    for (prev_w, next_w), count in pair_counts.items():
        ntl = pair_tl.get((prev_w, next_w), "")
        if not ntl:
            ntl = resolve_tl(next_w)

        if not ntl:
            skipped_count += 1
            continue

        grouped[prev_w].append(AssocEntry(next_w, ntl, count))

    # 排序每組內的 entries (count DESC)
    for prev_w in grouped:
        grouped[prev_w].sort(key=lambda e: e.count, reverse=True)
        if len(grouped[prev_w]) > 30:
            grouped[prev_w] = grouped[prev_w][:30]

    all_keys = sorted(grouped.keys(), key=lambda s: s.encode("utf-8"))
    key_count = len(all_keys)
    total_entries = sum(len(grouped[k]) for k in all_keys)

    print(f"▶ 統計：前置鍵數 = {key_count}, 接續詞條總數 = {total_entries} (剔除無音條目 = {skipped_count})")

    # 5. 二進制序列化
    encoded_groups = {k: [e.encode() for e in grouped[k]] for k in all_keys}

    build_ts = int(time.time())
    header_size = 20
    key_offset_table_size = key_count * 4
    key_section_start = header_size + key_offset_table_size

    key_entries_data = []
    for key in all_keys:
        kb = key.encode("utf-8")
        assert len(kb) <= 0xFF, f"Key too long: {key}"
        key_entries_data.append((kb, len(encoded_groups[key])))

    key_section_size = sum(1 + len(kb) + 4 + 2 for kb, _ in key_entries_data)
    entry_section_start = key_section_start + key_section_size

    entry_offset = entry_section_start
    key_entry_offsets = []
    for key in all_keys:
        key_entry_offsets.append(entry_offset)
        for enc in encoded_groups[key]:
            entry_offset += len(enc)

    key_offsets = []
    pos = key_section_start
    for kb, _ in key_entries_data:
        key_offsets.append(pos)
        pos += 1 + len(kb) + 4 + 2

    with open(OUTPUT_FILE, "wb") as f:
        f.write(MAGIC)
        f.write(struct.pack("<IIII", VERSION, key_count, total_entries, build_ts))

        for off in key_offsets:
            f.write(struct.pack("<I", off))

        for i, (kb, entry_count) in enumerate(key_entries_data):
            f.write(struct.pack("B", len(kb)))
            f.write(kb)
            f.write(struct.pack("<IH", key_entry_offsets[i], entry_count))

        for key in all_keys:
            for enc in encoded_groups[key]:
                f.write(enc)

    file_size = OUTPUT_FILE.stat().st_size
    print(f"  ✓ 成功產出：{OUTPUT_FILE} ({file_size / 1024:.1f} KB)")

    # 部署至 assets/dictionaries/
    asset_target = ASSETS_DIR / "association.bin"
    if ASSETS_DIR.exists():
        import shutil
        shutil.copyfile(OUTPUT_FILE, asset_target)
        print(f"  ✓ 成功部署至：{asset_target}")

    print("\n🎉 客話接續詞模型 association.bin 重構完成，100% 帶有精確客話標音！")


if __name__ == "__main__":
    main()
