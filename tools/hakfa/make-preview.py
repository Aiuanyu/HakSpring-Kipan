import math
import os
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter

# -------------------------------------------------------------
# Canvas & Theme Configuration
# -------------------------------------------------------------
WIDTH = 2000
HEIGHT = 1250

BG_DARK = (13, 16, 23, 255)         # Deep graphite #0D1017
PANEL_BG = (21, 26, 35, 255)        # Premium card background #151A23
PANEL_BORDER = (42, 51, 68, 255)    # Card border #2A3344

TEXT_WHITE = (245, 247, 250, 255)
TEXT_MUTED = (140, 150, 168, 255)
TEXT_DARK_MUTED = (95, 108, 128, 255)

BRAND_GREEN = (36, 93, 72, 255)     # HakSpring Ink #245D48
BRAND_BG = (234, 227, 241, 255)      # Tile Lavender #EAE3F1 (淡吊菜老鼠色)
ACCENT_CYAN = (88, 166, 255, 255)   # #58A6FF
ACCENT_GOLD = (240, 180, 41, 255)   # Safe zone gold #F0B429
ACCENT_GREEN = (56, 210, 140, 255)  # Success emerald

# -------------------------------------------------------------
# Font Management
# -------------------------------------------------------------
FONT_HEITI = "/System/Library/Fonts/STHeiti Medium.ttc"
FONT_HEITI_LIGHT = "/System/Library/Fonts/STHeiti Light.ttc"
FONT_MONO = "/System/Library/Fonts/SFNSMono.ttf"
FONT_SANS = "/System/Library/Fonts/SFNS.ttf"

def get_font(size, kind="heiti"):
    try:
        if kind == "heiti_bold":
            return ImageFont.truetype(FONT_HEITI, size)
        elif kind == "heiti_light":
            return ImageFont.truetype(FONT_HEITI_LIGHT, size)
        elif kind == "mono":
            return ImageFont.truetype(FONT_MONO, size)
        elif kind == "sans":
            return ImageFont.truetype(FONT_SANS, size)
        else:
            return ImageFont.truetype(FONT_HEITI, size)
    except:
        return ImageFont.load_default()

font_header_title = get_font(34, "heiti_bold")
font_header_sub = get_font(18, "heiti_light")
font_badge = get_font(13, "heiti_bold")
font_card_title = get_font(21, "heiti_bold")
font_card_sub = get_font(14, "heiti_light")
font_item_title = get_font(16, "heiti_bold")
font_item_code = get_font(12, "mono")
font_item_sub = get_font(12, "heiti_light")
font_body = get_font(14, "heiti")
font_caption = get_font(12, "mono")

# -------------------------------------------------------------
# Load Icon Assets
# -------------------------------------------------------------
fg_path = "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher_foreground.webp"
legacy_sq_path = "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.webp"
legacy_rd_path = "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher_round.webp"
play_path = "android/app/src/main/ic_launcher-playstore.png"

mono_path = "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher_monochrome.webp"

im_fg = Image.open(fg_path).convert("RGBA")        # 432x432
im_mono = Image.open(mono_path).convert("RGBA")    # 432x432，主題圖示用（h 與嫩芽挖空）
im_legacy_sq = Image.open(legacy_sq_path).convert("RGBA")
im_legacy_rd = Image.open(legacy_rd_path).convert("RGBA")
im_play = Image.open(play_path).convert("RGBA")

# 數字由 tools/hakfa/build-kipan-icons.py 產生圖檔時實際量測，寫在 JSON，這裡只讀、不寫死
import json
with open("docs/hakfa/android-icon-metrics.json", "r", encoding="utf-8") as _f:
    _M = json.load(_f)
INK_W_DP = _M["ink_width_dp"]            # 鍵帽（含邊線）寬
INK_RATIO = _M["visible_ratio_pct"]      # 佔桌面看得到的 72 dp
INK_FAR_DP = _M["essential_far_dp"]      # 本體（不含陰影）離中心最遠
ALL_FAR_DP = _M["all_far_dp"]            # 含陰影
MONO_FAR_DP = _M["mono_far_dp"]          # 單色層

