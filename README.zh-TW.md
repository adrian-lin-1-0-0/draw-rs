# 🖋️ draw-rs: macOS & Linux 全螢幕透明演算法解題塗鴉板

<div align="center">

<p>
  <a href="./README.md">English</a> | <b>繁體中文</b>
</p>

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![CI](https://github.com/adrian-lin-1-0-0/draw-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/adrian-lin-1-0-0/draw-rs/actions/workflows/ci.yml)
[![Platform](https://img.shields.io/badge/Platform-macOS_%7C_Linux-black)](https://github.com/adrian-lin-1-0-0/draw-rs)
[![Architecture](https://img.shields.io/badge/Architecture-SOLID_&_Idiomatic-blue)](https://en.wikipedia.org/wiki/SOLID)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**專為 macOS 與 Linux 使用者打造的高效能透明螢幕塗鴉工具**  
專門輔助在 **LeetCode 刷題**、**系統架構設計**、**白板編程面試** 與 **線上教學演練** 時即時繪製資料結構！

<br />

[實機展示](#-實機展示-demo-preview) •
[快速啟動](#-安裝與快速啟動) •
[為什麼需要 draw-rs？](#-為什麼需要-draw-rs) •
[核心功能](#-核心功能特點) •
[快捷鍵操作](#-快捷鍵與操作指南) •
[快捷啟動設定](#-全域快捷鍵與快速啟動設定指南) •
[系統與權限設定](#-系統配置與透明度指南) •
[軟體架構 (SOLID)](#-solid-軟體架構) •
[開源授權](#-開源授權-license)

</div>

---

## 📸 實機展示 (Demo Preview)

<div align="center">
  <img src="./assets/demo.png" alt="draw-rs 實機展示截圖" width="850" />
  <p><em>即時在 LeetCode 題目與編輯器上方進行透明向量標記與資料結構解說</em></p>
</div>

---

## 🚀 安裝與快速啟動

### 安裝方式

#### 方式一：透過 Cargo 一鍵安裝（推薦）

直接從 GitHub 安裝二進位執行檔至系統的 `~/.cargo/bin`：

```bash
cargo install --git https://github.com/adrian-lin-1-0-0/draw-rs.git
```

安裝完成後，在任何終端機視窗輸入指令即可立刻啟動：
```bash
draw-rs
```

#### 方式二：從原始碼編譯執行

```bash
# 1. 複製專案
git clone https://github.com/adrian-lin-1-0-0/draw-rs.git
cd draw-rs

# 2. 執行單元與光柵化整合測試
cargo test

# 3. 編譯並執行生產最佳化版本
cargo run --release
```

產生的獨立可執行二進位檔位於：
```bash
./target/release/draw-rs
```

---

## 💡 為什麼需要 draw-rs？

在 LeetCode 或線上評測解題時，傳統的演算法草稿紙或外部白板繪圖軟體需要頻繁切換視窗，打斷思考。

`draw-rs` 直接在您的螢幕上方懸浮一個 **100% 原生透明的向量畫布**：
- 🌲 **直接在 LeetCode 題目上方畫樹狀圖**（二元樹 Binary Tree、平衡樹 AVL）。
- 🔀 **標註雙指針與滑動窗口**（清晰標示 `left`、`right` 索引位移）。
- 🔗 **繪製鏈結串列與有向圖節點**（動態箭頭 `curr -> next`）。
- ⚡ **一秒切換「滑鼠穿透模式」(<kbd>F1</kbd>)**：畫完指針後，一鍵穿透回 VS Code / 瀏覽器寫 code，塗鴉依然清晰懸浮在螢幕上供你對照！

---

## 🌟 核心功能特點

| 功能特點 | 技術實現與優勢 |
| :--- | :--- |
| **原生透明無邊框** | 基於 macOS `NSWindow` 與 `CALayer` (Quartz 合成) 及 Linux `softbuffer` (X11 / Wayland 合成)，背景 Alpha = 0，無任何黑邊或延遲。 |
| **全域滑鼠穿透開關** | 透過原生作業系統視窗屬性 (`setIgnoresMouseEvents:` / `set_cursor_hittest`)，搭配獨立背景執行緒捕捉全域 <kbd>F1</kbd>，無焦點時依然能秒切模式。 |
| **互動式懸浮功能面板 HUD** | 毛玻璃半透明面板，提供點選切換工具、色票調色盤、快速操作（Undo/Clear/穿透），所有模式下皆可隨意拖曳位置。 |
| **穿透模式下依然可拖曳面板** | 即時座標碰撞偵測（Hit-testing），即便處於滑鼠穿透模式，滑鼠移至 HUD 卡片上方仍可靈活拖曳，不干擾底層應用程式操作。 |
| **演算法解題必備幾何圖形** | 手繪筆 (<kbd>P</kbd>)、半透明節點圓圈 (<kbd>Circle</kbd>)、動態指針箭頭 (<kbd>Arrow</kbd>) 與向量橡皮擦 (<kbd>E</kbd>)。 |
| **精心調配高對比色票** | 專為深色/淺色程式編輯器設計的高辨識度配色（青綠、翠綠、珊瑚橘、琥珀金、紫羅蘭）。 |
| **極致輕量零負擔** | 純 CPU 2D 向量光柵化引擊 (`tiny-skia`)，記憶體佔用極低，獨立可執行檔僅約 1.4 MB。 |

---

## ⌨️ 快捷鍵與操作指南

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ ● DRAWING (3 shapes)               [Pass-Thru F1]                     ::: Drag  │
│ ─────────────────────────────────────────────────────────────────────────────── │
│ TOOLS: [Pen P] [Circle] [Arrow] [Eraser E]        [Undo Z]  [Clear C]           │
│ ─────────────────────────────────────────────────────────────────────────────── │
│ COLOR: (●) (●) (●) (●) (●)   [1-5]                                 [繁體中文 ▼]  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

| 按鍵 / 滑鼠操作 | 功能動作 | 說明 |
| :---: | :--- | :--- |
| **點擊 HUD 介面按鈕** | **選擇工具 / 顏色 / 操作** | 直接以滑鼠在 HUD 卡片上點擊切換工具、色彩、復原、清除或穿透模式 |
| **語言下拉選單** | **切換介面語言** | 點擊 HUD 上的 `English ▼` 或 `繁體中文 ▼` 展開選單並即時切換語系 |
| **拖曳 HUD 面板** | **移動懸浮視窗位置** | 按住 HUD 面板頂部或背景可自由拖曳（在「繪圖模式」與「穿透模式」下皆能拖曳！） |
| <kbd>F1</kbd> | **切換模式（全域熱鍵）** | 切換 **「繪圖模式」** ↔ **「滑鼠穿透模式」**（無論焦點在哪均可觸發） |
| <kbd>F2</kbd> | **輪播切換工具** | 依序輪播：手繪筆 ➔ 圓圈（節點） ➔ 箭頭（指標） ➔ 橡皮擦 |
| <kbd>E</kbd> | **切換至橡皮擦** | 直接切換為 **橡皮擦**，游標呈現圓圈半徑範圍，精準擦除幾何線段 |
| <kbd>P</kbd> | **切換至手繪筆** | 直接切換回 **手繪筆 (Pen)** |
| <kbd>1</kbd> ~ <kbd>5</kbd> | **切換調色盤顏色** | 依序切換：青綠、翠綠、珊瑚橘、琥珀金、紫羅蘭色票 |
| <kbd>L</kbd> | **快速切換語言** | 鍵盤快速鍵切換介面語言 (**English** / **繁體中文**) |
| <kbd>Z</kbd> | **復原 (Undo)** | 可逆復原繪製筆畫，亦可復原誤擦除的線段與圖形 |
| <kbd>C</kbd> | **清除全螢幕塗鴉** | 一鍵清空畫布上所有塗鴉與標記 |
| <kbd>Esc</kbd> | **關閉選單 / 退出程式** | 收合展開的下拉選單，或安全關閉並退出 `draw-rs` |

---

## ⚡ 全域快捷鍵與快速啟動設定指南

### 1. MacBook 頂部功能鍵 (<kbd>F1</kbd> / <kbd>fn</kbd> + <kbd>F1</kbd>) 設定

在 Apple 鍵盤或 MacBook 內建鍵盤上，最上排按鍵預設為硬體多媒體控制（螢幕亮度、音量、指揮中心）：
- **預設按法**：請按下 <kbd>fn</kbd> + <kbd>F1</kbd> 切換「繪圖模式」與「滑鼠穿透模式」。
- **設為標準功能鍵（推薦）**：
  1. 打開 macOS **系統設定 (System Settings)** ➔ **鍵盤 (Keyboard)** ➔ **鍵盤快速鍵 (Keyboard Shortcuts)** ➔ **功能鍵 (Function Keys)**。
  2. 開啟 **「將 F1、F2 等按鍵用作標準功能鍵」**。
  3. 設定後，即可直接單鍵按下 <kbd>F1</kbd> 切換模式，不需再長按 <kbd>fn</kbd> 鍵！

### 2. 設定全域快捷鍵一鍵喚起 `draw-rs`

推薦設定全域熱鍵（例如 <kbd>⌥</kbd> + <kbd>D</kbd> 或 <kbd>⌘</kbd> + <kbd>⇧</kbd> + <kbd>D</kbd>），讓你在刷 LeetCode 或寫程式時，隨時一鍵呼叫塗鴉板：

#### 推薦方式 A：使用 Raycast / Alfred（工程師最推薦）
- **Raycast**：
  1. 打開 **Raycast Settings** ➔ **Extensions** ➔ **Script Commands**。
  2. 新增一個指向 `~/.cargo/bin/draw-rs` 的腳本或 Quicklink。
  3. 為其指派熱鍵（例如 <kbd>⌥</kbd> + <kbd>D</kbd>）。
- **Alfred**：
  1. 打開 **Alfred Preferences** ➔ **Workflows** ➔ 新增空白 Workflow。
  2. 建立 **Hotkey** 觸發器 ➔ 連接至 **Run Script** (`~/.cargo/bin/draw-rs`)。

#### 推薦方式 B：macOS 原生「捷徑」App (Shortcuts)
1. 打開 macOS 內建的 **捷徑 (Shortcuts)** App，點擊 `+` 新增捷徑。
2. 加入動作：**執行 Shell 工序指令 (Run Shell Script)**。
3. 填入指令：
   ```bash
   ~/.cargo/bin/draw-rs &
   ```
4. 點擊右側面板的「捷徑詳細資訊」，勾選 **「用作快速動作」**，並點擊 **「加入鍵盤快速鍵」**（例如 <kbd>⌘</kbd> + <kbd>⇧</kbd> + <kbd>D</kbd>）。

#### 推薦方式 C：`skhd`（平鋪式視窗管理器使用者）
在 `~/.config/skhd/skhdrc` 加入以下設定：
```text
cmd + shift - d : ~/.cargo/bin/draw-rs
```

---

## 🛡️ 系統配置與透明度指南

### 1. 為什麼背景能真正 100% 透明穿透？
- **macOS（Quartz 原生合成）**：`draw-rs` 採用原生視窗透明屬性（Native Alpha = 0），透過 `objc2-app-kit`、`objc2-core-graphics` 與 `objc2-quartz-core` 直接將 `tiny-skia` 預乘 Alpha 像素緩衝區 (`PremultipliedLast`) 交付 macOS 系統級 **Quartz Compositor** 合成，徹底免除傳統 cross-platform buffer 函式庫產生的全黑畫面瑕疵。
- **Linux（合成器零黑邊渲染）**：透過 `softbuffer` 與 `winit` 將預乘 Alpha 像素即時呈現至 X11 / Wayland 視窗。在啟用合成管理器（如 X11 上的 Picom 或原生 Wayland 合成器）環境下可達成 100% 硬體混合透明度。
- **純浮動不偷取隱私**：`draw-rs` 是純粹懸浮於所有視窗上方的畫布，**完全不需要、也不會抓取您的螢幕畫面**。

---

### 2. 系統權限檢查與配置步驟 (macOS)

在 macOS（尤其是 macOS 14 Sonoma 與 macOS 15 Sequoia）上，全域快捷鍵與視窗穿透需要對應的安全權限：

#### A. 輔助使用權限 (Accessibility) —— 【必備】
當切換為「滑鼠穿透模式」後，滑鼠會點擊下方的 VS Code 或瀏覽器，此時 `draw-rs` 視窗會處於無焦點狀態。為了讓背景執行緒能夠攔截全域 <kbd>F1</kbd> 按鍵切回繪圖模式：
1. 打開 **系統設定 (System Settings)** ➔ **隱私權與安全性 (Privacy & Security)**。
2. 點選 **輔助使用 (Accessibility)**。
3. 點擊 `+` 按鈕，加入您使用的終端機（如 **Terminal / 終端機**、**iTerm**、**Ghostty**、**VS Code**）或編譯出的 `draw-rs` 執行檔，並確認開關為 **開啟 (ON)**。
4. *終端機快捷指令*：
   ```bash
   open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
   ```

#### B. 輸入監控權限 (Input Monitoring) —— 【選用】
若您的 macOS 有開啟深度安全策略，全域鍵盤監聽可能需要授權：
```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
```

#### C. 螢幕錄製權限 (Screen Recording) —— 【選用】
若您在 LeetCode 刷題時希望搭配 **OBS**、**Zoom 螢幕分享** 或 **系統截圖快捷鍵 (Cmd+Shift+4)** 將螢幕筆記連同底層題目一同錄製，請確保該**第三方錄影/截圖軟體**具備螢幕錄製授權：
```bash
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
```

---

## 🏗️ SOLID 軟體架構

`draw-rs` 嚴格遵循物件導向 **SOLID 原則** 與 **Idiomatic Rust** 慣用語設計：

```mermaid
graph TD
    UserEvent[使用者操作 / 全域熱鍵 F1] --> App[DrawApp 主控制器]
    App --> StateMachine[AppMode 與工具狀態機]
    App --> Controller[CanvasController 歷程管理器]
    Controller --> Trait[<< trait >>\nDrawable]
    Trait --> Stroke[StrokeShape: 貝茲曲線自由筆畫]
    Trait --> Circle[CircleShape: 半透明資料結構節點]
    Trait --> Arrow[ArrowShape: 向量指針與箭頭]
    App --> Renderer[CanvasRenderer: tiny-skia 向量光柵化]
    App --> Platform[WindowPlatformController 平台抽象]
    Platform --> MacosPlatform[MacosPlatformController: objc2 & CALayer]
    Platform --> LinuxPlatform[LinuxPlatformController: softbuffer X11/Wayland]
```

### 模組目錄職責劃分

```text
src/
├── app/                  # Application 控制器與狀態機
│   ├── mod.rs            # DrawApp (實作 winit 0.30 ApplicationHandler)
│   ├── i18n.rs           # 多國語系即時字典
│   └── state.rs          # AppMode、DrawingTool、DragState、PaletteColor
├── controller/           # 高階畫布歷程管理器
│   ├── mod.rs
│   └── canvas.rs         # CanvasController (Vec<Box<dyn Drawable>> 復原/清除佇列)
├── models/               # 幾何數學與數據結構 (SRP, OCP, LSP)
│   ├── mod.rs            # 核心 Drawable Trait 定義
│   ├── point.rs          # Point2D 向量計算 (距離、角度、中點、插值)
│   ├── stroke.rs         # Pen 自由筆畫 (二階貝茲 Bézier 平滑曲線光柵化)
│   ├── circle.rs         # Circle 節點 (半透明填色 + 銳利外框)
│   └── arrow.rs          # Arrow 箭頭 (動態角度與三角形箭頭光柵化)
├── platform/             # 平台抽象與 OS 原生封裝 (DIP & ISP)
│   ├── mod.rs            # WindowPlatformController Trait & PlatformError
│   ├── keymap.rs         # PlatformKeymap Trait、MacosKeymap 與 LinuxKeymap
│   ├── macos.rs          # MacosPlatformController (完全封裝 unsafe Objective-C 指針操作)
│   ├── linux.rs          # LinuxPlatformController (softbuffer X11/Wayland 光柵呈現與視窗狀態)
│   └── dummy.rs          # DummyPlatformController (無頭測試降級實作)
├── render/               # 向量光柵化緩衝區與 UI 繪製 (SRP)
│   ├── mod.rs
│   ├── font.rs           # CJK 向量字體 (fontdue) 與 8x8 ASCII 點陣字體
│   ├── hud.rs            # 毛玻璃狀態面板 HUD 繪製
│   └── renderer.rs       # tiny-skia Pixmap 記憶體管理
├── lib.rs                # 函式庫公開介面
└── main.rs               # 主程式入口與全域熱鍵監聽背景執行緒
```

### SOLID 具體實踐

1. **單一職責原則 (Single Responsibility Principle, SRP)**:
   - `models`：僅定義純幾何數學結構，不接觸視窗與事件。
   - `render`：僅負責將向量幾何數據光柵化至像素記憶體。
   - `platform`：專注於原生 macOS AppKit 與 Linux X11/Wayland 視窗圖層配置。
   - `controller`：僅負責歷史紀錄堆疊管理與組合。
2. **開放封閉原則 (Open/Closed Principle, OCP) 與里氏替換原則 (LSP)**:
   - 核心定義抽象 Trait：
     ```rust
     pub trait Drawable: Send + Sync {
         fn draw(&self, pixmap: &mut tiny_skia::Pixmap);
     }
     ```
   - 擴充新圖形（例如網格 Grid、文字框 Text）只需新增 struct 並實作 `Drawable`，無須更動現有繪圖邏輯與歷程控制器。
   - 歷程容器統一管理 `Vec<Box<dyn Drawable>>`。
3. **介面隔離原則 (Interface Segregation Principle, ISP)**:
   - 平台視窗控制 (`WindowPlatformController`) 與畫布繪製職責徹底隔離，無肥大介面。
4. **依賴反轉原則 (Dependency Inversion Principle, DIP)**:
   - 高階畫布管理器僅依賴 `dyn Drawable` 抽象介面。
   - 應用程式依賴 `WindowPlatformController` 介面，將所有 unsafe 系統底層呼叫完全限制在 `platform` 模組內部。

---

## 📄 開源授權 (License)

本專案採用 [MIT License](LICENSE) 授權條款開源發布。
