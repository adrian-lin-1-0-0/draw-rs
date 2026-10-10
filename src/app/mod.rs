pub mod i18n;
pub mod state;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tiny_skia::{Color, Paint, PathBuilder, Stroke, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowId};

use crate::controller::CanvasController;
use crate::models::{
    CornerStyle, Drawable, Point2D, Rect2D, Sloppiness, StrokeStyle, TextShape, TransformHandle,
};
use crate::platform::WindowPlatformController;
use crate::render::{
    CanvasRenderer, HudHitTarget, HudOverlay, HudRenderParams, InspectorHitTarget,
    InspectorOverlay, InspectorRenderParams, SelectionRenderer,
};
pub use i18n::AppLanguage;
use state::{AppMode, DragState, DrawingTool, PaletteColor};

/// Custom application event triggered by global hotkeys or background threads.
#[derive(Debug, Clone)]
pub enum AppEvent {
    ToggleMode,
}

/// Active in-line text editing session.
#[derive(Debug, Clone)]
pub struct TextEditSession {
    pub shape_index: usize,
    pub is_new: bool,
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
    pub is_lang_menu_open: bool,
    stroke_width: f32,
    eraser_radius: f32,
    drag_state: DragState,
    cursor_pos: Point2D,
    scale_factor: f64,
    hud_pos: Point2D,
    inspector_pos: Point2D,
    pub is_properties_open: bool,
    is_click_through_active: bool,
    selected_indices: Vec<usize>,
    pub text_editing: Option<TextEditSession>,
    modifiers: ModifiersState,
    active_stroke_color: Color,
    active_fill_color: Option<Color>,
    active_stroke_style: StrokeStyle,
    active_sloppiness: Sloppiness,
    active_corner_style: CornerStyle,
    active_opacity: f32,
    active_font_size: f32,
}

impl DrawApp {
    pub fn new(platform: Box<dyn WindowPlatformController>) -> Self {
        let initial_color = PaletteColor::Cyan;
        Self {
            platform,
            canvas: CanvasController::new(),
            renderer: None,
            window: None,
            mode: AppMode::Drawing,
            tool: DrawingTool::Pen,
            color: initial_color,
            language: AppLanguage::default(),
            is_lang_menu_open: false,
            stroke_width: 3.5,
            eraser_radius: 18.0,
            drag_state: DragState::Idle,
            cursor_pos: Point2D::ZERO,
            scale_factor: 1.0,
            hud_pos: Point2D::new(24.0, 24.0),
            inspector_pos: Point2D::new(24.0, 150.0),
            is_properties_open: false,
            is_click_through_active: false,
            selected_indices: Vec::new(),
            text_editing: None,
            modifiers: ModifiersState::default(),
            active_stroke_color: initial_color.to_color(),
            active_fill_color: Some(
                Color::from_rgba(0.22, 0.74, 0.97, 0.25).unwrap_or(Color::TRANSPARENT),
            ),
            active_stroke_style: StrokeStyle::Solid,
            active_sloppiness: Sloppiness::Artist,
            active_corner_style: CornerStyle::Sharp,
            active_opacity: 1.0,
            active_font_size: 24.0,
        }
    }

    /// Computes the default anchored position for the Properties panel right below the HUD card.
    pub fn default_inspector_pos(&self) -> Point2D {
        let ui_scale = (self.scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5) as f32;
        let (prop_w, prop_h) = InspectorOverlay::get_card_size(self.scale_factor as f32);
        let (hud_w, hud_h) = HudOverlay::get_card_size(self.scale_factor as f32);

        let mut x = self.hud_pos.x + hud_w - prop_w;
        let mut y = self.hud_pos.y + hud_h + 6.0 * ui_scale;

        if let Some(window) = &self.window {
            let win_size = window.inner_size();
            let max_x = (win_size.width as f32 - prop_w - 8.0).max(8.0);
            let max_y = (win_size.height as f32 - prop_h - 8.0).max(8.0);
            x = x.clamp(8.0, max_x);
            if y > max_y && self.hud_pos.y > prop_h + 8.0 * ui_scale {
                y = self.hud_pos.y - prop_h - 6.0 * ui_scale;
            } else {
                y = y.clamp(8.0, max_y);
            }
        }
        Point2D::new(x, y)
    }

    /// Toggle between active Drawing mode and passive Click-Through mode.
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.toggle();
        self.is_lang_menu_open = false;

