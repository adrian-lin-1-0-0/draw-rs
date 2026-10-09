# `draw-rs` Project Architecture Rules & Development Guidelines

> [!IMPORTANT]
> This document defines the architectural contract, design standards, and constraints for the `draw-rs` project. Any developer or AI coding assistant modifying, refactoring, or extending this codebase **must strictly comply** with the architectural boundaries, SOLID principles, Idiomatic Rust patterns, and critical invariants outlined herein. **Do not violate module boundaries or introduce deprecated/unvetted dependencies.**

---

## 1. Project Overview & Tech Stack

- **Purpose**: A full-screen transparent screen doodle overlay designed specifically for solving algorithm problems (e.g., LeetCode, system design interviews, whiteboard coding presentations) on macOS.
- **Language Specification**: Rust 2024 Edition (MSRV 1.85+).
- **Windowing & Events**: `winit` 0.30+ (leveraging macOS extension).
- **2D Vector Rasterization**: `tiny-skia` (pure-CPU 2D vector rasterization, zero heavy external GUI framework dependencies, standalone executable size of ~1.4 MB).
- **macOS Native Interoperability**:
  - **Strict Requirement**: Use type-safe modern libraries `objc2` (0.6+) and `objc2-app-kit` (0.3+).
  - **Strictly Forbidden**: **Never use deprecated, unmaintained `cocoa` or `metal-rs` crates**.
- **Global Hotkey**: `global-hotkey`, safely listened in a dedicated background thread for <kbd>F1</kbd>, enabling mode switching even when the application is unfocused.

---

## 2. Architecture & Directory Structure

The project strictly follows the **Single Responsibility Principle (SRP)** with explicit module boundaries:

```text
locales/                 # [Localization Resources] Compile-time embedded YAML locale definitions
├── en.yaml               # English strings dictionary
└── zh-TW.yaml            # Traditional Chinese (繁體中文) strings dictionary
src/
├── app/                  # [Coordinator & State Machine] Application coordinator and state machine
│   ├── mod.rs            # DrawApp (implements winit::application::ApplicationHandler)
│   ├── i18n.rs           # AppLanguage localization system (deserializes embedded YAML)
│   └── state.rs          # AppMode, DrawingTool, DragState, PaletteColor definitions
├── controller/           # [Canvas Controller] High-level canvas history and undo stack
│   ├── mod.rs
│   └── canvas.rs         # CanvasController & CanvasAction (Draw, Erase, Clear)
├── models/               # [Geometry & Math] Pure geometric models and hit-testing (SRP, OCP, LSP)
│   ├── mod.rs            # Core Drawable trait (draw, intersects)
│   ├── point.rs          # Point2D vector math (distance, point-to-segment distance, midpoint, lerp)
│   ├── stroke.rs         # StrokeShape (freehand ink with quadratic Bézier curve smoothing)
│   ├── circle.rs         # CircleShape (translucent nodes, raycasting and border hit-testing)
│   └── arrow.rs          # ArrowShape (dynamic directional arrows, vector shaft and head calculations)
├── platform/             # [Platform Interop] Native OS and AppKit abstraction (DIP & ISP)
│   ├── mod.rs            # WindowPlatformController trait & PlatformError
│   └── macos.rs          # MacosPlatformController (encapsulates all unsafe Objective-C pointers)
├── render/               # [Rasterization & UI] Pixel rasterization and HUD rendering
│   ├── mod.rs
│   ├── font.rs           # Dynamic macOS CJK system font rasterizer (fontdue) with bitmap fallback
│   ├── hud.rs            # Glassmorphic floating HUD rasterizer
│   └── renderer.rs       # CanvasRenderer (tiny-skia Pixmap memory and buffer management)
├── lib.rs                # Library crate root and public exports
└── main.rs               # Application entry point, global hotkey background thread, EventLoop runner
```

---

## 3. SOLID Principles & Coding Standards

### 3.1 Single Responsibility Principle (SRP)
- **`models` module**:
  - Exclusively responsible for geometry data structures, vector math calculations, and hit-testing (`intersects`).
  - **Forbidden**: Referencing `winit`, window handles, operating system events, or platform APIs.
- **`render` module**:
  - Exclusively responsible for rasterizing abstract geometry and HUD components into `tiny-skia::Pixmap` pixel memory.
  - **Forbidden**: Handling event dispatching, undo history logic, or window lifecycles.
- **`platform` module**:
  - Exclusively responsible for native macOS window behavior (borderless, desktop pass-through, `NSFloatingWindowLevel`, `CALayer` blitting).
  - **Forbidden**: Touching canvas drawing shapes, color palettes, or tool state.
