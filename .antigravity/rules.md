# `draw-rs` 專案架構規範與修改守則 (Project Architecture & Development Rules)

> [!IMPORTANT]
> 本文件為 `draw-rs` 專案的架構規範與防篡改守則。任何工程師或 AI 助理在修改、重構或擴充此專案時，**必須嚴格遵守**本文件中所訂定之架構邊界、SOLID 原則、Idiomatic Rust 慣用法與關鍵不變量（Invariants）。**嚴禁隨意打破分層或引入已廢棄/未經評估的依賴庫。**

---

## 1. 專案定位與技術選型 (Project Overview & Tech Stack)

- **目標定位**：專門在 macOS 上解演算法題目（如 LeetCode、系統設計面試、線上教學）使用的「全螢幕透明螢幕塗鴉板工具」。
- **Rust 規格**：Rust 2024 Edition (MSRV 1.85+)。
- **視窗與事件**：`winit` 0.30+（搭配 macOS extension）。
- **2D 向量光柵化**：`tiny-skia`（純 CPU 向量光柵化，零重型外部 GUI 框架依賴，維持獨立執行檔約 1.4 MB）。
- **macOS 底層互操作**：
  - **嚴格要求**：使用型別安全的現代庫 `objc2` (0.6+) 與 `objc2-app-kit` (0.3+)。
  - **嚴格禁止**：**嚴禁使用已棄用且缺乏維護的舊版 `cocoa` 或 `metal-rs` crate**。
- **全域熱鍵**：`global-hotkey`，在獨立背景執行緒中安全監聽 <kbd>F1</kbd>，即使視窗無焦點亦能全域切換模式。

---

## 2. 核心架構與目錄結構 (Directory Structure & Module Boundaries)

專案嚴格遵守 **單一職責原則 (SRP)** 與明確的分層邊界，各模組職責劃分如下：

```text
src/
├── app/                  # [Coordinator & State Machine] 應用程式協調者與狀態機
│   ├── mod.rs            # DrawApp（實作 winit::application::ApplicationHandler）
│   └── state.rs          # AppMode, DrawingTool, DragState, PaletteColor 狀態定義
├── controller/           # [Canvas Controller] 高階畫布歷程與 Undo 堆疊管理
│   ├── mod.rs
│   └── canvas.rs         # CanvasController & CanvasAction (Draw, Erase, Clear)
├── models/               # [Geometry & Math] 純幾何模型與碰撞演算法 (SRP, OCP, LSP)
│   ├── mod.rs            # 核心 Drawable Trait (draw, intersects)
│   ├── point.rs          # Point2D 向量計算 (距離、點到線段最短距離、中點、差值)
│   ├── stroke.rs         # StrokeShape (二次貝茲曲線平滑化、自由畫筆)
│   ├── circle.rs         # CircleShape (半透明節點、外框光柵化與射線檢測)
│   └── arrow.rs          # ArrowShape (動態指向箭頭、箭身與箭頭向量幾何)
├── platform/             # [Platform Interop] 原生作業系統與 AppKit 介面封裝 (DIP & ISP)
│   ├── mod.rs            # WindowPlatformController Trait & PlatformError
│   └── macos.rs          # MacosPlatformController (完全封裝所有 unsafe Objective-C 指針)
├── render/               # [Rasterization & UI] 像素光柵化與 HUD 繪製
│   ├── mod.rs
│   ├── font.rs           # 零相依 8x8 ASCII 向量點陣字體引擎
│   ├── hud.rs            # Glassmorphic 毛玻璃狀態懸浮面板光柵化
│   └── renderer.rs       # CanvasRenderer (tiny-skia Pixmap 雙緩衝區記憶體管理)
├── lib.rs                # 函式庫公開 API 介面導出
└── main.rs               # 主程式入口點、全域熱鍵背景監聽緒與 EventLoop 啟動
```

---

## 3. SOLID 物件導向原則與開發鐵則 (SOLID Principles)

### 3.1 單一職責原則 (Single Responsibility Principle, SRP)
- **`models` 模組**：
  - 僅負責幾何形狀的資料結構定義、向量數學計算與幾何碰撞檢測（`intersects`）。
  - **禁止**：引用 `winit`、視窗物件、作業系統事件或底層平台 API。
- **`render` 模組**：
  - 僅負責將抽象幾何資料與 HUD 面板光柵化為 `tiny-skia::Pixmap` 像素記憶體。
  - **禁止**：處理事件分發、歷史堆疊邏輯或視窗生命週期。
- **`platform` 模組**：
  - 專門負責 macOS 原生視窗樣式（無標題欄、穿透桌面、浮動層級 `NSFloatingWindowLevel`、`CALayer` 呈現）。
  - **禁止**：涉及任何畫布圖形邏輯或具體筆畫計算。
- **`controller` 模組**：
  - 專門負責管理歷史記錄（`CanvasAction` 堆疊）、提供 `push_shape`、`erase_at`、`undo` 與 `clear`。
  - **禁止**：直接呼叫視窗重繪或接觸 UI。

