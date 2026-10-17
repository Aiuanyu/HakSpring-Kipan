#!/usr/bin/env python3
"""
Merge User Data from old bundle ID to new bundle ID
From: ~/Library/Application Support/tw.aiuanyu.inputmethod.HakSpring-Kipan
To:   ~/Library/Application Support/net.aiuanyu.inputmethod.HakSpring-Kipan
"""
import os
import sqlite3
import sys

OLD_DIR = os.path.expanduser("~/Library/Application Support/tw.aiuanyu.inputmethod.HakSpring-Kipan")
NEW_DIR = os.path.expanduser("~/Library/Application Support/net.aiuanyu.inputmethod.HakSpring-Kipan")

def merge_user_frequency():
    old_db = os.path.join(OLD_DIR, "user_frequency.db")
    new_db = os.path.join(NEW_DIR, "user_frequency.db")
    if not os.path.exists(old_db) or not os.path.exists(new_db):
        print(f"[-] user_frequency.db skipped (one of them does not exist)")
        return

    conn_old = sqlite3.connect(old_db)
    conn_new = sqlite3.connect(new_db)

    old_rows = conn_old.execute("SELECT word, tl, count, last_used, created_at FROM user_frequency").fetchall()
    inserted = 0
    updated = 0

    for word, tl, count, last_used, created_at in old_rows:
        cur = conn_new.execute("SELECT id, count, last_used FROM user_frequency WHERE word = ? AND tl = ?", (word, tl))
        row = cur.fetchone()
        if row:
            target_id, target_count, target_last_used = row
            new_count = target_count + count
            new_last_used = max(target_last_used or '', last_used or '')
            conn_new.execute("UPDATE user_frequency SET count = ?, last_used = ? WHERE id = ?", (new_count, new_last_used, target_id))
            updated += 1
        else:
            conn_new.execute("INSERT INTO user_frequency (word, tl, count, last_used, created_at) VALUES (?, ?, ?, ?, ?)",
                             (word, tl, count, last_used, created_at))
            inserted += 1

    conn_new.commit()
    conn_old.close()
    conn_new.close()
    print(f"[+] user_frequency.db merged: {inserted} inserted, {updated} updated")

def merge_user_association():
    old_db = os.path.join(OLD_DIR, "user_association.db")
    new_db = os.path.join(NEW_DIR, "user_association.db")
    if not os.path.exists(old_db) or not os.path.exists(new_db):
        print(f"[-] user_association.db skipped")
        return

    conn_old = sqlite3.connect(old_db)
    conn_new = sqlite3.connect(new_db)

    old_rows = conn_old.execute("SELECT prev_word, prev_tl, next_word, next_tl, count, last_used FROM user_association").fetchall()
    inserted = 0
    updated = 0

    for prev_word, prev_tl, next_word, next_tl, count, last_used in old_rows:
        cur = conn_new.execute("""
            SELECT id, count, last_used FROM user_association
            WHERE prev_word = ? AND prev_tl = ? AND next_word = ? AND next_tl = ?
        """, (prev_word, prev_tl, next_word, next_tl))
        row = cur.fetchone()
        if row:
            target_id, target_count, target_last_used = row
            new_count = target_count + count
            new_last_used = max(target_last_used or '', last_used or '')
            conn_new.execute("UPDATE user_association SET count = ?, last_used = ? WHERE id = ?", (new_count, new_last_used, target_id))
            updated += 1
        else:
            conn_new.execute("""
                INSERT INTO user_association (prev_word, prev_tl, next_word, next_tl, count, last_used)
                VALUES (?, ?, ?, ?, ?, ?)
            """, (prev_word, prev_tl, next_word, next_tl, count, last_used))
            inserted += 1

    conn_new.commit()
    conn_old.close()
    conn_new.close()
    print(f"[+] user_association.db merged: {inserted} inserted, {updated} updated")

