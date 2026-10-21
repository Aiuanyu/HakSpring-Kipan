---
description: 撰寫 Git commit message，討論好之後幫忙執行 commit 或任何 Git 操作
---

## 要點
- **【鐵則】必須使用臺灣客家語（Hakfa）撰寫**：使用四縣／海陸等共通詞彙（如「个」、「摎」、「在」、「肚底／裡肚」、「單淨」、「無共樣」、「毋使」等）。嚴禁誤用臺灣話／閩南語詞彙（如「干焦」、「無仝」、「佮」、「佇」、「親像」等）或純華語長篇文言。
- 先分析專案 Git 目前 staged 的 diff，歸納此次更新內容
- 依對話記憶提及不同 AI 模型，分別列出每個 AI 模型的分工及貢獻
- 先寫在專案根目錄的臨時 `commit_msg.txt` 中，供平哥審閱、直接編輯
- 平哥審核完成告知定稿後，再根據 `commit_msg.txt` 執行 commit
    - 絕對要小心避免那種會卡住終端機狀態的指令
    - 愛細膩莫 commit 著 `commit_msg.txt`；commit 煞後，暫存檔案愛提掉（清理）

## 貢獻者樣板
```
Co-authored-by: Opus 4.6 <Opus4.6@Antigravity>
Co-authored-by: Sonnet 4.6 <Sonnet4.6@Antigravity>
Co-authored-by: Gemini 3.1 Pro <Gemini-3-1-Pro@Antigravity>
Signed-off-by: GHSRobert <robertus0617@gmail.com> (Commit Helper: Gemini 3 Flash <Gemini-3-Flash@Antigravity>)
```
