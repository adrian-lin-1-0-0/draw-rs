pub mod state;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowId};

use crate::controller::CanvasController;
use crate::models::{point::Point2D, Drawable};
use crate::platform::WindowPlatformController;
use crate::render::{CanvasRenderer, HudOverlay};
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
    stroke_width: f32,
    drag_state: DragState,
    cursor_pos: Point2D,
    scale_factor: f64,
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
            stroke_width: 3.5,
            drag_state: DragState::Idle,
            cursor_pos: Point2D::ZERO,
            scale_factor: 1.0,
        }
    }

    /// Toggle between active Drawing mode and passive Click-Through mode.
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.toggle();

        if let Some(window) = &self.window {
            match self.mode {
                AppMode::ClickThrough => {
                    self.drag_state = DragState::Idle;
                    window.set_cursor(CursorIcon::Default);
                    if let Err(err) = self.platform.set_click_through(window, true) {
                        eprintln!("[draw-rs] Failed to set click-through mode: {err}");
                    }
                }
                AppMode::Drawing => {
                    if let Err(err) = self.platform.set_click_through(window, false) {
                        eprintln!("[draw-rs] Failed to disable click-through mode: {err}");
                    }
                    if let Err(err) = self.platform.activate_window(window) {
                        eprintln!("[draw-rs] Failed to activate window: {err}");
                    }
                    window.set_cursor(CursorIcon::Crosshair);
                }
            }
            window.request_redraw();
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

        // 3. Render Status HUD
        HudOverlay::render(
            renderer.pixmap_mut(),
            self.mode,
            self.tool,
            self.color,
            self.canvas.shape_count(),
            self.scale_factor as f32,
        );

        // 4. Present with native macOS Quartz Compositor alpha blending
        if let Err(err) = self.platform.present_pixmap(window, renderer.pixmap_mut()) {
            eprintln!("[draw-rs] Failed to present frame buffer: {err}");
        }
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
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point2D::new(position.x as f32, position.y as f32);

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
                        DragState::Idle => {}
                    }
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                if self.mode != AppMode::Drawing {
                    return;
                }

                match state {
                    ElementState::Pressed => {
                        self.drag_state = match self.tool {
                            DrawingTool::Pen => DragState::DrawingStroke(vec![self.cursor_pos]),
                            DrawingTool::Circle => DragState::DrawingCircle {
                                start: self.cursor_pos,
                                current: self.cursor_pos,
                            },
                            DrawingTool::Arrow => DragState::DrawingArrow {
                                start: self.cursor_pos,
                                current: self.cursor_pos,
                            },
                        };
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    ElementState::Released => {
                        // Commit active preview into canvas history
                        if let Some(shape) = self
                            .drag_state
                            .to_preview_drawable(self.color, self.stroke_width)
                        {
                            self.canvas.push_shape(shape);
                        }
                        self.drag_state = DragState::Idle;
                        if let Some(w) = &self.window {
                            w.request_redraw();
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
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
}