def merge_custom_dictionary():
    old_db = os.path.join(OLD_DIR, "custom_dictionary.db")
    new_db = os.path.join(NEW_DIR, "custom_dictionary.db")
    if not os.path.exists(old_db) or not os.path.exists(new_db):
        print(f"[-] custom_dictionary.db skipped")
        return

    conn_old = sqlite3.connect(old_db)
    conn_new = sqlite3.connect(new_db)

    old_words = conn_old.execute("SELECT id, roman, hanzi, created_at, updated_at FROM custom_dictionary").fetchall()
    inserted = 0
    skipped = 0

    for old_id, roman, hanzi, created_at, updated_at in old_words:
        cur = conn_new.execute("SELECT id FROM custom_dictionary WHERE roman = ? AND hanzi = ?", (roman, hanzi))
        if cur.fetchone():
            skipped += 1
            continue

        conn_new.execute("INSERT INTO custom_dictionary (id, roman, hanzi, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                         (old_id, roman, hanzi, created_at, updated_at))
        # copy keys
        keys = conn_old.execute("SELECT family, form, key FROM custom_search_key WHERE entry_id = ?", (old_id,)).fetchall()
        for fam, form, key in keys:
            conn_new.execute("INSERT INTO custom_search_key (entry_id, family, form, key) VALUES (?, ?, ?, ?)",
                             (old_id, fam, form, key))
        inserted += 1

    conn_new.commit()
    conn_old.close()
    conn_new.close()
    print(f"[+] custom_dictionary.db merged: {inserted} inserted, {skipped} skipped")

def merge_learned_phrases():
    old_db = os.path.join(OLD_DIR, "learned_phrases.db")
    new_db = os.path.join(NEW_DIR, "learned_phrases.db")
    if not os.path.exists(old_db) or not os.path.exists(new_db):
        print(f"[-] learned_phrases.db skipped")
        return

    conn_old = sqlite3.connect(old_db)
    conn_new = sqlite3.connect(new_db)

    old_phrases = conn_old.execute("SELECT id, roman, hanzi, learn_count, updated_at FROM learned_phrases").fetchall()
    inserted = 0
    updated = 0

    for old_id, roman, hanzi, learn_count, updated_at in old_phrases:
        cur = conn_new.execute("SELECT id, learn_count, updated_at FROM learned_phrases WHERE hanzi = ? AND roman = ?", (hanzi, roman))
        row = cur.fetchone()
        if row:
            target_id, target_count, target_updated = row
            new_count = max(target_count, learn_count)
            new_updated = max(target_updated or '', updated_at or '')
            conn_new.execute("UPDATE learned_phrases SET learn_count = ?, updated_at = ? WHERE id = ?", (new_count, new_updated, target_id))
            updated += 1
        else:
            cur_ins = conn_new.execute("INSERT INTO learned_phrases (roman, hanzi, learn_count, updated_at) VALUES (?, ?, ?, ?)",
                                       (roman, hanzi, learn_count, updated_at))
            new_id = cur_ins.lastrowid
            # copy keys with new phrase_id
            keys = conn_old.execute("SELECT family, form, key FROM learned_search_key WHERE phrase_id = ?", (old_id,)).fetchall()
            for fam, form, key in keys:
                conn_new.execute("INSERT OR IGNORE INTO learned_search_key (phrase_id, family, form, key) VALUES (?, ?, ?, ?)",
                                 (new_id, fam, form, key))
            inserted += 1

    conn_new.commit()
    conn_old.close()
    conn_new.close()
    print(f"[+] learned_phrases.db merged: {inserted} inserted, {updated} updated")

if __name__ == '__main__':
    print(f"Merging from:\n  {OLD_DIR}\nTo:\n  {NEW_DIR}\n")
    merge_user_frequency()
    merge_user_association()
    merge_custom_dictionary()
    merge_learned_phrases()
    print("\nMerge completed successfully!")
