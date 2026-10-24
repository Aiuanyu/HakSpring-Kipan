"""產生企盤 Android 圖示（構想 3-1 方案 3A：大導角方圓鍵帽 × H/h 活水萌芽）。

來源（108×108 dp 原生設計，1 SVG 單位＝1 dp）：
- docs/hakfa/concept-3-1-kipan.svg       彩色。第一個 rect 是背景層，其餘是前景層
- docs/hakfa/concept-3-1-kipan-mono.svg  Android 13 主題圖示用單色（「h」和嫩芽挖空）

輸出（提交進 repo，不接進 Gradle）：
- mipmap-{mdpi,hdpi,xhdpi,xxhdpi,xxxhdpi}/ic_launcher_foreground.webp、ic_launcher_monochrome.webp
  各密度都直接從向量渲染（不是縮放點陣圖）
- 同上 ic_launcher.webp、ic_launcher_round.webp  舊版（Android 7 以下；minSdk 28 用不到）
- android/app/src/main/ic_launcher-playstore.png  512×512，滿版方形（Play 自己套遮罩）
- res/values/ic_launcher_background.xml
- docs/hakfa/android-icon-metrics.json  這次實際量到的數字（預覽圖和筆記引用）

需要：rsvg-convert（brew install librsvg）、Pillow、numpy。
在 repo 根目錄執行：python3 tools/hakfa/build-kipan-icons.py

安全區（Android 官方）：畫布 108 dp，桌面看得到中間 72 dp（圓形桌面半徑 36 dp）；
重要內容放在直徑 66 dp（半徑 33 dp）的圓內，任何桌面都不會切到。
產生時先量「離畫布中心最遠的墨跡」，兩段檢查，**任何一段超過就整支失敗、一個檔都不寫**：
- 本體（鍵帽、h、嫩芽、露珠，不含陰影）≤ SAFE_RADIUS_DP（33 dp）
- 全部（含陰影）≤ DECOR_RADIUS_DP（35.4 dp，在圓形桌面邊緣 36 dp 之內）
"""
import json
import os
import re
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
RES_DIR = os.path.join(REPO_ROOT, "android/app/src/main/res")
PLAY_STORE_PATH = os.path.join(REPO_ROOT, "android/app/src/main/ic_launcher-playstore.png")
BG_XML_PATH = os.path.join(REPO_ROOT, "android/app/src/main/res/values/ic_launcher_background.xml")
METRICS_PATH = os.path.join(REPO_ROOT, "docs/hakfa/android-icon-metrics.json")

SVG_FULL_PATH = os.path.join(REPO_ROOT, "docs/hakfa/concept-3-1-kipan.svg")
SVG_MONO_PATH = os.path.join(REPO_ROOT, "docs/hakfa/concept-3-1-kipan-mono.svg")

CANVAS_DP = 108
VISIBLE_DP = 72
SAFE_RADIUS_DP = 33.0
DECOR_RADIUS_DP = 35.4
INK_ALPHA_MIN = 16          # alpha 超過這個值才算墨跡
HI_PX = CANVAS_DP * 20      # 量測、舊版、Play 用的高解析度渲染（20 px/dp）
BRAND_BG_HEX = "#EAE3F1"    # 自適應圖示的背景層（純色）

BG_RECT = '<rect width="108" height="108" fill="url(#c31_bg)"/>'
SHADOW_RE = re.compile(r'<rect\b[^>]*fill="#0d1b15"[^>]*/>')  # 鍵帽底下的陰影（裝飾）

TMP = "/tmp/kipan_icons"
os.makedirs(TMP, exist_ok=True)


def render(svg_text, px, name):
    svg_path = f"{TMP}/{name}.svg"
    png_path = f"{TMP}/{name}_{px}.png"
    with open(svg_path, "w", encoding="utf-8") as f:
        f.write(svg_text)
    subprocess.run(["rsvg-convert", "-w", str(px), "-h", str(px), svg_path, "-o", png_path], check=True)
    return Image.open(png_path).convert("RGBA")


def farthest_ink_dp(img):
    """離畫布中心最遠的墨跡有多遠（dp）。"""
    px = img.size[0]
    alpha = np.array(img)[:, :, 3]
    ys, xs = np.where(alpha > INK_ALPHA_MIN)
    c = (px - 1) / 2.0
    return (np.hypot(xs - c, ys - c).max() + 0.5) * CANVAS_DP / px


def ink_width_dp(img):
    px = img.size[0]
    alpha = np.array(img)[:, :, 3]
    _, xs = np.where(alpha > INK_ALPHA_MIN)
    return (xs.max() - xs.min() + 1) * CANVAS_DP / px


def make_mask(size, shape):
    """4 倍超取樣遮罩。"""
    s = 4
    m = Image.new("L", (size * s, size * s), 0)
    d = ImageDraw.Draw(m)
    if shape == "circle":
        d.ellipse([0, 0, size * s - 1, size * s - 1], fill=255)
    else:  # rounded_rect，圓角 3/16
        d.rounded_rectangle([0, 0, size * s - 1, size * s - 1], radius=size * s * 3 / 16, fill=255)
    return m.resize((size, size), Image.Resampling.LANCZOS)


# 1. 讀 SVG、切出前景（拿掉背景 rect）、前景本體（再拿掉陰影）
with open(SVG_FULL_PATH, "r", encoding="utf-8") as f:
    svg_full = f.read()
with open(SVG_MONO_PATH, "r", encoding="utf-8") as f:
    svg_mono = f.read()