# -------------------------------------------------------------
# Smooth Mask & Blending Engine
# -------------------------------------------------------------
def make_mask(size, shape="circle"):
    scale = 4
    sw, sh = size[0] * scale, size[1] * scale
    mask = Image.new("L", (sw, sh), 0)
    draw = ImageDraw.Draw(mask)
    
    if shape == "circle":
        draw.ellipse([0, 0, sw - 1, sh - 1], fill=255)
    elif shape == "squircle":
        r = sw / 2.0
        n = 4.2
        y, x = np.ogrid[:sh, :sw]
        dx = np.abs((x - r + 0.5) / (r - 1))
        dy = np.abs((y - r + 0.5) / (r - 1))
        dist = dx**n + dy**n
        arr = np.clip((1.0 - (dist - 0.98) / 0.04) * 255, 0, 255).astype(np.uint8)
        mask = Image.fromarray(arr, mode="L")
        return mask.resize(size, Image.Resampling.LANCZOS)
    elif shape == "rounded_rect":
        rad = sw * (3.0 / 16.0)
        draw.rounded_rectangle([0, 0, sw - 1, sh - 1], radius=rad, fill=255)
    elif shape == "teardrop":
        draw.pieslice([0, 0, sw - 1, sh - 1], start=90, end=360, fill=255)
        draw.rounded_rectangle([sw//2, sh//2, sw - 1, sh - 1], radius=sw*0.14, fill=255)
    elif shape == "square":
        draw.rectangle([0, 0, sw - 1, sh - 1], fill=255)
        
    return mask.resize(size, Image.Resampling.LANCZOS)

def alpha_composite_at(base, overlay, pos):
    ox, oy = pos
    ow, oh = overlay.size
    crop = base.crop((ox, oy, ox + ow, oy + oh))
    comp = Image.alpha_composite(crop, overlay)
    base.paste(comp, (ox, oy))

def draw_floating_icon(base, icon, pos, blur=18, offset_y=12):
    w, h = icon.size
    pad = blur * 2 + offset_y
    
    s_canvas = Image.new("RGBA", (w + pad * 2, h + pad * 2), (0, 0, 0, 0))
    alpha = icon.split()[-1]
    
    # 1. Key shadow
    key_s = Image.new("RGBA", (w, h), (0, 0, 0, 140))
    key_s.putalpha(alpha)
    s_canvas.paste(key_s, (pad, pad + offset_y), key_s)
    
    # 2. Ambient shadow
    amb_s = Image.new("RGBA", (w, h), (0, 0, 0, 90))
    amb_s.putalpha(alpha)
    s_canvas.paste(amb_s, (pad, pad + 4), amb_s)
    
    s_canvas = s_canvas.filter(ImageFilter.GaussianBlur(blur))
    alpha_composite_at(base, s_canvas, (pos[0] - pad, pos[1] - pad))
    alpha_composite_at(base, icon, pos)

# -------------------------------------------------------------
# Generate Styled Icons
# -------------------------------------------------------------
def get_adaptive_icon(shape="circle", size=150):
    full = Image.new("RGBA", (432, 432), BRAND_BG)
    full.paste(im_fg, (0, 0), im_fg)
    
    # 桌面實際看得到的範圍是中間 72 dp（288 px），不是安全區的 66 dp：圓形桌面的直徑是 72 dp
    cropped = full.crop((72, 72, 360, 360)).resize((size, size), Image.Resampling.LANCZOS)
    mask = make_mask((size, size), shape=shape)
    
    res = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    res.paste(cropped, (0, 0), mask)
    
    # Subtle inner rim
    rim = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    r_draw = ImageDraw.Draw(rim)
    if shape == "circle":
        r_draw.ellipse([1, 1, size - 2, size - 2], outline=(255, 255, 255, 40), width=1)
    elif shape == "rounded_rect":
        r_draw.rounded_rectangle([1, 1, size - 2, size - 2], radius=size*(3.0/16.0), outline=(255, 255, 255, 40), width=1)
    res = Image.alpha_composite(res, rim)
    return res

def get_material_you_icon(bg_color, fg_color, shape="circle", size=130):
    fg_alpha = im_mono.split()[-1]  # 專用單色層：鍵帽實心、h 與嫩芽挖空
    tinted_fg = Image.new("RGBA", im_mono.size, fg_color)
    tinted_fg.putalpha(fg_alpha)
    
    full = Image.new("RGBA", (432, 432), bg_color)
    full.paste(tinted_fg, (0, 0), tinted_fg)
    
    cropped = full.crop((72, 72, 360, 360)).resize((size, size), Image.Resampling.LANCZOS)
    mask = make_mask((size, size), shape=shape)
    
    res = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    res.paste(cropped, (0, 0), mask)
    return res

# -------------------------------------------------------------
# Blueprint Generator
# -------------------------------------------------------------
def get_blueprint_view(target_size=330):
    bp = Image.new("RGBA", (432, 432), (16, 20, 27, 255))
    draw = ImageDraw.Draw(bp)
    cx, cy = 216, 216
    
    # Grid lines inside blueprint
    for g in range(36, 432, 36):
        draw.line([g, 0, g, 432], fill=(25, 33, 46, 255), width=1)
        draw.line([0, g, 432, g], fill=(25, 33, 46, 255), width=1)
        
    # 108 dp Canvas Border
    draw.rectangle([3, 3, 428, 428], outline=(55, 80, 115, 255), width=2)
    
    # Diagonals & Center crosshair
    draw.line([0, 0, 432, 432], fill=(45, 60, 85, 180), width=1)
    draw.line([0, 432, 432, 0], fill=(45, 60, 85, 180), width=1)
    draw.line([cx, 0, cx, 432], fill=(70, 95, 135, 200), width=1)
    draw.line([0, cy, 432, cy], fill=(70, 95, 135, 200), width=1)
    
    # 72 dp Viewport Circle (r=144)
    draw.ellipse([cx - 144, cy - 144, cx + 144, cy + 144], outline=(75, 95, 125, 220), width=1)
    
    # 66 dp Safe Zone Circle (r=132 px) - Gold
    draw.ellipse([cx - 132, cy - 132, cx + 132, cy + 132], outline=ACCENT_GOLD, width=2)
    
    # Overlay the foreground glyph / keycap
    bp.paste(im_fg, (0, 0), im_fg)
    
    # Callout annotations
    draw.text((cx - 128, cy - 156), "66 dp Safe Zone (r = 33.0 dp)", fill=ACCENT_GOLD, font=font_caption)
    draw.text((cx - 128, cy + 138), f"Body {INK_FAR_DP:.1f} <= 33.0 | shadow {ALL_FAR_DP:.1f}", fill=ACCENT_GREEN, font=font_caption)
    draw.text((10, 8), "108 dp Canvas (432 px)", fill=(100, 130, 170, 255), font=font_caption)
    
    return bp.resize((target_size, target_size), Image.Resampling.LANCZOS)

# -------------------------------------------------------------
# Construct Master Canvas
# -------------------------------------------------------------
master = Image.new("RGBA", (WIDTH, HEIGHT), BG_DARK)
draw = ImageDraw.Draw(master)

# Ambient subtle dot matrix in background
for x in range(25, WIDTH, 35):
    for y in range(25, HEIGHT, 35):
        draw.rectangle([x, y, x+1, y+1], fill=(40, 50, 70, 80))

# -------------------------------------------------------------
# Header Section
# -------------------------------------------------------------
header_x, header_y = 60, 40
draw.text((header_x, header_y), "HakSpring Kipan 客源翠 / 企盤", fill=TEXT_WHITE, font=font_header_title)
draw.text((header_x, header_y + 46), "構想 3-1 方案 3A（大導角方圓鍵帽）：鍵帽 × H/h 活水萌芽 · Launcher 自適應幾何全覽", fill=TEXT_MUTED, font=font_header_sub)

# Badges
badges = [
    ("意象：鍵帽 × H/h 活水（方案 3A）", (220, 205, 235, 255), (42, 35, 55, 255)),
    ("自適應畫布：108 dp（432 × 432 px）", (190, 210, 230, 255), (32, 42, 56, 255)),
    (f"鍵帽 {INK_W_DP:.0f} dp，佔可見範圍 {INK_RATIO:.0f}%", ACCENT_GOLD, (50, 42, 18, 255)),
    (f"安全區：本體 {INK_FAR_DP:.1f} dp ≤ 33，含陰影 {ALL_FAR_DP:.1f} dp < 36", ACCENT_GREEN, (18, 48, 35, 255)),
    ("客源翠 5 色調色盤體系完整融入", ACCENT_CYAN, (20, 42, 70, 255)),
]

bx = header_x
by = header_y + 82
for text, fg, bg in badges:
    bbox = font_badge.getbbox(text)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    pw, ph = 12, 6
    draw.rounded_rectangle([bx, by, bx + tw + pw * 2, by + th + ph * 2], radius=6, fill=bg, outline=(fg[0], fg[1], fg[2], 80), width=1)
    draw.text((bx + pw, by + ph - 1), text, fill=fg, font=font_badge)
    bx += tw + pw * 2 + 12

def draw_card(x, y, w, h, title, sub):
    draw.rounded_rectangle([x, y, x + w, y + h], radius=16, fill=PANEL_BG, outline=PANEL_BORDER, width=1)
    draw.text((x + 26, y + 22), title, fill=TEXT_WHITE, font=font_card_title)
    draw.text((x + 26, y + 50), sub, fill=TEXT_MUTED, font=font_card_sub)

# =============================================================
# Panel 1: Adaptive Launcher Masks (Top Left, w=1120, h=395)
# =============================================================
p1_x, p1_y, p1_w, p1_h = 60, 180, 1120, 395
draw_card(p1_x, p1_y, p1_w, p1_h, "自適應桌面遮罩（Adaptive Launcher Masks）", f"鍵帽佔可見範圍約 {INK_RATIO:.0f}%，本體貼著 33 dp 安全圓、陰影不超出桌面圓邊，主流 Android 桌面零裁切")

masks_data = [
    ("原生圓形（Circle）", "circle", "Pixel · AOSP · Motorola", f"桌面圓邊半徑 36 dp；含陰影最遠 {ALL_FAR_DP:.1f} dp"),
    ("方圓超橢圓（Squircle）", "squircle", "Samsung One UI", "Superellipse n=4.2 滿版"),
    ("經典圓角方（Rounded Square）", "rounded_rect", "ColorOS · HyperOS · macOS", "圓角半徑 3/16 大氣大方"),
    ("水滴形（Teardrop）", "teardrop", "Nova · 智慧啟動器", "非對稱有機弧線"),
]

spacing_p1 = p1_w / 4
icon_y_p1 = p1_y + 92
for i, (name, shape, launcher, note) in enumerate(masks_data):
    icon_x = p1_x + int(spacing_p1 * i + (spacing_p1 - 150) / 2)
    icon = get_adaptive_icon(shape=shape, size=150)
    draw_floating_icon(master, icon, (icon_x, icon_y_p1), blur=16, offset_y=12)
    
    # Text labels
    cx = icon_x + 75
    bb1 = font_item_title.getbbox(name)
    draw.text((cx - (bb1[2]-bb1[0])//2, icon_y_p1 + 172), name, fill=TEXT_WHITE, font=font_item_title)
    bb2 = font_item_code.getbbox(launcher)
    draw.text((cx - (bb2[2]-bb2[0])//2, icon_y_p1 + 200), launcher, fill=ACCENT_CYAN, font=font_item_code)
    bb3 = font_item_sub.getbbox(note)
    draw.text((cx - (bb3[2]-bb3[0])//2, icon_y_p1 + 220), note, fill=TEXT_MUTED, font=font_item_sub)

# =============================================================
# Panel 2: Engineering Blueprint & Geometry (Top Right, w=700, h=510)
# =============================================================
p2_x, p2_y, p2_w, p2_h = 1240, 180, 700, 510
draw_card(p2_x, p2_y, p2_w, p2_h, "幾何工程藍圖與安全區檢驗", "方案 3A（108 dp 原生設計）安全區驗證")

bp_view = get_blueprint_view(target_size=330)
draw_floating_icon(master, bp_view, (p2_x + 32, p2_y + 90), blur=14, offset_y=8)

spec_items = [
    ("自適應全畫布 (Adaptive Canvas)", "108 dp (432 × 432 px)"),
    ("安全區直徑 (Safe Zone Ø)", "66.0 dp (半徑 33.0 dp)"),
    ("鍵帽寬度 (Keycap Width)", f"{INK_W_DP:.1f} dp 含邊線（可見 72 dp 的 {INK_RATIO:.0f}%）"),
    ("最遠墨跡 (Farthest Ink)", f"本體 {INK_FAR_DP:.1f}、含陰影 {ALL_FAR_DP:.1f}、單色 {MONO_FAR_DP:.1f} dp"),
    ("圓形遮罩安全 (Circle Safety)", "本體 ≤ 33 dp、含陰影 ≤ 35.4 dp，超出就讓腳本失敗"),
    ("品牌調色盤融入 (HakSpring Palette)", "深蒼綠·初萌若芽·朝霧薄綠·薰紫"),
    ("符號語言深度 (Icon Metaphor)", "鍵帽 × 活水 × H/h 客語羅馬字"),
    ("光柵化輸出 (Rasteriser Engine)", "向量 rsvg-convert 轉無損 WebP"),
]

sx = p2_x + 388
sy = p2_y + 90
for lbl, val in spec_items:
    draw.text((sx, sy), lbl, fill=TEXT_DARK_MUTED, font=font_item_sub)
    col = ACCENT_GREEN if ("100%" in val or "滿版" in val) else TEXT_WHITE
    if "Safe Zone" in lbl:
        col = ACCENT_GOLD
    draw.text((sx, sy + 16), val, fill=col, font=font_body)
    sy += 44

# =============================================================
# Panel 3: Material You Dynamic Theming (Bottom Left, w=1120, h=470)
# =============================================================
p3_x, p3_y, p3_w, p3_h = 60, 605, 1120, 470
draw_card(p3_x, p3_y, p3_w, p3_h, "Material You 動態主題色適配（Android 13+）", "單色向量 Alpha 圖層自動隨系統桌布 Monet 色盤動態調色")

themes_data = [
    ("淺色莫蘭迪綠（Sage Light）", (216, 234, 220, 255), (25, 62, 45, 255), "#D8EADC / #193E2D", "清新草木色系"),
    ("深色墨翠暗夜（Forest Dark）", (22, 38, 30, 255), (145, 232, 186, 255), "#16261E / #91E8BA", "護眼深邃高對比"),
    ("淺色薰衣草紫（Lavender Light）", (236, 226, 246, 255), (66, 46, 86, 255), "#ECE2F6 / #422E56", "客源翠品牌同調"),
    ("深色紫夜極光（Nightfall Dark）", (30, 26, 38, 255), (212, 196, 238, 255), "#1E1A26 / #D4C4EE", "典雅沉靜暗色"),
]

spacing_p3 = p3_w / 4
icon_y_p3 = p3_y + 95
for i, (t_name, bg_c, fg_c, hex_c, desc) in enumerate(themes_data):
    icon_x = p3_x + int(spacing_p3 * i + (spacing_p3 - 130) / 2)
    t_icon = get_material_you_icon(bg_c, fg_c, shape="squircle" if i % 2 == 1 else "circle", size=130)
    draw_floating_icon(master, t_icon, (icon_x, icon_y_p3), blur=14, offset_y=10)
    
    cx = icon_x + 65
    bb1 = font_item_title.getbbox(t_name)
    draw.text((cx - (bb1[2]-bb1[0])//2, icon_y_p3 + 152), t_name, fill=TEXT_WHITE, font=font_item_title)
    bb2 = font_item_code.getbbox(hex_c)
    draw.text((cx - (bb2[2]-bb2[0])//2, icon_y_p3 + 178), hex_c, fill=ACCENT_CYAN, font=font_item_code)
    bb3 = font_item_sub.getbbox(desc)
    draw.text((cx - (bb3[2]-bb3[0])//2, icon_y_p3 + 198), desc, fill=TEXT_MUTED, font=font_item_sub)

draw.text((p3_x + 30, p3_y + 424), "• 系統規範：Android 13+ 主題圖示透過 res/mipmap-anydpi-v26/ic_launcher.xml 讀取專用單色層 ic_launcher_monochrome（鍵帽實心、h 與嫩芽挖空），由系統自動著色。", fill=TEXT_MUTED, font=font_item_sub)

# =============================================================
# Panel 4: Distribution & Legacy Fallback (Bottom Right, w=700, h=410)
# =============================================================
p4_x, p4_y, p4_w, p4_h = 1240, 720, 700, 410
draw_card(p4_x, p4_y, p4_w, p4_h, "商店發行與歷史版本相容支援", "Google Play 商店滿版圖示與 Android 7- 傳統圖示")

# 1. Play Store
play_mini = im_play.resize((120, 120), Image.Resampling.LANCZOS)
draw_floating_icon(master, play_mini, (p4_x + 46, p4_y + 90), blur=14, offset_y=10)
draw.text((p4_x + 38, p4_y + 230), "Google Play 商店", fill=TEXT_WHITE, font=font_item_title)
draw.text((p4_x + 38, p4_y + 252), "512 × 512 px 滿版方形", fill=ACCENT_GOLD, font=font_item_code)
draw.text((p4_x + 38, p4_y + 270), "構想 3-1 滿版（商店自動圓角）", fill=TEXT_MUTED, font=font_item_sub)

# 2. Legacy Square
leg_sq_mini = im_legacy_sq.resize((120, 120), Image.Resampling.LANCZOS)
draw_floating_icon(master, leg_sq_mini, (p4_x + 252, p4_y + 90), blur=14, offset_y=10)
draw.text((p4_x + 242, p4_y + 230), "舊版圓角方（Legacy Sq.）", fill=TEXT_WHITE, font=font_item_title)
draw.text((p4_x + 242, p4_y + 252), "48 dp（圓角比例 3/16）", fill=TEXT_MUTED, font=font_item_code)
draw.text((p4_x + 242, p4_y + 270), "Android 7.1 及以下備援", fill=TEXT_MUTED, font=font_item_sub)

# 3. Legacy Round
leg_rd_mini = im_legacy_rd.resize((120, 120), Image.Resampling.LANCZOS)
draw_floating_icon(master, leg_rd_mini, (p4_x + 468, p4_y + 90), blur=14, offset_y=10)
draw.text((p4_x + 464, p4_y + 230), "舊版圓形（Legacy Round）", fill=TEXT_WHITE, font=font_item_title)
draw.text((p4_x + 464, p4_y + 252), "48 dp（100% 正圓形）", fill=TEXT_MUTED, font=font_item_code)
draw.text((p4_x + 464, p4_y + 270), "Android 7.1 及以下備援", fill=TEXT_MUTED, font=font_item_sub)

# Mobile Dock Simulation Bar
dock_w, dock_h = 600, 76
dock_x = p4_x + (p4_w - dock_w) // 2
dock_y = p4_y + 314
draw.rounded_rectangle([dock_x, dock_y, dock_x + dock_w, dock_y + dock_h], radius=26, fill=(28, 35, 48, 220), outline=(52, 65, 88, 255), width=1)

def draw_phone_icon(size=52):
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=13, fill=(52, 199, 89, 255))
    d.arc([16, 14, 36, 36], start=100, end=260, fill=(255, 255, 255, 255), width=4)
    d.ellipse([14, 28, 22, 36], fill=(255, 255, 255, 255))
    d.ellipse([24, 14, 32, 22], fill=(255, 255, 255, 255))
    return im

def draw_sms_icon(size=52):
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=13, fill=(0, 122, 255, 255))
    d.rounded_rectangle([12, 14, 40, 34], radius=6, fill=(255, 255, 255, 255))
    d.polygon([(16, 34), (16, 40), (24, 34)], fill=(255, 255, 255, 255))
    return im

def draw_browser_icon(size=52):
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=13, fill=(255, 149, 0, 255))
    d.ellipse([14, 14, 38, 38], outline=(255, 255, 255, 255), width=3)
    d.line([14, 26, 38, 26], fill=(255, 255, 255, 255), width=2)
    return im

dock_apps = [
    ("電話", draw_phone_icon(), "phone"),
    ("客語輸入法", get_adaptive_icon("squircle", size=52), "kipan"),
    ("訊息", draw_sms_icon(), "sms"),
    ("瀏覽器", draw_browser_icon(), "web")
]

for idx, (label, icon_img, kind) in enumerate(dock_apps):
    ix = dock_x + 42 + idx * 140
    iy = dock_y + 9
    alpha_composite_at(master, icon_img, (ix, iy))
    
    bb = font_item_sub.getbbox(label)
    lw = bb[2] - bb[0]
    draw.text((ix + 26 - lw//2, iy + 56), label, fill=TEXT_WHITE if kind == "kipan" else TEXT_MUTED, font=font_item_sub)

# -------------------------------------------------------------
# Footer
# -------------------------------------------------------------
footer_text = "由 tools/hakfa/make-preview.py 產生 | 客源翠 企盤（HakSpring Kipan）構想 3-1（芽形貼合版）全規格設計規範"
bb_f = font_caption.getbbox(footer_text)
draw.text((WIDTH//2 - (bb_f[2]-bb_f[0])//2, HEIGHT - 30), footer_text, fill=TEXT_DARK_MUTED, font=font_caption)

# Save Master
out_path = "docs/hakfa/android-icon-preview.png"
master.save(out_path, format="PNG", optimize=True)
print(f"Successfully generated new showcase at: {out_path} ({WIDTH}x{HEIGHT})")
