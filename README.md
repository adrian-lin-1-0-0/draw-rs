# draw-rs: macOS LeetCode Transparent Doodle Overlay

> A lightweight, blazing-fast, transparent screen annotation and doodle board for macOS, specifically tailored for solving algorithm and data-structure problems (e.g., LeetCode, Codeforces, system design).

Built with **Rust 2024 Edition** adhering strictly to **SOLID Principles** and **Idiomatic Rust**.

---

## 🌟 Features

1. **Native Frameless & Transparent Overlay**:
   - Alpha = 0 transparent background.
   - Pinned at `NSStatusWindowLevel` (stays reliably on top of all browser and IDE windows).
   - Multi-space persistence (`CanJoinAllSpaces | FullScreenAuxiliary`).

2. **macOS Mouse Click-Through Toggle (F1)**:
   - Powered by modern, type-safe `objc2-app-kit` calling `NSWindow.setIgnoresMouseEvents:`.
   - **Drawing Mode**: Draw binary tree nodes, arrows, and freehand annotations.
   - **Click-Through Mode**: Keep diagrams suspended on screen while clicking and typing directly in your IDE, terminal, or LeetCode web page.
   - Global hotkey listener monitors `F1` in a background thread to seamlessly switch back even when the window is unfocused.

3. **LeetCode Dedicated Drawing Tools (F2)**:
   - **Pen**: Smooth freehand ink with quadratic Bézier curve interpolation.
   - **Circle**: Node drawing with translucent fill (25% opacity) and crisp outlines (ideal for Binary Trees and Graph nodes).
   - **Arrow**: Directional arrows with vector arrowheads (ideal for Two Pointers, Linked List `next` pointers, and directed edges).
   - **Real-Time Live Preview**: Shapes resize dynamically as you drag.

4. **Curated Algorithm Color Palette (Keys 1-5)**:
   - `1`: Cyan (`#38BDF8`) - High contrast default
   - `2`: Emerald (`#34D399`) - Left pointer / visited nodes
   - `3`: Coral (`#F87171`) - Right pointer / target / pivot
   - `4`: Amber (`#FBBF24`) - Highlight / slow pointer
   - `5`: Violet (`#C084FC`) - Special / aux nodes

5. **Glassmorphic Status HUD**:
   - Built-in zero-dependency raster bitmap font engine.
   - Displays real-time operational mode, current tool, active color, shape count, and keybinding cheatsheet.

6. **History & Control**:
   - `Z`: Undo last shape.
   - `C`: Clear all doodles.
   - `Esc`: Exit application.

---

## 🏗️ SOLID Architecture

```text
src/
├── app/                  # Application Controller & State Machine
│   ├── mod.rs            # DrawApp (winit ApplicationHandler<AppEvent>)
│   └── state.rs          # AppMode, DrawingTool, DragState, PaletteColor
├── controller/           # High-Level Canvas Management
│   ├── mod.rs
│   └── canvas.rs         # History stack (Vec<Box<dyn Drawable>>), undo, clear
├── models/               # Geometric Data & Math (SRP)
│   ├── mod.rs            # Drawable trait (OCP & LSP)
│   ├── point.rs          # Point2D geometry utilities
│   ├── stroke.rs         # Freehand stroke with quadratic Bézier smoothing
│   ├── circle.rs         # Tree/Graph node with translucent fill
│   └── arrow.rs          # Vector directional arrow with arrowheads
├── platform/             # Platform Abstraction & macOS Interop (DIP & ISP)
│   ├── mod.rs            # WindowPlatformController trait
│   └── macos.rs          # Modern objc2 & objc2-app-kit implementation
├── render/               # Rasterization & Surface Presentation (SRP)
│   ├── mod.rs
│   ├── font.rs           # Zero-dependency 8x8 ASCII font rasterizer
│   ├── hud.rs            # Translucent glassmorphic status card
│   └── renderer.rs       # tiny-skia Pixmap & softbuffer ARGB blitter
├── lib.rs                # Public library exports
└── main.rs               # Application entry point & background hotkey thread
```

- **Single Responsibility Principle (SRP)**:
  - `models`: Only geometry and math; completely independent of windows and events.
  - `render`: Only rasterization to `Pixmap` and buffer blitting.
  - `platform`: Strictly encapsulates macOS AppKit interop.
  - `controller`: Only canvas history and shape composition.
  - `app`: Coordinates events and delegates state transitions.

- **Open/Closed Principle (OCP) & Liskov Substitution Principle (LSP)**:
  - `pub trait Drawable: Send + Sync { fn draw(&self, pixmap: &mut tiny_skia::Pixmap); }`
  - Adding new shapes (e.g., `Grid`, `TextRect`) requires only implementing `Drawable`. The canvas and rendering pipeline remain untouched.
  - Unified history container: `Vec<Box<dyn Drawable>>`.

- **Interface Segregation Principle (ISP)**:
  - `WindowPlatformController`: Isolated window behavior interface (`configure_overlay`, `set_click_through`, `activate_window`).
  - Separated from drawing and canvas management.

