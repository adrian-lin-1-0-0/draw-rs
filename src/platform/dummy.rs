use super::{PlatformError, WindowPlatformController};
use tiny_skia::Pixmap;
use winit::window::Window;

use crate::models::point::Point2D;

/// Fallback platform controller for non-macOS environments (such as Linux CI runners).
#[derive(Default)]
pub struct DummyPlatformController;

impl DummyPlatformController {
    pub fn new() -> Self {
        Self
    }
}

impl WindowPlatformController for DummyPlatformController {
    fn configure_overlay(&self, _window: &Window) -> Result<(), PlatformError> {
        Ok(())
    }

    fn set_click_through(
        &self,
        _window: &Window,
        _click_through: bool,
    ) -> Result<(), PlatformError> {
        Ok(())
    }

    fn activate_window(&self, _window: &Window) -> Result<(), PlatformError> {
        Ok(())
    }

    fn present_pixmap(&self, _window: &Window, _pixmap: &Pixmap) -> Result<(), PlatformError> {
        Ok(())
    }

    fn get_global_cursor_pos(&self, _window: &Window) -> Option<Point2D> {
        None
    }
}