- **`controller` module**:
  - Exclusively responsible for managing drawing history (`CanvasAction` stack), offering `push_shape`, `erase_at`, `undo`, and `clear`.
  - **Forbidden**: Directly triggering window redraw requests or rendering UI elements.

### 3.2 Open/Closed Principle (OCP) & Liskov Substitution Principle (LSP)
- All shapes drawn on the canvas must implement the `Drawable` trait:
  ```rust
  pub trait Drawable: Send + Sync {
      fn draw(&self, pixmap: &mut tiny_skia::Pixmap);
      fn intersects(&self, point: Point2D, radius: f32) -> bool;
  }
  ```
- **Extension Rules**: When introducing new shape types (e.g., `RectShape`, `GridShape`, `TextShape`):
  1. Only add the new struct and implement `Drawable` within `src/models/`.
  2. **Do not** modify the core stack management in `CanvasController`. All shapes must be polymorphically managed via `Box<dyn Drawable>`.

### 3.3 Interface Segregation Principle (ISP)
- The platform abstraction (`WindowPlatformController`) is kept lean, exposing only the minimal windowing operations needed by the application without monolithic coupling.

### 3.4 Dependency Inversion Principle (DIP)
- The high-level coordinator (`DrawApp`) depends strictly on abstractions (`WindowPlatformController` trait and `dyn Drawable`), rather than concrete macOS AppKit implementations.
- **All `unsafe` Objective-C pointer operations must be strictly confined within `src/platform/macos.rs`** and never leak into outer layers.

---

## 4. Critical Invariants (DO NOT BREAK)

When maintaining or extending any feature, the following architectural invariants are fundamental:

### 4.1 Native Zero-Alpha True Transparency (Quartz Compositing)
- **Why this invariant exists**:
  - Standard `softbuffer` (0.4.8) on macOS hardcodes `CGImageAlphaInfo::NoneSkipFirst`, forcing pixels with Alpha = 0 to render as opaque black, thereby blocking the screen content underneath.
- **Invariant Rules**:
  - Canvas presentation must route through `MacosPlatformController::present_pixmap`.
  - The implementation must create a `CGImage` with `CGImageAlphaInfo::PremultipliedLast` and blit directly to `NSView.layer` (`CALayer`), allowing the macOS Quartz Compositor to achieve true zero-alpha hardware transparency.
  - **Never** replace this with an unvetted framebuffer blitter that lacks custom alpha channel support.

### 4.2 Global Click-Through & Draggable HUD
- **Why this invariant exists**:
  1. **Click-Through Mode**:
     - `NSWindow.setIgnoresMouseEvents(true)` allows user clicks to pass through freely to background applications (VSCode, browsers, terminals).
  2. **Repositioning HUD in Click-Through Mode**:
     - In `about_to_wait`, the application polls `platform.get_global_cursor_pos()` at 60 FPS using the zero-permission, ultra-fast `NSEvent::mouseLocation()`.
     - When the cursor moves over the HUD card bounds (`HudOverlay::contains_point`), the application **dynamically toggles** `setIgnoresMouseEvents(false)` and switches the cursor icon to `Grab`, allowing the user to click and drag the HUD card anywhere on screen.
     - When the cursor leaves the HUD area, click-through is seamlessly re-enabled (`setIgnoresMouseEvents(true)`).
  - **Do not** remove or alter this dynamic hover-toggle mechanism, as it ensures the HUD remains interactive even during pass-through mode.

### 4.3 Vector Eraser & Reversible History (Undo)
- **Why this invariant exists**:
  - The eraser operates via vector geometric hit-testing (`Drawable::intersects`), rather than raster masking.
  - `CanvasController` must preserve erased shapes and their original indices in `CanvasAction::Erase(Vec<(usize, Box<dyn Drawable>)>)`.
  - Pressing <kbd>Z</kbd> (Undo) must restore all erased shapes back to their exact original layer order.

---

## 5. Code Quality & Verification Gates

Before submitting any Pull Request or committing changes, all of the following checks must pass in the repository root:

1. **Unit Tests**:
   ```bash
   cargo test
   ```
   All geometric math, hit-testing, undo history, and state transitions must pass 100%.

2. **Static Analysis (Clippy)**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
   Must maintain **0 warnings**.

3. **Code Formatting**:
   ```bash
   cargo fmt -- --check
   ```
   All source code must strictly comply with standard `rustfmt` formatting.

4. **Release Compilation**:
   ```bash
   cargo build --release
   ```
   The output binary `target/release/draw-rs` must compile cleanly without unexpected external dependencies.