- **Dependency Inversion Principle (DIP)**:
  - High-level canvas manager depends on `dyn Drawable` abstractions.
  - Application depends on `WindowPlatformController` trait, keeping unsafe Objective-C calls completely confined inside `platform::macos`.

---

## 🚀 Quick Start

### Prerequisites
- macOS (Apple Silicon 或 Intel)
- Rust 1.85+ (Edition 2024)

### Build & Run

```bash
# 開發除錯執行
cargo run

# 生產最佳化執行
cargo run --release

# 執行單元與幾何光柵化整合測試
cargo test
```

---

## 🛡️ macOS 系統權限配置指南 (Permissions Guide)

### 1. 為什麼先前執行時會看到「全黑背景」？（底層原理修復）
- **原因**：跨平台的 `softbuffer` 在 macOS 後端 (`softbuffer/src/backends/cg.rs`) 內部硬編碼了 `CGImageAlphaInfo::NoneSkipFirst`。這會強制忽略 Alpha 通道，導致透明區域（RGB=0,0,0, Alpha=0）被 Core Graphics 當作「100% 不透明的純黑」渲染至視窗。
- **解決方案**：本專案已在 `platform::macos` 模組透過現代型別安全庫 `objc2-app-kit`、`objc2-core-graphics` 與 `objc2-quartz-core`，直接將 `tiny-skia` 緩衝區對接原生 `CALayer`，並宣告 `CGImageAlphaInfo::PremultipliedLast`。藉由 macOS 系統級 Quartz Compositor 合成，**視窗背景達成 100% 真正穿透透明，完全告別黑畫面！**

---

### 2. macOS 權限配置步驟

在 macOS（尤其是 macOS 14 Sonoma 與 macOS 15 Sequoia）上，系統對**全域快捷鍵監聽**與**視窗浮動穿透**有嚴格的安全權限管控。請依照以下指引檢查或配置權限：

#### A. 輔助使用權限 (Accessibility) —— 【必備：確保全域 F1 快捷鍵在任何視窗下皆可切換】
當您在「滑鼠穿透模式」下，點擊底層的 VS Code 或 Chrome 瀏覽器敲代碼時，`draw-rs` 視窗會失去輸入焦點。為了讓背景執行緒能夠隨時攔截全域 `F1` 按鍵將您切回繪圖模式，執行程式的終端機或 binary 需要輔助使用權限：
1. 打開 **系統設定 (System Settings)** ➔ **隱私權與安全性 (Privacy & Security)**。
2. 點選 **輔助使用 (Accessibility)**。
3. 點擊 `+` 按鈕，加入您使用的終端機（如 **Terminal / 終端機**、**iTerm**、**Ghostty**、**VS Code**）或編譯出的 `target/release/draw-rs` 執行檔，並確保右側開關保持 **開啟 (ON)**。
4. *快速開啟快捷鍵指令*：在終端機輸入：
   ```bash
   open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
   ```

#### B. 輸入監控權限 (Input Monitoring) —— 【選用：部分 macOS 版本的全域監聽需求】
如果您的 macOS 系統有開啟深度的安全防護，全域鍵盤事件可能需要額外授權「輸入監控」：
1. 前往 **系統設定** ➔ **隱私權與安全性** ➔ **輸入監控 (Input Monitoring)**。
2. 檢查您所使用的終端機是否已啟用權限。
3. *快速開啟快捷鍵指令*：
   ```bash
   open "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
   ```

#### C. 螢幕錄製權限 (Screen Recording) —— 【說明：塗鴉板本體無須偷取螢幕畫面】
- **重要概念**：`draw-rs` 是「原生透明浮動視窗 (Native Alpha = 0)」，它直接懸浮在您的螢幕上方，而不是靠「截圖/擷取底層螢幕像素」來假裝透明。因此 **`draw-rs` 本身完全不需要、也不會讀取您的螢幕畫面隱私**！
- **何時需要螢幕錄製權限？**：
  - 如果您希望在解題時配合 **OBS**、**Zoom 螢幕共享** 或 **CleanShot / 系統截圖快捷鍵 (Cmd+Shift+4)** 將您的塗鴉板筆記連同底層 LeetCode 題目一起截圖或錄影，則該**錄影/截圖軟體**需要獲得「螢幕錄製」授權：
    ```bash
    open "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
    ```

---

## ⌨️ Controls Summary

| 快捷鍵 | 動作 | 功能說明 |
| :--- | :--- | :--- |
| **`F1`** | **切換模式** | 於「繪圖模式」與「穿透模式」間切換（全域生效） |
| **`F2`** | **切換工具** | 循環切換：`Pen (畫筆)` ➔ `Circle (節點)` ➔ `Arrow (指針)` |
| **`1` ~ `5`** | **切換顏色** | `1: 天藍 (Cyan)`、`2: 翠綠 (Emerald)`、`3: 珊瑚紅 (Coral)`、`4: 琥珀金 (Amber)`、`5: 紫羅蘭 (Violet)` |
| **`Z`** | **復原 (Undo)** | 移除最近一次繪製的圖形 |
| **`C`** | **清除 (Clear)** | 清空畫布上的所有圖形 |
| **`Esc`** | **退出** | 關閉程式並退出 |
