#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hakfa Dictionary Builder for HakSpring-Kipan (客話企盤)
Builds dictionary.bin (TKDB v3) and dictionary.fst from HakSpring data files.
"""

import json
import re
import struct
import subprocess
import sys
from pathlib import Path

MAGIC = b"TKDB"
VERSION = 3

REPO_ROOT = Path(__file__).resolve().parent.parent
HAKSPRING_DATA = Path("/Users/gjrobert/NAS/projects/HakSpring/data")
OUTPUT_DIR = REPO_ROOT / "output" / "hakfa"
FST_BUILDER = REPO_ROOT / "engine" / "target" / "debug" / "fst-builder"

def clean_reading(text: str) -> str:
    """Remove annotations like 【...】 or (...) and extra whitespace."""
    text = re.sub(r'【.*?】', '', text)
    text = re.sub(r'\(.*?\)', '', text)
    text = re.sub(r'（.*?）', '', text)
    return text.strip()

def extract_readings(disp: str, query: str):
    """
    Returns a list of (disp_clean, query_clean) pairs.
    Handles primary reading + alternative readings in 【...】.
    """
    res = []
    # Primary
    clean_d = clean_reading(disp)
    clean_q = clean_reading(query)
    if clean_d and clean_q:
        res.append((clean_d, clean_q))
        
    # Check if there is 【...】 alternative
    bracket_m = re.findall(r'【(.*?)】', disp)
    for alt_disp in bracket_m:
        alt_disp = alt_disp.strip()
        if alt_disp:
            res.append((alt_disp, alt_disp))
    return res

def process_hakfa_data():
    records = [] # list of (hanzi, disp, query, freq)
    seen = set()

    # 1. Load Cert 四縣 (Higher priority frequency)
    cert_files = [
        "113四基.json", "113四初.json", "113四中.json", "113四中高.json", "113四高.json"
    ]
    base_cert_freq = 50000
    for cf in cert_files:
        path = HAKSPRING_DATA / "cert" / cf
        if not path.exists():
            continue
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        lines = data.get("content", "").split("\n")
        if len(lines) <= 1:
            continue
        for line in lines[1:]:
            parts = line.split(",")
            if len(parts) < 4:
                continue
            hanzi = parts[1].strip()
            disp = parts[2].strip()
            query = parts[3].strip()
            if not hanzi or not disp:
                continue
            readings = extract_readings(disp, query)
            for d, q in readings:
                key = (hanzi, d)
                if key not in seen:
                    seen.add(key)
                    records.append((hanzi, d, q, base_cert_freq))
        base_cert_freq -= 5000

    # 2. Load GIP 四縣
    gip_path = HAKSPRING_DATA / "gip" / "20250630-四.json"
    if gip_path.exists():
        with open(gip_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        lines = data.get("content", "").split("\n")
        if len(lines) > 1:
            for line in lines[1:]:
                parts = line.split(",")
                if len(parts) < 4:
                    continue
                hanzi = parts[1].strip()
                disp = parts[2].strip()
                query = parts[3].strip()
                if not hanzi or not disp:
                    continue
                readings = extract_readings(disp, query)
                for d, q in readings:
                    key = (hanzi, d)
                    if key not in seen:
                        seen.add(key)
                        # default frequency
                        freq = max(100, 10000 - len(hanzi) * 1000)
                        records.append((hanzi, d, q, freq))

    print(f"Total collected Hakfa dictionary records: {len(records)}")
    return records

def build_bin_and_fst(records):
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    bin_path = OUTPUT_DIR / "dictionary.bin"
    fst_path = OUTPUT_DIR / "dictionary.fst"

    encoded_records = []
    fst_lines = []

    for rowid, (hanzi, disp, query, freq) in enumerate(records, start=1):
        # Parse query for syllables
        sylls = query.split()
        if not sylls:
            sylls = disp.split()
        
        notone_sylls = [re.sub(r'[\d\-]', '', s).lower() for s in sylls if s]
        notone = "".join(notone_sylls)
        abbrev = "".join(s[0] for s in notone_sylls if s)
        numeric = "".join(sylls).lower()
        syllable_count = min(len(sylls), 4)

        # FST entries
        if notone:
            fst_lines.append(f"TL\ttl:{notone}\t{rowid}\n")
        if numeric and numeric != notone:
            fst_lines.append(f"TL\ttl:{numeric}\t{rowid}\n")
        if abbrev:
            fst_lines.append(f"TL\ttl-abbrev:{abbrev}\t{rowid}\n")
        if hanzi:
            fst_lines.append(f"TL\thanzi:{hanzi}\t{rowid}\n")

        # Encode BIN record
        hanzi_bytes = hanzi.encode("utf-8")
        tl_bytes = disp.encode("utf-8")
        bitmask = 1 # official source bit
        kautian_subtag = 0 # default

        rec_bytes = struct.pack(
            f"<HIBBBH{len(hanzi_bytes)}s{len(tl_bytes)}s",
            bitmask,
            freq,
            len(hanzi_bytes),
            len(tl_bytes),
            syllable_count,
            kautian_subtag,
            hanzi_bytes,
            tl_bytes,
        )
        encoded_records.append(rec_bytes)

    # 1. Write dictionary.bin
    count = len(encoded_records)
    build_ts = 20261007
    header_size = 16
    offset_table_size = count * 4
    data_start = header_size + offset_table_size

    offsets = []
    current_offset = data_start
    for rec in encoded_records:
        offsets.append(current_offset)
        current_offset += len(rec)

    with open(bin_path, "wb") as f:
        f.write(MAGIC)
        f.write(struct.pack("<III", VERSION, count, build_ts))
        for off in offsets:
            f.write(struct.pack("<I", off))
        for rec in encoded_records:
            f.write(rec)

    print(f"Successfully wrote {bin_path} ({count} entries, {bin_path.stat().st_size / 1024 / 1024:.2f} MB)")

    # 2. Build dictionary.fst via fst-builder
    if not FST_BUILDER.exists():
        print(f"Error: {FST_BUILDER} does not exist. Please run cargo build -p fst-builder first.")
        sys.exit(1)

    print(f"Feeding {len(fst_lines)} keys into fst-builder...")
    proc = subprocess.Popen(
        [str(FST_BUILDER), "build", str(fst_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    stdout, stderr = proc.communicate("".join(fst_lines))
    if proc.returncode != 0:
        print(f"fst-builder error: {stderr}")
        sys.exit(1)
    else:
        print(stderr.strip())
        print(f"Successfully built {fst_path} ({fst_path.stat().st_size / 1024 / 1024:.2f} MB)")

if __name__ == "__main__":
    records = process_hakfa_data()
    build_bin_and_fst(records)
