pub mod i18n;
pub mod state;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tiny_skia::{Color, Paint, PathBuilder, Stroke, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowId};

use crate::controller::CanvasController;
use crate::models::{Drawable, point::Point2D};
use crate::platform::WindowPlatformController;
use crate::render::{CanvasRenderer, HudHitTarget, HudOverlay, HudRenderParams};
pub use i18n::AppLanguage;
use state::{AppMode, DragState, DrawingTool, PaletteColor};

/// Custom application event triggered by global hotkeys or background threads.
#[derive(Debug, Clone)]
pub enum AppEvent {
    ToggleMode,
}

/// The top-level application coordinator managing window lifecycle, user interaction,
/// rendering pipeline, and state transitions.
///
/// Adheres to:
/// - SRP: Coordinates event dispatching without directly performing low-level rendering or OS calls.
/// - DIP: Interacts with the platform layer strictly via `Box<dyn WindowPlatformController>`.
pub struct DrawApp {
    platform: Box<dyn WindowPlatformController>,
    canvas: CanvasController,
    renderer: Option<CanvasRenderer>,
    window: Option<Arc<Window>>,
    mode: AppMode,
    tool: DrawingTool,
    color: PaletteColor,
    pub language: AppLanguage,
    stroke_width: f32,
    eraser_radius: f32,
    drag_state: DragState,
    cursor_pos: Point2D,
    scale_factor: f64,
    hud_pos: Point2D,
    is_click_through_active: bool,
}

impl DrawApp {
    pub fn new(platform: Box<dyn WindowPlatformController>) -> Self {
        Self {
            platform,
            canvas: CanvasController::new(),
            renderer: None,
            window: None,
            mode: AppMode::Drawing,
            tool: DrawingTool::Pen,
            color: PaletteColor::Cyan,
            language: AppLanguage::default(),
            stroke_width: 3.5,
            eraser_radius: 18.0,
            drag_state: DragState::Idle,
            cursor_pos: Point2D::ZERO,
            scale_factor: 1.0,
            hud_pos: Point2D::new(24.0, 24.0),
            is_click_through_active: false,
        }
    }

    /// Toggle between active Drawing mode and passive Click-Through mode.
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.toggle();

