pub mod keymap;
pub use keymap::{
    LinuxKeymap, MacosKeymap, PlatformKeymap, ShortcutAction, default_platform_keymap,
};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacosPlatformController;

pub mod linux;
pub use linux::LinuxPlatformController;

pub mod dummy;
pub use dummy::DummyPlatformController;

use std::fmt;
use winit::window::Window;

#[derive(Debug)]
pub enum PlatformError {
    InvalidWindowHandle,
    AppKitError(String),
    BackendError(String),
    #[allow(dead_code)]
    UnsupportedPlatform,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindowHandle => write!(f, "Invalid raw window handle"),
            Self::AppKitError(msg) => write!(f, "AppKit error: {msg}"),
            Self::BackendError(msg) => write!(f, "Backend error: {msg}"),
            Self::UnsupportedPlatform => write!(f, "Current platform is not supported"),
        }
    }
}

impl std::error::Error for PlatformError {}

/// Platform-specific abstraction for window behaviors (DIP & ISP).
///
/// Encapsulates native OS calls (such as macOS AppKit ignoresMouseEvents and window levels)
/// behind safe Rust trait methods.
pub trait WindowPlatformController: Send + Sync {
    /// Configure the window as a transparent, always-on-top, frameless overlay.
    fn configure_overlay(&self, window: &Window) -> Result<(), PlatformError>;

    /// Toggle mouse event click-through.
    /// When true, mouse clicks pass directly through to windows below.
    fn set_click_through(&self, window: &Window, click_through: bool) -> Result<(), PlatformError>;

    /// Reactivate and bring the window to the foreground as key window.
    fn activate_window(&self, window: &Window) -> Result<(), PlatformError>;

    /// Present a tiny-skia Pixmap with true native alpha transparency directly to the window layer.
    fn present_pixmap(
        &self,
        window: &Window,
        pixmap: &tiny_skia::Pixmap,
    ) -> Result<(), PlatformError>;

    /// Retrieve the current mouse cursor location in window physical pixel coordinates,
    /// even if the window is currently unfocused or ignoring mouse events.
    fn get_global_cursor_pos(&self, window: &Window) -> Option<crate::models::point::Point2D>;

    /// Retrieve the platform-specific keymap & shortcut resolver.
    fn keymap(&self) -> Box<dyn PlatformKeymap> {
        default_platform_keymap()
    }
}
