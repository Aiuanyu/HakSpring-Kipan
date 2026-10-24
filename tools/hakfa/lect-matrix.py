#!/usr/bin/env python3
"""各腔資料矩陣：每一腔在客源翠資料和企盤詞庫裡有多少（Obs IME lect-matrix 的數字來源）。

用法：python3 tools/hakfa/lect-matrix.py [客源翠 data 目錄]
預設讀 ../HakSpring/data；詞庫數字讀 dictionary/sources/official/hakspring/data/hakspring.csv。
只印 Markdown 表，不寫檔。數字是「現在量到的」，不是計畫。
"""
import csv
import io
import json
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATA = sys.argv[1] if len(sys.argv) > 1 else os.path.abspath(os.path.join(ROOT, "../HakSpring/data"))
CSV = os.path.join(ROOT, "dictionary/sources/official/hakspring/data/hakspring.csv")
BUILD = os.path.join(ROOT, "dictionary/sources/official/hakspring/extract_and_build.js")

# 資料檔上的腔字 → 名稱（客語自稱）
LECTS = [("四", "四縣 Xi'ien"), ("南", "南四縣 Nǎm Xi'ian"), ("海", "海陸 Hóiliug"),
         ("大", "大埔 Dapu"), ("平", "饒平 Raoping"), ("安", "詔安 Zhao'an")]


def csv_rows(path):
    """客源翠 json 檔的 content 是一整串 CSV；回傳資料列數（不含表頭）。"""
    if not os.path.exists(path):
        return None
    content = json.load(open(path, encoding="utf-8"))["content"]
    return max(0, sum(1 for _ in csv.reader(io.StringIO(content))) - 1)


def plain_csv_rows(path):
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        return max(0, sum(1 for _ in csv.reader(f)) - 1)


def built_lects():
    text = open(BUILD, encoding="utf-8").read()
    return set(re.findall(r"^\s*'(.)': \{ code: \d+, name:", text, re.M))


def dictionary_counts():
    """企盤合併詞庫（hakspring.csv）每腔、每來源位元的筆數。"""
    out = {}
    if not os.path.exists(CSV):
        return out
    for row in csv.DictReader(open(CSV, encoding="utf-8")):
        lect, mask = row["lect"], int(row["bitmask"])
        d = out.setdefault(lect, {"all": 0, "moe": 0, "cert": 0, "supp": 0})
        d["all"] += 1
        d["moe"] += bool(mask & 1)
        d["cert"] += bool(mask & 2)
        d["supp"] += bool(mask & 64)
    return out


def cross_map_members():
    path = os.path.join(DATA, "gip/cross_dialect_map.json")
    if not os.path.exists(path):
        return {}
    counts = {}
    for group in json.load(open(path, encoding="utf-8"))["groups"].values():
        for lect in group["members"]:
            counts[lect] = counts.get(lect, 0) + 1
    return counts


def cell(value):
    return "—" if value is None else f"{value:,}"


built = built_lects()
ours = dictionary_counts()
cross = cross_map_members()

print("### 客源翠資料有多少（原料）\n")
print("| 腔 | 教典 | 認證五級（列數合計） | 補字典 | 擬音字 | 跨腔對照組成員 |")
print("|---|---:|---:|---:|---:|---:|")
for key, name in LECTS:
    gip = csv_rows(os.path.join(DATA, f"gip/20250630-{key}.json"))
    cert = [csv_rows(os.path.join(DATA, f"cert/113{key}{lv}.json")) for lv in ("基", "初", "中", "中高", "高")]
    cert_total = None if all(c is None for c in cert) else sum(c or 0 for c in cert)
    supp = csv_rows(os.path.join(DATA, f"supp/補字典-{key}.json"))
    nisim = plain_csv_rows(os.path.join(DATA, f"supp/擬音-{key}.csv"))
    print(f"| {name} | {cell(gip)} | {cell(cert_total)} | {cell(supp)} | {cell(nisim)} | {cell(cross.get(key))} |")

print("\n### 企盤出貨詞庫有多少（建好的）\n")
print("| 腔 | 已建置 | 詞庫總筆數 | 其中教典 | 其中認證 | 其中補字典 |")
print("|---|:-:|---:|---:|---:|---:|")
for key, name in LECTS:
    d = ours.get(key)
    flag = "✅" if key in built else "—"
    if d:
        print(f"| {name} | {flag} | {d['all']:,} | {d['moe']:,} | {d['cert']:,} | {d['supp']:,} |")
    else:
        print(f"| {name} | {flag} | — | — | — | — |")

print("\n### 次方言補音附表（客源翠 sub-dialects-fill）\n")
for name in ("南四縣", "美濃"):
    path = os.path.join(DATA, f"sub-dialects-fill/{name}.json")
    if os.path.exists(path):
        print(f"- {name}：{len(json.load(open(path, encoding='utf-8'))['entries']):,} 列")
    else:
        print(f"- {name}：（無檔）")