        if let Some(window) = &self.window {
            match self.mode {
                AppMode::ClickThrough => {
                    self.drag_state = DragState::Idle;
                    // Check if cursor happens to be on the HUD
                    let is_over_hud = HudOverlay::contains_point(
                        self.hud_pos,
                        self.cursor_pos,
                        self.scale_factor as f32,
                    );
                    if is_over_hud {
                        let _ = self.platform.set_click_through(window, false);
                        self.is_click_through_active = false;
                        window.set_cursor(CursorIcon::Grab);
                    } else {
                        let _ = self.platform.set_click_through(window, true);
                        self.is_click_through_active = true;
                        window.set_cursor(CursorIcon::Default);
                    }
                }
                AppMode::Drawing => {
                    let _ = self.platform.set_click_through(window, false);
                    self.is_click_through_active = false;
                    if let Err(err) = self.platform.activate_window(window) {
                        eprintln!("[draw-rs] Failed to activate window: {err}");
                    }
                    window.set_cursor(self.default_cursor_for_tool());
                }
            }
            window.request_redraw();
        }
    }

    /// Selects appropriate default cursor icon for the active tool in drawing mode.
    fn default_cursor_for_tool(&self) -> CursorIcon {
        match self.tool {
            DrawingTool::Pen | DrawingTool::Circle | DrawingTool::Arrow => CursorIcon::Crosshair,
            DrawingTool::Eraser => CursorIcon::Default,
        }
    }

    /// Redraw the entire canvas and present the frame buffer with true native alpha transparency.
    fn render_frame(&mut self) {
        let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) else {
            return;
        };

        // 1. Clear pixmap to transparent
        renderer.clear();

        // 2. Render committed shapes plus active live preview
        let preview = self
            .drag_state
            .to_preview_drawable(self.color, self.stroke_width);
        let preview_ref: Option<&dyn Drawable> = preview.as_deref();

        self.canvas.draw_all(renderer.pixmap_mut(), preview_ref);

        // 3. If eraser tool is active in Drawing mode, draw interactive eraser radius ring
        if self.mode == AppMode::Drawing && self.tool == DrawingTool::Eraser {
            let radius = self.eraser_radius * (self.scale_factor as f32);
            let mut pb = PathBuilder::new();
            pb.push_circle(self.cursor_pos.x, self.cursor_pos.y, radius);
            if let Some(path) = pb.finish() {
                // Subtle glowing fill
                let mut fill_paint = Paint::default();
                fill_paint.set_color(Color::from_rgba8(244, 63, 94, 30));
                renderer.pixmap_mut().fill_path(
                    &path,
                    &fill_paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // Crisp outline
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(Color::from_rgba8(244, 63, 94, 210));
                let stroke = Stroke {
                    width: 2.0 * (self.scale_factor as f32),
                    ..Default::default()
                };
                renderer.pixmap_mut().stroke_path(
                    &path,
                    &stroke_paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }

        // 4. Render draggable Status HUD at dynamic position
        let hud_params = HudRenderParams {
            mode: self.mode,
            tool: self.tool,
            color: self.color,
            shape_count: self.canvas.shape_count(),
            language: self.language,
        };
        HudOverlay::render(
            renderer.pixmap_mut(),
            &hud_params,
            self.scale_factor as f32,
            self.hud_pos,
        );

        // 5. Present with native macOS Quartz Compositor alpha blending
        if let Err(err) = self.platform.present_pixmap(window, renderer.pixmap_mut()) {
            eprintln!("[draw-rs] Failed to present frame buffer: {err}");
        }
    }

    /// Clamp HUD position to keep the card fully visible inside window bounds.
    fn clamp_hud_position(&mut self, window_size: PhysicalSize<u32>) {
        let (card_w, card_h) = HudOverlay::get_card_size(self.scale_factor as f32);
        let max_x = (window_size.width as f32 - card_w - 8.0).max(8.0);
        let max_y = (window_size.height as f32 - card_h - 8.0).max(8.0);
        self.hud_pos.x = self.hud_pos.x.clamp(8.0, max_x);
        self.hud_pos.y = self.hud_pos.y.clamp(8.0, max_y);
    }
}

impl ApplicationHandler<AppEvent> for DrawApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Determine screen geometry for full-screen borderless overlay
        let monitor = event_loop.primary_monitor();
        let size = monitor
            .as_ref()
            .map(|m| m.size())
            .unwrap_or(PhysicalSize::new(1920, 1080));
        let pos = monitor
            .as_ref()
            .map(|m| m.position())
            .unwrap_or(PhysicalPosition::new(0, 0));

        let attrs = Window::default_attributes()
            .with_title("Draw-RS LeetCode Overlay")
            .with_inner_size(size)
            .with_position(pos)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_cursor(CursorIcon::Crosshair);

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("[draw-rs] Fatal: Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        self.scale_factor = window.scale_factor();

        // Configure native macOS overlay attributes (AppKit & CALayer transparency)
        if let Err(err) = self.platform.configure_overlay(&window) {
            eprintln!("[draw-rs] Warning: Failed to configure platform overlay: {err}");
        }

        let renderer = match CanvasRenderer::new(size.width, size.height) {
            Some(r) => r,
            None => {
                eprintln!("[draw-rs] Fatal: Failed to allocate canvas renderer");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        self.clamp_hud_position(size);
        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::ToggleMode => {
                self.toggle_mode();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                let win_size = self.window.as_ref().map(|w| w.inner_size());
                if let Some(size) = win_size {
                    self.clamp_hud_position(size);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                }
                self.clamp_hud_position(new_size);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point2D::new(position.x as f32, position.y as f32);

                // 1. If currently dragging HUD, update HUD position
                if let DragState::DraggingHud { drag_offset } = self.drag_state {
                    self.hud_pos = self.cursor_pos - drag_offset;
                    let win_size = self.window.as_ref().map(|w| w.inner_size());
                    if let Some(size) = win_size {
                        self.clamp_hud_position(size);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                    return;
                }

                // 2. Handle active drawing / erasing
                if self.mode == AppMode::Drawing {
                    match &mut self.drag_state {
                        DragState::DrawingStroke(points) => {
                            let should_push = points
                                .last()
                                .map(|last| last.distance(&self.cursor_pos) >= 2.0)
                                .unwrap_or(true);
                            if should_push {
                                points.push(self.cursor_pos);
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                            }
                        }
                        DragState::DrawingCircle { current, .. }
                        | DragState::DrawingArrow { current, .. } => {
                            *current = self.cursor_pos;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }
                        DragState::Erasing { removed } => {
                            let radius = self.eraser_radius * (self.scale_factor as f32);
                            let just_erased = self.canvas.erase_at(self.cursor_pos, radius);
                            if !just_erased.is_empty() {
                                removed.extend(just_erased);
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                            }
                        }
                        DragState::Idle => {
                            // Update cursor icon if hovering over HUD buttons or drag area
                            if let Some(w) = &self.window {
                                let hit = HudOverlay::hit_test(
                                    self.hud_pos,
                                    self.cursor_pos,
                                    self.scale_factor as f32,
                                );
                                match hit {
                                    HudHitTarget::Tool(_)
                                    | HudHitTarget::Color(_)
                                    | HudHitTarget::ToggleMode
                                    | HudHitTarget::ToggleLanguage
                                    | HudHitTarget::Undo
                                    | HudHitTarget::Clear => {
                                        w.set_cursor(CursorIcon::Pointer);
                                    }
                                    HudHitTarget::DragHeader => {
                                        w.set_cursor(CursorIcon::Grab);
                                    }
                                    HudHitTarget::None => {
                                        w.set_cursor(self.default_cursor_for_tool());
                                        // Request redraw when eraser is active to update eraser ring
                                        if self.tool == DrawingTool::Eraser {
                                            w.request_redraw();
                                        }
                                    }
                                }
                            }
                        }
                        DragState::DraggingHud { .. } => unreachable!(),
                    }
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        // Check if user clicked on the HUD (menu button or drag handle)
                        let hit = HudOverlay::hit_test(
                            self.hud_pos,
                            self.cursor_pos,
                            self.scale_factor as f32,
                        );

                        match hit {
                            HudHitTarget::Tool(tool) => {
                                self.tool = tool;
                                if self.mode == AppMode::ClickThrough {
                                    self.toggle_mode();
                                } else if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::Color(color) => {
                                self.color = color;
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::ToggleMode => {
                                self.toggle_mode();
                                return;
                            }
                            HudHitTarget::ToggleLanguage => {
                                self.language = self.language.toggle();
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::Undo => {
                                self.canvas.undo();
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::Clear => {
                                self.canvas.clear();
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::DragHeader => {
                                self.drag_state = DragState::DraggingHud {
                                    drag_offset: self.cursor_pos - self.hud_pos,
                                };
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Grabbing);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::None => {
                                // Outside HUD, proceed to canvas drawing
                            }
                        }

                        // Drawing / Erasing operations only in Drawing mode
                        if self.mode != AppMode::Drawing {
                            return;
                        }

                        match self.tool {
                            DrawingTool::Pen => {
                                self.drag_state = DragState::DrawingStroke(vec![self.cursor_pos]);
                            }
                            DrawingTool::Circle => {
                                self.drag_state = DragState::DrawingCircle {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }
                            DrawingTool::Arrow => {
                                self.drag_state = DragState::DrawingArrow {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }
                            DrawingTool::Eraser => {
                                let radius = self.eraser_radius * (self.scale_factor as f32);
                                let removed = self.canvas.erase_at(self.cursor_pos, radius);
                                self.drag_state = DragState::Erasing { removed };
                            }
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }

                    ElementState::Released => {
                        match std::mem::replace(&mut self.drag_state, DragState::Idle) {
                            DragState::DraggingHud { .. } => {
                                if let Some(w) = &self.window {
                                    let hit = HudOverlay::hit_test(
                                        self.hud_pos,
                                        self.cursor_pos,
                                        self.scale_factor as f32,
                                    );
                                    if self.mode == AppMode::ClickThrough {
                                        if hit != HudHitTarget::None {
                                            let icon = match hit {
                                                HudHitTarget::Tool(_)
                                                | HudHitTarget::Color(_)
                                                | HudHitTarget::ToggleMode
                                                | HudHitTarget::ToggleLanguage
                                                | HudHitTarget::Undo
                                                | HudHitTarget::Clear => CursorIcon::Pointer,
                                                _ => CursorIcon::Grab,
                                            };
                                            w.set_cursor(icon);
                                        } else {
                                            let _ = self.platform.set_click_through(w, true);
                                            self.is_click_through_active = true;
                                            w.set_cursor(CursorIcon::Default);
                                        }
                                    } else {
                                        let icon = match hit {
                                            HudHitTarget::Tool(_)
                                            | HudHitTarget::Color(_)
                                            | HudHitTarget::ToggleMode
                                            | HudHitTarget::ToggleLanguage
                                            | HudHitTarget::Undo
                                            | HudHitTarget::Clear => CursorIcon::Pointer,
                                            HudHitTarget::DragHeader => CursorIcon::Grab,
                                            HudHitTarget::None => self.default_cursor_for_tool(),
                                        };
                                        w.set_cursor(icon);
                                    }
                                    w.request_redraw();
                                }
                            }
                            DragState::Erasing { removed } => {
                                self.canvas.commit_erase(removed);
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                            }
                            preview_drag => {
                                if let Some(shape) =
                                    preview_drag.to_preview_drawable(self.color, self.stroke_width)
                                {
                                    self.canvas.push_shape(shape);
                                }
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                            }
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => match key {
                KeyCode::F1 => {
                    self.toggle_mode();
                }
                KeyCode::F2 => {
                    self.tool = self.tool.cycle();
                    if let Some(w) = &self.window {
                        w.set_cursor(self.default_cursor_for_tool());
                        w.request_redraw();
                    }
                }
                KeyCode::KeyE => {
                    self.tool = DrawingTool::Eraser;
                    if let Some(w) = &self.window {
                        w.set_cursor(self.default_cursor_for_tool());
                        w.request_redraw();
                    }
                }
                KeyCode::KeyP => {
                    self.tool = DrawingTool::Pen;
                    if let Some(w) = &self.window {
                        w.set_cursor(self.default_cursor_for_tool());
                        w.request_redraw();
                    }
                }
                KeyCode::KeyZ => {
                    self.canvas.undo();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::KeyC => {
                    self.canvas.clear();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::Escape => {
                    event_loop.exit();
                }
                KeyCode::Digit1 => {
                    self.color = PaletteColor::Cyan;
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::Digit2 => {
                    self.color = PaletteColor::Emerald;
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::Digit3 => {
                    self.color = PaletteColor::Coral;
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::Digit4 => {
                    self.color = PaletteColor::Amber;
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::Digit5 => {
                    self.color = PaletteColor::Violet;
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                KeyCode::KeyL => {
                    self.language = self.language.toggle();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.mode == AppMode::ClickThrough {
            // Keep checking cursor position at 60 FPS while in Click-Through mode
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(16),
            ));

            let Some(window) = &self.window else { return };

            // Don't modify click-through state if user is actively dragging the HUD
            if matches!(self.drag_state, DragState::DraggingHud { .. }) {
                return;
            }

            if let Some(global_cursor) = self.platform.get_global_cursor_pos(window) {
                self.cursor_pos = global_cursor;
                let is_over_hud = HudOverlay::contains_point(
                    self.hud_pos,
                    global_cursor,
                    self.scale_factor as f32,
                );

                if is_over_hud {
                    if self.is_click_through_active {
                        // Cursor hovered over HUD! Temporarily disable click-through
                        // so user can immediately click menu buttons, pick colors, or drag the HUD
                        let _ = self.platform.set_click_through(window, false);
                        self.is_click_through_active = false;
                    }
                    let hit =
                        HudOverlay::hit_test(self.hud_pos, global_cursor, self.scale_factor as f32);
                    let cursor_icon = match hit {
                        HudHitTarget::Tool(_)
                        | HudHitTarget::Color(_)
                        | HudHitTarget::ToggleMode
                        | HudHitTarget::ToggleLanguage
                        | HudHitTarget::Undo
                        | HudHitTarget::Clear => CursorIcon::Pointer,
                        _ => CursorIcon::Grab,
                    };
                    window.set_cursor(cursor_icon);
                } else if !self.is_click_through_active {
                    // Cursor left HUD! Re-enable click-through so clicks pass to background apps
                    let _ = self.platform.set_click_through(window, true);
                    self.is_click_through_active = true;
                    window.set_cursor(CursorIcon::Default);
                }
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