        if let Some(window) = &self.window {
            match self.mode {
                AppMode::ClickThrough => {
                    self.drag_state = DragState::Idle;
                    let is_over_hud = HudOverlay::contains_point(
                        self.hud_pos,
                        self.cursor_pos,
                        self.scale_factor as f32,
                        self.is_lang_menu_open,
                    );
                    let is_over_inspector = self.is_properties_open
                        && InspectorOverlay::contains_point(
                            self.inspector_pos,
                            self.cursor_pos,
                            self.scale_factor as f32,
                        );

                    if is_over_hud || is_over_inspector {
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

    /// Selects appropriate default cursor icon for active tool and selection hover.
    fn default_cursor_for_tool(&self) -> CursorIcon {
        match self.tool {
            DrawingTool::Selection => {
                if !self.selected_indices.is_empty()
                    && let Some(bounds) = self.canvas.combined_bounds(&self.selected_indices)
                {
                    let expanded = bounds.expand(4.0 * self.scale_factor as f32);
                    if let Some(handle) =
                        expanded.hit_test_handle(self.cursor_pos, 8.0 * self.scale_factor as f32)
                    {
                        return match handle {
                            TransformHandle::Nw | TransformHandle::Se => CursorIcon::NwseResize,
                            TransformHandle::Ne | TransformHandle::Sw => CursorIcon::NeswResize,
                            TransformHandle::N | TransformHandle::S => CursorIcon::NsResize,
                            TransformHandle::E | TransformHandle::W => CursorIcon::EwResize,
                            TransformHandle::Rotate => CursorIcon::Grab,
                        };
                    }
                    if bounds.contains(self.cursor_pos) {
                        return CursorIcon::Move;
                    }
                }
                CursorIcon::Default
            }
            DrawingTool::Hand => CursorIcon::Grab,
            DrawingTool::Pen
            | DrawingTool::Rectangle
            | DrawingTool::Diamond
            | DrawingTool::Circle
            | DrawingTool::Arrow
            | DrawingTool::Line => CursorIcon::Crosshair,
            DrawingTool::Text => CursorIcon::Text,
            DrawingTool::Eraser => CursorIcon::Default,
        }
    }

    /// Commits the active inline text editing session, deleting empty new shapes if discarded.
    pub fn commit_text_editing(&mut self) {
        if let Some(session) = self.text_editing.take() {
            if let Some(shape) = self.canvas.shape(session.shape_index)
                && let Some(text) = shape.text_content()
                && text.trim().is_empty()
                && session.is_new
            {
                self.canvas.delete_selected(&[session.shape_index]);
                self.selected_indices.clear();
            }
            if let Some(w) = &self.window {
                w.set_ime_allowed(false);
                w.request_redraw();
            }
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
        let preview = self.drag_state.to_preview_drawable_styled(
            self.active_stroke_color,
            self.active_fill_color,
            self.stroke_width,
            self.active_stroke_style,
            self.active_sloppiness,
            self.active_corner_style,
            self.active_opacity,
        );
        let preview_ref: Option<&dyn Drawable> = preview.as_deref();

        self.canvas.draw_all(renderer.pixmap_mut(), preview_ref);

        // 3. Render selection bounding box and resize handles
        if let DragState::BoxSelecting { start, current } = self.drag_state {
            SelectionRenderer::render_marquee(
                renderer.pixmap_mut(),
                start,
                current,
                self.scale_factor as f32,
            );
        }

        if self.text_editing.is_none()
            && !self.selected_indices.is_empty()
            && let Some(bounds) = self.canvas.combined_bounds(&self.selected_indices)
        {
            SelectionRenderer::render_selection_box(
                renderer.pixmap_mut(),
                bounds,
                self.scale_factor as f32,
            );
        }

        // Render in-line text editing active indicator & typing cursor
        if let Some(session) = &self.text_editing
            && let Some(shape) = self.canvas.shape(session.shape_index)
            && let Some(bb) = shape.bounding_box()
        {
            let expanded = bb.expand(4.0 * (self.scale_factor as f32));
            let mut pb = PathBuilder::new();
            if let Some(r) = tiny_skia::Rect::from_xywh(
                expanded.min_x,
                expanded.min_y,
                expanded.width().max(16.0),
                expanded.height().max(24.0),
            ) {
                pb.push_rect(r);
                if let Some(path) = pb.finish() {
                    let mut p = Paint::default();
                    p.set_color(Color::from_rgba8(59, 130, 246, 200));
                    let stroke = Stroke {
                        width: 1.5 * (self.scale_factor as f32),
                        ..Default::default()
                    };
                    renderer.pixmap_mut().stroke_path(
                        &path,
                        &p,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }

            // Draw vertical blinking/solid text cursor
            let cursor_x = expanded.max_x;
            let cursor_y1 = expanded.min_y + 4.0;
            let cursor_y2 = expanded.max_y - 4.0;
            let mut cursor_pb = PathBuilder::new();
            cursor_pb.move_to(cursor_x, cursor_y1);
            cursor_pb.line_to(cursor_x, cursor_y2.max(cursor_y1 + 16.0));
            if let Some(cursor_path) = cursor_pb.finish() {
                let mut p = Paint::default();
                p.set_color(Color::from_rgba8(59, 130, 246, 255));
                let stroke = Stroke {
                    width: 2.0 * (self.scale_factor as f32),
                    ..Default::default()
                };
                renderer.pixmap_mut().stroke_path(
                    &cursor_path,
                    &p,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }

        // 4. If eraser tool is active in Drawing mode, draw interactive eraser radius ring
        if self.mode == AppMode::Drawing && self.tool == DrawingTool::Eraser {
            let radius = self.eraser_radius * (self.scale_factor as f32);
            let mut pb = PathBuilder::new();
            pb.push_circle(self.cursor_pos.x, self.cursor_pos.y, radius);
            if let Some(path) = pb.finish() {
                // Glowing fill
                let mut fill_paint = Paint::default();
                fill_paint.set_color(Color::from_rgba8(244, 63, 94, 30));
                renderer.pixmap_mut().fill_path(
                    &path,
                    &fill_paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // Outline
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

        // 5. Render draggable Status HUD at dynamic position
        let hud_params = HudRenderParams {
            mode: self.mode,
            tool: self.tool,
            color: self.color,
            shape_count: self.canvas.shape_count(),
            language: self.language,
            is_lang_menu_open: self.is_lang_menu_open,
            is_properties_open: self.is_properties_open,
        };
        HudOverlay::render(
            renderer.pixmap_mut(),
            &hud_params,
            self.scale_factor as f32,
            self.hud_pos,
        );

        // 6. Render Properties Dropdown Inspector if expanded
        if self.is_properties_open {
            let inspector_params = InspectorRenderParams {
                selected_count: self.selected_indices.len(),
                stroke_color: self.active_stroke_color,
                fill_color: self.active_fill_color,
                stroke_width: self.stroke_width,
                stroke_style: self.active_stroke_style,
                language: self.language,
            };
            InspectorOverlay::render(
                renderer.pixmap_mut(),
                &inspector_params,
                self.scale_factor as f32,
                self.inspector_pos,
            );
        }

        // 7. Present with native macOS Quartz Compositor alpha blending
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

    /// Clamp Inspector position within window bounds.
    fn clamp_inspector_position(&mut self, window_size: PhysicalSize<u32>) {
        let (card_w, card_h) = InspectorOverlay::get_card_size(self.scale_factor as f32);
        let max_x = (window_size.width as f32 - card_w - 8.0).max(8.0);
        let max_y = (window_size.height as f32 - card_h - 8.0).max(8.0);
        self.inspector_pos.x = self.inspector_pos.x.clamp(8.0, max_x);
        self.inspector_pos.y = self.inspector_pos.y.clamp(8.0, max_y);
    }

    /// Updates attributes of selected shapes and commits the change to the Undo stack.
    fn apply_style_to_selected(&mut self) {
        if self.selected_indices.is_empty() {
            return;
        }

        let mut original = Vec::new();
        let mut modified = Vec::new();

        for &idx in &self.selected_indices {
            if let Some(shape) = self.canvas.shape_mut(idx) {
                original.push((idx, shape.clone_box()));
                shape.set_stroke_color(self.active_stroke_color);
                shape.set_fill_color(self.active_fill_color);
                shape.set_stroke_width(self.stroke_width);
                shape.set_stroke_style(self.active_stroke_style);
                shape.set_sloppiness(self.active_sloppiness);
                shape.set_corner_style(self.active_corner_style);
                shape.set_opacity(self.active_opacity);
                modified.push((idx, shape.clone_box()));
            }
        }

        self.canvas.commit_transform(original, modified);
    }
}

impl ApplicationHandler<AppEvent> for DrawApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

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
            .with_title("Draw-RS Whiteboard Overlay")
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
        self.clamp_inspector_position(size);
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
                    self.clamp_inspector_position(size);
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
                self.clamp_inspector_position(new_size);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point2D::new(position.x as f32, position.y as f32);

                // 1. If currently dragging HUD card, update HUD position
                if let DragState::DraggingHud { drag_offset } = self.drag_state {
                    self.hud_pos = self.cursor_pos - drag_offset;
                    let win_size = self.window.as_ref().map(|w| w.inner_size());
                    if let Some(size) = win_size {
                        self.clamp_hud_position(size);
                    }
                    if self.is_properties_open {
                        self.inspector_pos = self.default_inspector_pos();
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                    return;
                }

                // 2. If currently dragging Inspector card, update Inspector position
                if let DragState::DraggingInspector { drag_offset } = self.drag_state {
                    self.inspector_pos = self.cursor_pos - drag_offset;
                    let win_size = self.window.as_ref().map(|w| w.inner_size());
                    if let Some(size) = win_size {
                        self.clamp_inspector_position(size);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                    return;
                }

                // 3. Handle active drawing / erasing / selection operations
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
                        | DragState::DrawingArrow { current, .. }
                        | DragState::DrawingRectangle { current, .. }
                        | DragState::DrawingDiamond { current, .. }
                        | DragState::DrawingLine { current, .. }
                        | DragState::BoxSelecting { current, .. } => {
                            *current = self.cursor_pos;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }

                        DragState::PanningCanvas { last_pos, .. } => {
                            let delta = self.cursor_pos - *last_pos;
                            *last_pos = self.cursor_pos;
                            self.canvas.translate_all(delta);
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }

                        DragState::MovingSelection { last_pos, .. } => {
                            let delta = self.cursor_pos - *last_pos;
                            *last_pos = self.cursor_pos;
                            for &idx in &self.selected_indices {
                                if let Some(shape) = self.canvas.shape_mut(idx) {
                                    shape.translate(delta);
                                }
                            }
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }

                        DragState::ResizingSelection {
                            handle,
                            start_bounds,
                            start_mouse,
                            initial_shapes,
                        } => {
                            let mut new_bounds = *start_bounds;
                            let delta = self.cursor_pos - *start_mouse;
                            match handle {
                                TransformHandle::Nw => {
                                    new_bounds.min_x += delta.x;
                                    new_bounds.min_y += delta.y;
                                }
                                TransformHandle::N => {
                                    new_bounds.min_y += delta.y;
                                }
                                TransformHandle::Ne => {
                                    new_bounds.max_x += delta.x;
                                    new_bounds.min_y += delta.y;
                                }
                                TransformHandle::E => {
                                    new_bounds.max_x += delta.x;
                                }
                                TransformHandle::Se => {
                                    new_bounds.max_x += delta.x;
                                    new_bounds.max_y += delta.y;
                                }
                                TransformHandle::S => {
                                    new_bounds.max_y += delta.y;
                                }
                                TransformHandle::Sw => {
                                    new_bounds.min_x += delta.x;
                                    new_bounds.max_y += delta.y;
                                }
                                TransformHandle::W => {
                                    new_bounds.min_x += delta.x;
                                }
                                TransformHandle::Rotate => {}
                            }

                            if new_bounds.width() > 4.0 && new_bounds.height() > 4.0 {
                                for (idx, original) in initial_shapes {
                                    if let Some(shape) = self.canvas.shape_mut(*idx) {
                                        *shape = original.clone_box();
                                        shape.resize(*start_bounds, new_bounds);
                                    }
                                }
                            }
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }

                        DragState::RotatingSelection { .. } => {
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
                            if let Some(w) = &self.window {
                                let hud_hit = HudOverlay::hit_test(
                                    self.hud_pos,
                                    self.cursor_pos,
                                    self.scale_factor as f32,
                                    self.is_lang_menu_open,
                                );
                                let inspector_hit = if self.is_properties_open {
                                    InspectorOverlay::hit_test(
                                        self.inspector_pos,
                                        self.cursor_pos,
                                        self.scale_factor as f32,
                                    )
                                } else {
                                    InspectorHitTarget::None
                                };

                                if hud_hit != HudHitTarget::None {
                                    let icon = match hud_hit {
                                        HudHitTarget::DragHeader => CursorIcon::Grab,
                                        _ => CursorIcon::Pointer,
                                    };
                                    w.set_cursor(icon);
                                } else if inspector_hit != InspectorHitTarget::None {
                                    let icon = match inspector_hit {
                                        InspectorHitTarget::DragHeader => CursorIcon::Grab,
                                        InspectorHitTarget::Close => CursorIcon::Pointer,
                                        _ => CursorIcon::Pointer,
                                    };
                                    w.set_cursor(icon);
                                } else {
                                    w.set_cursor(self.default_cursor_for_tool());
                                    if self.tool == DrawingTool::Eraser {
                                        w.request_redraw();
                                    }
                                }
                            }
                        }

                        DragState::DraggingHud { .. } | DragState::DraggingInspector { .. } => {
                            unreachable!()
                        }
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
                        // 1. Check if user clicked on Top HUD
                        let hud_hit = HudOverlay::hit_test(
                            self.hud_pos,
                            self.cursor_pos,
                            self.scale_factor as f32,
                            self.is_lang_menu_open,
                        );

                        match hud_hit {
                            HudHitTarget::ToggleLanguageMenu => {
                                self.is_lang_menu_open = !self.is_lang_menu_open;
                                if self.is_lang_menu_open {
                                    self.is_properties_open = false;
                                }
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::ToggleProperties => {
                                self.is_properties_open = !self.is_properties_open;
                                self.is_lang_menu_open = false;
                                if self.is_properties_open {
                                    self.inspector_pos = self.default_inspector_pos();
                                }
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::SelectLanguage(lang) => {
                                self.language = lang;
                                self.is_lang_menu_open = false;
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::Tool(tool) => {
                                self.is_lang_menu_open = false;
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
                                self.is_lang_menu_open = false;
                                self.color = color;
                                self.active_stroke_color = color.to_color();
                                self.apply_style_to_selected();
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Pointer);
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::ToggleMode => {
                                self.is_lang_menu_open = false;
                                self.toggle_mode();
                                return;
                            }
                            HudHitTarget::Undo => {
                                self.is_lang_menu_open = false;
                                self.canvas.undo();
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::Clear => {
                                self.is_lang_menu_open = false;
                                self.canvas.clear();
                                self.selected_indices.clear();
                                if let Some(w) = &self.window {
                                    w.request_redraw();
                                }
                                return;
                            }
                            HudHitTarget::DragHeader => {
                                self.is_lang_menu_open = false;
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
                                if self.is_lang_menu_open {
                                    self.is_lang_menu_open = false;
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                            }
                        }

                        // 2. Check if user clicked on Properties Inspector HUD
                        if self.is_properties_open {
                            let insp_hit = InspectorOverlay::hit_test(
                                self.inspector_pos,
                                self.cursor_pos,
                                self.scale_factor as f32,
                            );

                            match insp_hit {
                                InspectorHitTarget::Close => {
                                    self.is_properties_open = false;
                                    if let Some(w) = &self.window {
                                        w.set_cursor(CursorIcon::Default);
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::DragHeader => {
                                    self.drag_state = DragState::DraggingInspector {
                                        drag_offset: self.cursor_pos - self.inspector_pos,
                                    };
                                    if let Some(w) = &self.window {
                                        w.set_cursor(CursorIcon::Grabbing);
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::StrokeColor(pal) => {
                                    self.color = pal;
                                    self.active_stroke_color = pal.to_color();
                                    self.apply_style_to_selected();
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::FillColor(opt_pal) => {
                                    self.active_fill_color = opt_pal.map(|p| {
                                        Color::from_rgba(
                                            p.to_color().red(),
                                            p.to_color().green(),
                                            p.to_color().blue(),
                                            0.30,
                                        )
                                        .unwrap_or(Color::TRANSPARENT)
                                    });
                                    self.apply_style_to_selected();
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::StrokeWidth(w_val) => {
                                    self.stroke_width = w_val;
                                    self.apply_style_to_selected();
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::StrokeStyle(style) => {
                                    self.active_stroke_style = style;
                                    self.apply_style_to_selected();
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::LayerFront => {
                                    self.selected_indices =
                                        self.canvas.bring_to_front(&self.selected_indices);
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::LayerForward => {
                                    self.selected_indices =
                                        self.canvas.bring_forward(&self.selected_indices);
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::LayerBackward => {
                                    self.selected_indices =
                                        self.canvas.send_backward(&self.selected_indices);
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::LayerBack => {
                                    self.selected_indices =
                                        self.canvas.send_to_back(&self.selected_indices);
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::Duplicate => {
                                    self.selected_indices = self.canvas.duplicate(
                                        &self.selected_indices,
                                        Point2D::new(20.0, 20.0),
                                    );
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::Delete => {
                                    self.canvas.delete_selected(&self.selected_indices);
                                    self.selected_indices.clear();
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                                InspectorHitTarget::None => {}
                            }
                        }

                        // Drawing / Erasing / Selecting operations only in Drawing mode
                        if self.mode != AppMode::Drawing {
                            return;
                        }

                        match self.tool {
                            DrawingTool::Hand => {
                                self.commit_text_editing();
                                self.selected_indices.clear();
                                let initial_shapes: Vec<(usize, Box<dyn Drawable>)> =
                                    (0..self.canvas.shape_count())
                                        .filter_map(|idx| {
                                            self.canvas.shape(idx).map(|s| (idx, s.clone_box()))
                                        })
                                        .collect();
                                self.drag_state = DragState::PanningCanvas {
                                    last_pos: self.cursor_pos,
                                    initial_shapes,
                                };
                                if let Some(w) = &self.window {
                                    w.set_cursor(CursorIcon::Grabbing);
                                    w.request_redraw();
                                }
                                return;
                            }

                            DrawingTool::Selection => {
                                self.commit_text_editing();
                                // 1. Check transform handles on existing selection
                                if !self.selected_indices.is_empty()
                                    && let Some(bounds) =
                                        self.canvas.combined_bounds(&self.selected_indices)
                                {
                                    let expanded = bounds.expand(4.0 * self.scale_factor as f32);
                                    if let Some(handle) = expanded.hit_test_handle(
                                        self.cursor_pos,
                                        8.0 * self.scale_factor as f32,
                                    ) {
                                        let initial_shapes: Vec<(usize, Box<dyn Drawable>)> = self
                                            .selected_indices
                                            .iter()
                                            .filter_map(|&idx| {
                                                self.canvas.shape(idx).map(|s| (idx, s.clone_box()))
                                            })
                                            .collect();

                                        if handle == TransformHandle::Rotate {
                                            self.drag_state = DragState::RotatingSelection {
                                                center: bounds.center(),
                                                start_angle: bounds
                                                    .center()
                                                    .angle_to(&self.cursor_pos),
                                                initial_shapes,
                                            };
                                        } else {
                                            self.drag_state = DragState::ResizingSelection {
                                                handle,
                                                start_bounds: bounds,
                                                start_mouse: self.cursor_pos,
                                                initial_shapes,
                                            };
                                        }
                                        if let Some(w) = &self.window {
                                            w.request_redraw();
                                        }
                                        return;
                                    }
                                }

                                // 2. Check if clicked inside existing selection bounding box
                                if !self.selected_indices.is_empty()
                                    && let Some(bounds) =
                                        self.canvas.combined_bounds(&self.selected_indices)
                                    && bounds.contains(self.cursor_pos)
                                {
                                    let initial_shapes: Vec<(usize, Box<dyn Drawable>)> = self
                                        .selected_indices
                                        .iter()
                                        .filter_map(|&idx| {
                                            self.canvas.shape(idx).map(|s| (idx, s.clone_box()))
                                        })
                                        .collect();

                                    self.drag_state = DragState::MovingSelection {
                                        start_pos: self.cursor_pos,
                                        last_pos: self.cursor_pos,
                                        initial_shapes,
                                    };
                                    if let Some(w) = &self.window {
                                        w.set_cursor(CursorIcon::Grabbing);
                                        w.request_redraw();
                                    }
                                    return;
                                }

                                // 3. Check if clicked on any shape
                                if let Some(idx) = self.canvas.hit_test_all(self.cursor_pos) {
                                    if self.modifiers.shift_key() {
                                        if let Some(pos) =
                                            self.selected_indices.iter().position(|&i| i == idx)
                                        {
                                            self.selected_indices.remove(pos);
                                        } else {
                                            self.selected_indices.push(idx);
                                        }
                                    } else {
                                        self.selected_indices = vec![idx];
                                    }

                                    // Sync active styles with selected shape
                                    if let Some(s) = self.canvas.shape(idx) {
                                        self.active_stroke_color = s.stroke_color();
                                        self.active_fill_color = s.fill_color();
                                        self.stroke_width = s.stroke_width();
                                        self.active_stroke_style = s.stroke_style();
                                        self.active_sloppiness = s.sloppiness();
                                        self.active_corner_style = s.corner_style();
                                        self.active_opacity = s.opacity();
                                    }

                                    let initial_shapes: Vec<(usize, Box<dyn Drawable>)> = self
                                        .selected_indices
                                        .iter()
                                        .filter_map(|&i| {
                                            self.canvas.shape(i).map(|s| (i, s.clone_box()))
                                        })
                                        .collect();

                                    self.drag_state = DragState::MovingSelection {
                                        start_pos: self.cursor_pos,
                                        last_pos: self.cursor_pos,
                                        initial_shapes,
                                    };
                                    if let Some(w) = &self.window {
                                        w.set_cursor(CursorIcon::Grabbing);
                                        w.request_redraw();
                                    }
                                } else {
                                    // Clicked empty area -> start box select
                                    if !self.modifiers.shift_key() {
                                        self.selected_indices.clear();
                                    }
                                    self.drag_state = DragState::BoxSelecting {
                                        start: self.cursor_pos,
                                        current: self.cursor_pos,
                                    };
                                }
                            }

                            DrawingTool::Rectangle => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingRectangle {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }

                            DrawingTool::Diamond => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingDiamond {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }

                            DrawingTool::Circle => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingCircle {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }

                            DrawingTool::Arrow => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingArrow {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }

                            DrawingTool::Line => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingLine {
                                    start: self.cursor_pos,
                                    current: self.cursor_pos,
                                };
                            }

                            DrawingTool::Pen => {
                                self.selected_indices.clear();
                                self.drag_state = DragState::DrawingStroke(vec![self.cursor_pos]);
                            }

                            DrawingTool::Text => {
                                self.commit_text_editing();

                                // Check if clicked on an existing text shape to edit it
                                if let Some(idx) = self.canvas.hit_test_all(self.cursor_pos)
                                    && let Some(shape) = self.canvas.shape(idx)
                                    && shape.text_content().is_some()
                                {
                                    self.selected_indices = vec![idx];
                                    self.text_editing = Some(TextEditSession {
                                        shape_index: idx,
                                        is_new: false,
                                    });
                                } else {
                                    // Create new TextShape with NO default text! (User inputs text)
                                    let text_shape = TextShape::new(
                                        self.cursor_pos,
                                        "",
                                        self.active_font_size,
                                        self.active_stroke_color,
                                    );
                                    self.canvas.push_shape(Box::new(text_shape));
                                    let idx = self.canvas.shape_count() - 1;
                                    self.selected_indices = vec![idx];
                                    self.text_editing = Some(TextEditSession {
                                        shape_index: idx,
                                        is_new: true,
                                    });
                                }

                                if let Some(w) = &self.window {
                                    let _ = self.platform.set_click_through(w, false);
                                    w.set_ime_allowed(true);
                                    w.request_redraw();
                                }
                                return;
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
                                        self.is_lang_menu_open,
                                    );
                                    if self.mode == AppMode::ClickThrough {
                                        if hit != HudHitTarget::None {
                                            w.set_cursor(CursorIcon::Pointer);
                                        } else {
                                            let _ = self.platform.set_click_through(w, true);
                                            self.is_click_through_active = true;
                                            w.set_cursor(CursorIcon::Default);
                                        }
                                    } else {
                                        w.set_cursor(self.default_cursor_for_tool());
                                    }
                                    w.request_redraw();
                                }
                            }

                            DragState::DraggingInspector { .. } => {
                                if let Some(w) = &self.window {
                                    if self.mode == AppMode::ClickThrough {
                                        let hit = InspectorOverlay::hit_test(
                                            self.inspector_pos,
                                            self.cursor_pos,
                                            self.scale_factor as f32,
                                        );
                                        if hit != InspectorHitTarget::None {
                                            w.set_cursor(CursorIcon::Pointer);
                                        } else {
                                            let _ = self.platform.set_click_through(w, true);
                                            self.is_click_through_active = true;
                                            w.set_cursor(CursorIcon::Default);
                                        }
                                    } else {
                                        w.set_cursor(self.default_cursor_for_tool());
                                    }
                                    w.request_redraw();
                                }
                            }
                            DragState::PanningCanvas { initial_shapes, .. } => {
                                let modified_shapes: Vec<(usize, Box<dyn Drawable>)> =
                                    (0..self.canvas.shape_count())
                                        .filter_map(|idx| {
                                            self.canvas.shape(idx).map(|s| (idx, s.clone_box()))
                                        })
                                        .collect();
                                self.canvas
                                    .commit_transform(initial_shapes, modified_shapes);
                                if let Some(w) = &self.window {
                                    w.set_cursor(self.default_cursor_for_tool());
                                    w.request_redraw();
                                }
                            }

                            DragState::MovingSelection {
                                start_pos,
                                initial_shapes,
                                ..
                            } => {
                                if self.cursor_pos.distance(&start_pos) >= 1.0 {
                                    let modified_shapes: Vec<(usize, Box<dyn Drawable>)> =
                                        initial_shapes
                                            .iter()
                                            .filter_map(|(idx, _)| {
                                                self.canvas
                                                    .shape(*idx)
                                                    .map(|s| (*idx, s.clone_box()))
                                            })
                                            .collect();
                                    self.canvas
                                        .commit_transform(initial_shapes, modified_shapes);
                                }
                                if let Some(w) = &self.window {
                                    w.set_cursor(self.default_cursor_for_tool());
                                    w.request_redraw();
                                }
                            }

                            DragState::ResizingSelection { initial_shapes, .. } => {
                                let modified_shapes: Vec<(usize, Box<dyn Drawable>)> =
                                    initial_shapes
                                        .iter()
                                        .filter_map(|(idx, _)| {
                                            self.canvas.shape(*idx).map(|s| (*idx, s.clone_box()))
                                        })
                                        .collect();
                                self.canvas
                                    .commit_transform(initial_shapes, modified_shapes);
                                if let Some(w) = &self.window {
                                    w.set_cursor(self.default_cursor_for_tool());
                                    w.request_redraw();
                                }
                            }

                            DragState::RotatingSelection { initial_shapes, .. } => {
                                let modified_shapes: Vec<(usize, Box<dyn Drawable>)> =
                                    initial_shapes
                                        .iter()
                                        .filter_map(|(idx, _)| {
                                            self.canvas.shape(*idx).map(|s| (*idx, s.clone_box()))
                                        })
                                        .collect();
                                self.canvas
                                    .commit_transform(initial_shapes, modified_shapes);
                                if let Some(w) = &self.window {
                                    w.set_cursor(self.default_cursor_for_tool());
                                    w.request_redraw();
                                }
                            }

                            DragState::BoxSelecting { start, current } => {
                                let rect = Rect2D::from_points(start, current);
                                if rect.width() >= 3.0 && rect.height() >= 3.0 {
                                    self.selected_indices = self.canvas.box_select(rect);
                                }
                                if let Some(w) = &self.window {
                                    w.set_cursor(self.default_cursor_for_tool());
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
                                if let Some(shape) = preview_drag.to_preview_drawable_styled(
                                    self.active_stroke_color,
                                    self.active_fill_color,
                                    self.stroke_width,
                                    self.active_stroke_style,
                                    self.active_sloppiness,
                                    self.active_corner_style,
                                    self.active_opacity,
                                ) {
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

            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                if let Some(session) = &self.text_editing {
                    if let Some(shape) = self.canvas.shape_mut(session.shape_index) {
                        let mut current = shape.text_content().unwrap_or("").to_string();
                        current.push_str(&text);
                        shape.set_text_content(current);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        text,
                        ..
                    },
                ..
            } => {
                // If inline text editing is active, route all keystrokes to the text shape
                if let Some(session) = &self.text_editing {
                    let shape_idx = session.shape_index;

                    if key == KeyCode::Escape {
                        self.commit_text_editing();
                        return;
                    }

                    if key == KeyCode::Enter {
                        if let Some(shape) = self.canvas.shape_mut(shape_idx) {
                            let mut current = shape.text_content().unwrap_or("").to_string();
                            current.push('\n');
                            shape.set_text_content(current);
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    if key == KeyCode::Backspace {
                        if let Some(shape) = self.canvas.shape_mut(shape_idx) {
                            let mut current = shape.text_content().unwrap_or("").to_string();
                            current.pop();
                            shape.set_text_content(current);
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    if let Some(text_str) = text.as_deref() {
                        let filtered: String = text_str
                            .chars()
                            .filter(|&c| c >= ' ' || c == '\t')
                            .collect();
                        if !filtered.is_empty() {
                            if let Some(shape) = self.canvas.shape_mut(shape_idx) {
                                let mut current = shape.text_content().unwrap_or("").to_string();
                                current.push_str(&filtered);
                                shape.set_text_content(current);
                            }
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }
                    }
                    return;
                }

                match key {
                    KeyCode::F1 => {
                        self.commit_text_editing();
                        self.toggle_mode();
                    }
                    KeyCode::F2 => {
                        self.commit_text_editing();
                        self.tool = self.tool.cycle();
                        if self.tool == DrawingTool::Hand {
                            self.selected_indices.clear();
                        }
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyV => {
                        self.commit_text_editing();
                        self.tool = DrawingTool::Selection;
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyH => {
                        self.commit_text_editing();
                        self.tool = DrawingTool::Hand;
                        self.selected_indices.clear();
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyR => {
                        self.tool = DrawingTool::Rectangle;
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyD => {
                        if self.modifiers.super_key() {
                            self.selected_indices = self
                                .canvas
                                .duplicate(&self.selected_indices, Point2D::new(20.0, 20.0));
                        } else {
                            self.tool = DrawingTool::Diamond;
                        }
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyO => {
                        self.tool = DrawingTool::Circle;
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyA => {
                        self.tool = DrawingTool::Arrow;
                        if let Some(w) = &self.window {
                            w.set_cursor(self.default_cursor_for_tool());
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyL => {
                        self.tool = DrawingTool::Line;
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
                    KeyCode::KeyT => {
                        self.tool = DrawingTool::Text;
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
                    KeyCode::KeyZ => {
                        if self.modifiers.super_key() && self.modifiers.shift_key() {
                            self.canvas.redo();
                        } else {
                            self.canvas.undo();
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Delete | KeyCode::Backspace if !self.selected_indices.is_empty() => {
                        self.canvas.delete_selected(&self.selected_indices);
                        self.selected_indices.clear();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::KeyC => {
                        self.canvas.clear();
                        self.selected_indices.clear();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Escape => {
                        if self.is_lang_menu_open {
                            self.is_lang_menu_open = false;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        } else if self.is_properties_open {
                            self.is_properties_open = false;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        } else if !self.selected_indices.is_empty() {
                            self.selected_indices.clear();
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        } else {
                            event_loop.exit();
                        }
                    }
                    KeyCode::Digit1 => {
                        self.color = PaletteColor::Cyan;
                        self.active_stroke_color = self.color.to_color();
                        self.apply_style_to_selected();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Digit2 => {
                        self.color = PaletteColor::Emerald;
                        self.active_stroke_color = self.color.to_color();
                        self.apply_style_to_selected();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Digit3 => {
                        self.color = PaletteColor::Coral;
                        self.active_stroke_color = self.color.to_color();
                        self.apply_style_to_selected();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Digit4 => {
                        self.color = PaletteColor::Amber;
                        self.active_stroke_color = self.color.to_color();
                        self.apply_style_to_selected();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    KeyCode::Digit5 => {
                        self.color = PaletteColor::Violet;
                        self.active_stroke_color = self.color.to_color();
                        self.apply_style_to_selected();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.mode == AppMode::ClickThrough {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(16),
            ));

            let Some(window) = &self.window else { return };

            if matches!(
                self.drag_state,
                DragState::DraggingHud { .. } | DragState::DraggingInspector { .. }
            ) {
                return;
            }

            if let Some(global_cursor) = self.platform.get_global_cursor_pos(window) {
                self.cursor_pos = global_cursor;
                let is_over_hud = HudOverlay::contains_point(
                    self.hud_pos,
                    global_cursor,
                    self.scale_factor as f32,
                    self.is_lang_menu_open,
                );
                let is_over_inspector = self.is_properties_open
                    && InspectorOverlay::contains_point(
                        self.inspector_pos,
                        global_cursor,
                        self.scale_factor as f32,
                    );
                let is_over_interactive = is_over_hud || is_over_inspector;

                if is_over_interactive {
                    if self.is_click_through_active {
                        let _ = self.platform.set_click_through(window, false);
                        self.is_click_through_active = false;
                    }

                    let hit_hud = HudOverlay::hit_test(
                        self.hud_pos,
                        global_cursor,
                        self.scale_factor as f32,
                        self.is_lang_menu_open,
                    );
                    let hit_inspector = if self.is_properties_open {
                        InspectorOverlay::hit_test(
                            self.inspector_pos,
                            global_cursor,
                            self.scale_factor as f32,
                        )
                    } else {
                        InspectorHitTarget::None
                    };

                    let cursor_icon = if hit_hud != HudHitTarget::None {
                        match hit_hud {
                            HudHitTarget::DragHeader => CursorIcon::Grab,
                            _ => CursorIcon::Pointer,
                        }
                    } else if hit_inspector != InspectorHitTarget::None {
                        match hit_inspector {
                            InspectorHitTarget::DragHeader => CursorIcon::Grab,
                            _ => CursorIcon::Pointer,
                        }
                    } else {
                        CursorIcon::Grab
                    };
                    window.set_cursor(cursor_icon);
                } else if !self.is_click_through_active {
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