### 3.2 開放封閉原則 (Open/Closed Principle, OCP) 與里氏替換原則 (LSP)
- 所有畫布上的形狀必須實作 `Drawable` Trait：
  ```rust
  pub trait Drawable: Send + Sync {
      fn draw(&self, pixmap: &mut tiny_skia::Pixmap);
      fn intersects(&self, point: Point2D, radius: f32) -> bool;
  }
  ```
- **擴充規範**：新增圖形（例如矩形 `RectShape`、網格 `GridShape`、文字框 `TextShape`）時：
  1. 只能在 `src/models/` 新增對應結構並實作 `Drawable` Trait。
  2. **嚴禁** 修改 `CanvasController` 的核心堆疊架構。所有圖形必須以 `Box<dyn Drawable>` 多型管理。

### 3.3 介面隔離原則 (Interface Segregation Principle, ISP)
- 將底層平台控制抽像為最小必要的 `WindowPlatformController` Trait，不與繪圖、光柵化或事件處理產生肥大耦合。

### 3.4 依賴反轉原則 (Dependency Inversion Principle, DIP)
- 高階協調者（`DrawApp`）依賴 `WindowPlatformController` Trait 與 `dyn Drawable` 抽象介面，不依賴具體 macOS AppKit 內部實作。
- **所有 `unsafe` Objective-C 指針操作必須徹底封閉在 `src/platform/macos.rs` 內部**，絕不允許外洩到其他模組。

---

## 4. 關鍵功能與不可破壞之不變量 (Critical Invariants - DO NOT BREAK)

在維護或擴充任何功能時，以下幾點為本系統之基石，**切勿更動或破壞**：

### 4.1 原生零黑邊真透明背景 (True Alpha = 0 Transparency)
- **不可破壞原因**：
  - 舊版/常規 `softbuffer` (0.4.8) 在 macOS 上硬編碼了 `CGImageAlphaInfo::NoneSkipFirst`，會將 Alpha = 0 像素強制填為不透明全黑底色，導致螢幕全黑看不見底下的程式碼。
- **鐵則**：
  - 繪圖輸出必須維持呼叫 `MacosPlatformController::present_pixmap`。
  - 內部必須透過 `CGImage` 搭配 `CGImageAlphaInfo::PremultipliedLast`，直接輸出至 `NSView.layer`（`CALayer`），由 macOS Quartz 視窗管理器進行硬體加速透明混合合成。
  - **嚴禁** 隨意更換回未支援自訂 Alpha 的第三方 Framebuffer blitter。

### 4.2 全域滑鼠穿透與 HUD 動態拖移 (Click-Through & Draggable HUD)
- **不可破壞機制**：
  1. **穿透模式（Click-Through Mode）**：
     - 調用 `NSWindow.setIgnoresMouseEvents(true)`，使用者的所有點擊均穿透至背景的 IDE、瀏覽器或終端機。
  2. **穿透模式下拖移 HUD（Draggable HUD）**：
     - 在 `about_to_wait` 中，以 60 FPS 輪詢 `platform.get_global_cursor_pos()`（透過免權限且極高效能的 `NSEvent::mouseLocation()`）。
     - 當游標移動至 HUD 面板矩形（`HudOverlay::contains_point`）時，系統會**動態切換** `setIgnoresMouseEvents(false)` 並將滑鼠游標設為 `Grab`，允許使用者按住拖移面板。
     - 游標移出 HUD 時，自動無縫切換回 `setIgnoresMouseEvents(true)`。
  - **嚴禁** 移除此輪詢與動態切換機制，否則穿透模式下 HUD 將失去響應或無法移動。

### 4.3 向量橡皮擦與完全可逆 Undo (Vector Eraser & Reversible History)
- **不可破壞機制**：
  - 橡皮擦並非使用位圖 Mask 破壞像素，而是基於向量碰撞檢測（`Drawable::intersects`）。
  - `CanvasController` 必須使用 `CanvasAction::Erase(Vec<(usize, Box<dyn Drawable>)>)` 保存被擦除圖形及其原始位置索引。
  - 按下 <kbd>Z</kbd>（Undo）時，必須能將被擦除的多個形狀無損還原至原始順序。

---

## 5. 程式碼規範與品質檢查 (Code Quality & Verification)

在發起任何 PR 或提交 Commit 之前，必須在專案根目錄通過以下所有檢查：

1. **單元測試 (Unit Tests)**：
   ```bash
   cargo test
   ```
   所有核心幾何運算、橡皮擦碰撞、歷程堆疊與狀態切換測試必須 100% 通過。

2. **靜態分析 (Clippy)**：
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
   必須維持 **0 warnings**。

3. **程式碼格式化 (Formatting)**：
   ```bash
   cargo fmt -- --check
   ```
   所有代碼風格必須符合標準 `rustfmt` 規則。

4. **發行版編譯 (Release Build)**：
   ```bash
   cargo build --release
   ```
   二進位輸出為 `target/release/draw-rs`，且不得引入非預期的大型外部依賴。
