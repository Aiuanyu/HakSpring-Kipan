#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hakfa Gloss & Synonym Database Builder for HakSpring-Kipan (客話企盤)
Extracts definitions and synonyms from HakSpring cert and gip data files,
merges duplicates, and compiles them into assets/dictionaries/gloss.db (SQLite).
"""

import csv
import json
import os
import re
import sqlite3
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
HAKSPRING_DATA = Path(os.environ.get("HAKSPRING_ROOT", str(REPO_ROOT.parent / "HakSpring"))) / "data"
OUTPUT_DB = REPO_ROOT / "assets" / "dictionaries" / "gloss.db"

# 腔調代號與名稱（嚴格遵守「四南海大平安」順序）
LECT_ROSTER = [
    ("xiien", "四縣", "四"),
    ("namxiian", "南四縣", "南"),
    ("hoiliug", "海陸", "海"),
    ("taibu", "大埔", "大"),
    ("ngiaupin", "饒平", "平"),
    ("zhioon", "詔安", "安"),
]

def clean_brackets(text: str) -> str:
    """去除註記括號（保留純漢字詞目）"""
    text = re.sub(r'【.*?】', '', text)
    text = re.sub(r'\(.*?\)', '', text)
    text = re.sub(r'（.*?）', '', text)
    return text.strip()

def extract_variants_from_word(raw_word: str):
    """
    從帶括號的詞目中展開出所有合法詞目變體：
    例如「發子（病子）」-> ["發子", "病子"]
    「啞仔【啞眵】」-> ["啞仔", "啞眵"]
    """
    results = set()
    base = clean_brackets(raw_word)
    if base:
        results.add(base)
    for m in re.findall(r'[（\(](.*?)[）\)]', raw_word):
        m = m.strip()
        if m and not re.search(r'[a-zA-Z0-9]', m):
            for sub in re.split(r'[,，、/]+', m):
                sub = sub.strip()
                if sub:
                    results.add(sub)
    for m in re.findall(r'【(.*?)】', raw_word):
        m = m.strip()
        if m and not re.search(r'[a-zA-Z0-9]', m):
            for sub in re.split(r'[,，、/]+', m):
                sub = sub.strip()
                if sub:
                    results.add(sub)
    return sorted(list(results))

def extract_gip_gloss_and_synonyms(def_text: str, similar_text: str):
    synonyms = set()
    if similar_text and similar_text.strip():
        for s in re.split(r'[,，、;；/\s]+', similar_text):
            s = clean_brackets(s).strip()
            if s and len(s) <= 8 and not re.search(r'[。！？]', s):
                synonyms.add(s)

    clean_def = def_text.strip()
    clean_def = re.sub(r'^[1-9]\.\s*|^\([1-9]\)\s*', '', clean_def)
    parts = re.split(r'[。例：\n]', clean_def)
    first_sentence = parts[0].strip() if parts else clean_def

    tokens = re.split(r'[,，、;；/]+', first_sentence)
    for tok in tokens:
        pure_tok = clean_brackets(tok).strip()
        if pure_tok and 1 <= len(pure_tok) <= 6 and not re.search(r'[。！？]', pure_tok):
            synonyms.add(pure_tok)

    return first_sentence, sorted(list(synonyms))

def extract_cert_gloss_and_synonyms(def_text: str):
    synonyms = set()
    clean_def = def_text.strip()
    parts = re.split(r'[。例：\n]', clean_def)
    first_sentence = parts[0].strip() if parts else clean_def

    for match in re.finditer(r'(?:也|亦)可(?:說|稱|做|作)[為：:\s]*([^\n。；;]+)', clean_def):
        for s in re.split(r'[,，、;；/]+', match.group(1)):
            s = clean_brackets(s).strip()
            if s and len(s) <= 8:
                synonyms.add(s)

    tokens = re.split(r'[,，、;；/]+', first_sentence)
    for tok in tokens:
        pure_tok = clean_brackets(tok).strip()
        if pure_tok and 1 <= len(pure_tok) <= 6 and not re.search(r'[。！？]', pure_tok):
            synonyms.add(pure_tok)

    return first_sentence, sorted(list(synonyms))

def build_database():
    print(f"==> Building optimized gloss & synonym database from {HAKSPRING_DATA}")
    OUTPUT_DB.parent.mkdir(parents=True, exist_ok=True)
    if OUTPUT_DB.exists():
        OUTPUT_DB.unlink()

    entry_map = {}

    # 1. 處理認證 (CERT) - 認證有高質量的 HPP 調符與現代華語詞義，優先度高
    cert_dir = HAKSPRING_DATA / "cert"
    if cert_dir.exists():
        for lect_id, lect_name, char_key in LECT_ROSTER:
            search_char = '四' if char_key == '南' else char_key
            matched_files = sorted(list(cert_dir.glob(f"*{search_char}*.csv")))
            for csv_path in matched_files:
                level_name = csv_path.stem
                with open(csv_path, encoding='utf-8') as f:
                    reader = csv.DictReader(f)
                    fieldnames = reader.fieldnames or []
                    word_col = next((c for c in fieldnames if '客家語' in c), None)
                    reading_col = next((c for c in fieldnames if '標音' in c), None)
                    gloss_col = next((c for c in fieldnames if '詞義' in c), None)
                    if not word_col or not reading_col or not gloss_col:
                        continue

                    for row in reader:
                        raw_word = row.get(word_col, '').strip()
                        raw_reading = row.get(reading_col, '').strip()
                        def_text = row.get(gloss_col, '').strip()
                        if not raw_word:
                            continue

                        dialect_tag = ""
                        reading = raw_reading
                        if char_key == '南':
                            south_match = re.search(r'【(.*?)】', raw_reading)
                            if south_match:
                                reading = south_match.group(1).strip()
                                dialect_tag = "南"
                        elif '【' in raw_reading:
                            reading = clean_brackets(raw_reading)

                        gloss, syns = extract_cert_gloss_and_synonyms(def_text)
                        words = extract_variants_from_word(raw_word)

                        for w in words:
                            k = (w, lect_id)
                            if k not in entry_map:
                                entry_map[k] = {
                                    'lect_name': lect_name,
                                    'reading_hpp': reading,
                                    'reading_raw': reading,
                                    'gloss': gloss,
                                    'synonyms': set(syns),
                                    'sources': {f"認證-{level_name}"},
                                    'dialect_tags': {dialect_tag} if dialect_tag else set()
                                }
                            else:
                                item = entry_map[k]
                                if not item['reading_hpp'] and reading:
                                    item['reading_hpp'] = reading
                                if not item['gloss'] and gloss:
                                    item['gloss'] = gloss
                                item['synonyms'].update(syns)
                                item['sources'].add(f"認證-{level_name}")
                                if dialect_tag:
                                    item['dialect_tags'].add(dialect_tag)

    # 2. 處理教典 (GIP) - 補充認證未收錄之大量傳統客語詞條與豐富釋義
    gip_dir = HAKSPRING_DATA / "gip"
    if gip_dir.exists():
        for lect_id, lect_name, char_key in LECT_ROSTER:
            pattern = f"*-{char_key}.csv"
            matched_files = list(gip_dir.glob(pattern))
            if not matched_files:
                continue
            csv_path = matched_files[0]
            print(f"  Reading GIP {lect_name} ({csv_path.name})...")
            with open(csv_path, encoding='utf-8') as f:
                reader = csv.DictReader(f)
                for row in reader:
                    raw_word = row.get('詞目', '').strip()
                    reading = row.get('音讀', '').strip()
                    def_text = row.get('釋義', '').strip()
                    similar = row.get('相似詞', '').strip()
                    point = row.get('方言點', '').strip()
                    if not raw_word:
                        continue

                    gloss, syns = extract_gip_gloss_and_synonyms(def_text, similar)
                    words = extract_variants_from_word(raw_word)

                    for w in words:
                        k = (w, lect_id)
                        if k not in entry_map:
                            entry_map[k] = {
                                'lect_name': lect_name,
                                'reading_hpp': '',
                                'reading_raw': reading,
                                'gloss': gloss,
                                'synonyms': set(syns),
                                'sources': {"教典"},
                                'dialect_tags': {point} if point else set()
                            }
                        else:
                            item = entry_map[k]
                            if not item['gloss'] and gloss:
                                item['gloss'] = gloss
                            item['synonyms'].update(syns)
                            item['sources'].add("教典")
                            if not item['reading_raw']:
                                item['reading_raw'] = reading
                            if point:
                                item['dialect_tags'].add(point)

    print(f"==> Deduplicated into {len(entry_map)} unique (word, lect) entries")

    entry_rows = []
    synonym_rows = set()

    for (w, lect_id), item in entry_map.items():
        reading = item['reading_hpp'] or item['reading_raw']
        gloss = item['gloss']
        syns = sorted(list(item['synonyms']))
        syns_str = ",".join(syns) if syns else ""
        source_str = "、".join(sorted(list(item['sources'])))
        dialect_str = "　".join(sorted(list(item['dialect_tags'])))

        entry_rows.append((
            w, lect_id, item['lect_name'], reading, gloss, syns_str, source_str, dialect_str
        ))

        for syn in syns:
            if syn != w:
                synonym_rows.add((syn, w, lect_id))

    conn = sqlite3.connect(OUTPUT_DB)
    cur = conn.cursor()

    cur.execute("PRAGMA page_size = 4096;")

    cur.execute("""
    CREATE TABLE entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        word TEXT NOT NULL,
        lect_id TEXT NOT NULL,
        lect_name TEXT NOT NULL,
        reading TEXT NOT NULL,
        gloss TEXT,
        synonyms TEXT,
        source TEXT,
        dialect_tags TEXT
    );
    """)

    cur.execute("""
    CREATE TABLE synonyms (
        keyword TEXT NOT NULL,
        word TEXT NOT NULL,
        lect_id TEXT NOT NULL,
        PRIMARY KEY(keyword, word, lect_id)
    );
    """)

    print(f"==> Inserting {len(entry_rows)} entries and {len(synonym_rows)} synonyms...")
    cur.executemany("""
    INSERT INTO entries (word, lect_id, lect_name, reading, gloss, synonyms, source, dialect_tags)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?);
    """, entry_rows)

    cur.executemany("""
    INSERT OR IGNORE INTO synonyms (keyword, word, lect_id)
    VALUES (?, ?, ?);
    """, list(synonym_rows))

    print("==> Creating indexes...")
    cur.execute("CREATE INDEX idx_entries_word ON entries(word);")
    cur.execute("CREATE INDEX idx_entries_lect ON entries(lect_id);")
    cur.execute("CREATE INDEX idx_synonyms_keyword ON synonyms(keyword);")

    conn.commit()
    cur.execute("VACUUM;")
    conn.close()

    db_size = OUTPUT_DB.stat().st_size / (1024 * 1024)
    print(f"✓ Complete! Generated optimized {OUTPUT_DB} ({db_size:.2f} MB)")

if __name__ == "__main__":
    build_database()