if BG_RECT not in svg_full:
    sys.exit("找不到背景 rect：concept-3-1-kipan.svg 的第一個 rect 改過了，請同步修改 BG_RECT")
svg_fg = svg_full.replace(BG_RECT, "")
shadows = SHADOW_RE.findall(svg_fg)
if len(shadows) != 1:
    sys.exit(f"預期恰好 1 個陰影 rect（fill=#0d1b15），找到 {len(shadows)} 個：請同步修改 SHADOW_RE")
svg_fg_body = svg_fg.replace(shadows[0], "")

# 2. 先量、通過才寫檔（失敗時不留下壞檔）
fg_all = render(svg_fg, HI_PX, "fg_all")
fg_body = render(svg_fg_body, HI_PX, "fg_body")
mono_hi = render(svg_mono, HI_PX, "mono")
metrics = {
    "ink_width_dp": round(ink_width_dp(fg_all), 2),
    "visible_ratio_pct": round(ink_width_dp(fg_all) / VISIBLE_DP * 100, 1),
    "essential_far_dp": round(farthest_ink_dp(fg_body), 2),
    "all_far_dp": round(farthest_ink_dp(fg_all), 2),
    "mono_far_dp": round(farthest_ink_dp(mono_hi), 2),
    "safe_radius_dp": SAFE_RADIUS_DP,
    "decor_limit_dp": DECOR_RADIUS_DP,
}
print(
    f"量測：鍵帽（含邊線）寬 {metrics['ink_width_dp']} dp＝可見 {VISIBLE_DP} dp 的 {metrics['visible_ratio_pct']}%；"
    f"本體最遠 {metrics['essential_far_dp']} dp（≤{SAFE_RADIUS_DP}）；含陰影最遠 {metrics['all_far_dp']} dp（≤{DECOR_RADIUS_DP}）；"
    f"單色層最遠 {metrics['mono_far_dp']} dp（≤{SAFE_RADIUS_DP}）"
)
problems = []
if metrics["essential_far_dp"] > SAFE_RADIUS_DP:
    problems.append(f"本體最遠 {metrics['essential_far_dp']} dp 超出安全圓半徑 {SAFE_RADIUS_DP} dp")
if metrics["mono_far_dp"] > SAFE_RADIUS_DP:
    problems.append(f"單色層最遠 {metrics['mono_far_dp']} dp 超出安全圓半徑 {SAFE_RADIUS_DP} dp")
if metrics["all_far_dp"] > DECOR_RADIUS_DP:
    problems.append(f"含陰影最遠 {metrics['all_far_dp']} dp 超出 {DECOR_RADIUS_DP} dp（桌面圓邊 36 dp）")
if problems:
    sys.exit("；".join(problems) + "。沒有寫任何檔：請調整 SVG（位置或大小）。")

# 3. 各密度：前景、單色層（向量直接渲染到該密度的像素）
densities = [("mdpi", 1), ("hdpi", 1.5), ("xhdpi", 2), ("xxhdpi", 3), ("xxxhdpi", 4)]
# 舊版與 Play：完整圖（背景＋前景）高解析度渲染，取中間桌面看得到的 72 dp
full_hi = render(svg_full, HI_PX, "full")
lo = (HI_PX - VISIBLE_DP * 20) // 2
visible_hi = full_hi.crop((lo, lo, lo + VISIBLE_DP * 20, lo + VISIBLE_DP * 20))

for folder, density in densities:
    target_dir = os.path.join(RES_DIR, f"mipmap-{folder}")
    os.makedirs(target_dir, exist_ok=True)
    fg_px = round(CANVAS_DP * density)       # 108、162、216、324、432
    leg_px = round(48 * density)             # 48、72、96、144、192

    render(svg_fg, fg_px, f"fg_{folder}").save(os.path.join(target_dir, "ic_launcher_foreground.webp"), "WEBP", lossless=True)
    render(svg_mono, fg_px, f"mono_{folder}").save(os.path.join(target_dir, "ic_launcher_monochrome.webp"), "WEBP", lossless=True)

    legacy = visible_hi.resize((leg_px, leg_px), Image.Resampling.LANCZOS)
    for name, shape in (("ic_launcher.webp", "rounded_rect"), ("ic_launcher_round.webp", "circle")):
        canvas = Image.new("RGBA", (leg_px, leg_px), (0, 0, 0, 0))
        canvas.paste(legacy, (0, 0), make_mask(leg_px, shape))
        canvas.save(os.path.join(target_dir, name), "WEBP", lossless=True)
    print(f"Generated {folder}: foreground+monochrome {fg_px}px, legacy {leg_px}px")

# 4. Play 商店 512 × 512：滿版方形（Play 自己套遮罩），內容與桌面看到的 72 dp 一致
visible_hi.resize((512, 512), Image.Resampling.LANCZOS).save(PLAY_STORE_PATH, "PNG")
print(f"Generated Play Store icon: {PLAY_STORE_PATH}")

# 5. 背景色（自適應圖示的背景層）
with open(BG_XML_PATH, "w", encoding="utf-8") as f:
    f.write(f"""<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">{BRAND_BG_HEX}</color>
</resources>
""")
print(f"Updated {BG_XML_PATH} to {BRAND_BG_HEX}")

# 6. 量測數字（預覽圖與筆記引用）
with open(METRICS_PATH, "w", encoding="utf-8") as f:
    json.dump(metrics, f, ensure_ascii=False, indent=2, sort_keys=True)
    f.write("\n")
print(f"Wrote {METRICS_PATH}")
