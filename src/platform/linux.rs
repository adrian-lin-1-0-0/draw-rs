use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::num::NonZeroU32;
use std::sync::Mutex;
use tiny_skia::Pixmap;
use winit::window::{Window, WindowLevel};

use super::{PlatformError, WindowPlatformController};
use crate::models::point::Point2D;

/// Container wrapping raw display and window handles with `'static` lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RawHandleWrapper {
    display: RawDisplayHandle,
    window: RawWindowHandle,
}

impl HasDisplayHandle for RawHandleWrapper {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: The display handle originates from an active winit Window that is kept alive by DrawApp.
        unsafe { Ok(raw_window_handle::DisplayHandle::borrow_raw(self.display)) }
    }
}

impl HasWindowHandle for RawHandleWrapper {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: The window handle originates from an active winit Window that is kept alive by DrawApp.
        unsafe { Ok(raw_window_handle::WindowHandle::borrow_raw(self.window)) }
    }
}

struct SurfaceState {
    handles: RawHandleWrapper,
    _context: softbuffer::Context<RawHandleWrapper>,
    surface: softbuffer::Surface<RawHandleWrapper, RawHandleWrapper>,
    width: u32,
    height: u32,
}

// SAFETY: All softbuffer rendering operations on Linux occur exclusively on the main event loop thread.
unsafe impl Send for SurfaceState {}

/// Native window platform controller for Linux (supporting both X11 and Wayland compositors).
pub struct LinuxPlatformController {
    surface_state: Mutex<Option<SurfaceState>>,
}

impl Default for LinuxPlatformController {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxPlatformController {
    pub fn new() -> Self {
        Self {
            surface_state: Mutex::new(None),
        }
    }

    /// Retrieve or initialize the softbuffer Surface for the active Window.
    fn with_surface<F, R>(&self, window: &Window, f: F) -> Result<R, PlatformError>
    where
        F: FnOnce(
            &mut softbuffer::Surface<RawHandleWrapper, RawHandleWrapper>,
            &mut u32,
            &mut u32,
        ) -> Result<R, PlatformError>,
    {
        let display_raw = window
            .display_handle()
            .map_err(|_| PlatformError::InvalidWindowHandle)?
            .as_raw();
        let window_raw = window
            .window_handle()
            .map_err(|_| PlatformError::InvalidWindowHandle)?
            .as_raw();
        let current_handles = RawHandleWrapper {
            display: display_raw,
            window: window_raw,
        };

        let mut lock = self
            .surface_state
            .lock()
            .map_err(|_| PlatformError::BackendError("Failed to lock surface mutex".to_string()))?;

        // Re-initialize surface if handles changed or not yet initialized
        let needs_init = match lock.as_ref() {
            Some(state) => state.handles != current_handles,
            None => true,
        };

        if needs_init {
            let context = softbuffer::Context::new(current_handles).map_err(|err| {
                PlatformError::BackendError(format!(
                    "Failed to initialize softbuffer context: {err}"
                ))
            })?;
            let surface = softbuffer::Surface::new(&context, current_handles).map_err(|err| {
                PlatformError::BackendError(format!(
                    "Failed to initialize softbuffer surface: {err}"
                ))
            })?;
            *lock = Some(SurfaceState {
                handles: current_handles,
                _context: context,
                surface,
                width: 0,
                height: 0,
            });
        }

        let state = lock.as_mut().expect("Surface state must be initialized");
        f(&mut state.surface, &mut state.width, &mut state.height)
    }
}

impl WindowPlatformController for LinuxPlatformController {
    fn configure_overlay(&self, window: &Window) -> Result<(), PlatformError> {
        // 1. Enable window transparency hint (crucial for Wayland and X11 compositors like picom)
        window.set_transparent(true);

        // 2. Ensure frameless, borderless overlay
        window.set_decorations(false);

        // 3. Keep drawing overlay floating always on top of background applications
        window.set_window_level(WindowLevel::AlwaysOnTop);

        // 4. Default to intercepting mouse clicks for active drawing
        let _ = window.set_cursor_hittest(true);

        Ok(())
    }

    fn set_click_through(&self, window: &Window, click_through: bool) -> Result<(), PlatformError> {
        // When click_through is true, disable cursor hittest so mouse events pass through to windows below.
        // When click_through is false, re-enable cursor hittest to intercept drawing gestures.
        let _ = window.set_cursor_hittest(!click_through);
        Ok(())
    }

    fn activate_window(&self, window: &Window) -> Result<(), PlatformError> {
        window.focus_window();
        Ok(())
    }

    fn present_pixmap(&self, window: &Window, pixmap: &Pixmap) -> Result<(), PlatformError> {
        let width = NonZeroU32::new(pixmap.width()).ok_or_else(|| {
            PlatformError::BackendError("Pixmap width must be non-zero".to_string())
        })?;
        let height = NonZeroU32::new(pixmap.height()).ok_or_else(|| {
            PlatformError::BackendError("Pixmap height must be non-zero".to_string())
        })?;

        self.with_surface(window, |surface, last_w, last_h| {
            if *last_w != width.get() || *last_h != height.get() {
                surface.resize(width, height).map_err(|err| {
                    PlatformError::BackendError(format!(
                        "Failed to resize softbuffer surface: {err}"
                    ))
                })?;
                *last_w = width.get();
                *last_h = height.get();
            }

            let mut buffer = surface.buffer_mut().map_err(|err| {
                PlatformError::BackendError(format!("Failed to get softbuffer buffer: {err}"))
            })?;

            let count = buffer.len().min(pixmap.pixels().len());
            for (dst, src) in buffer[..count].iter_mut().zip(&pixmap.pixels()[..count]) {
                let a = src.alpha() as u32;
                let r = src.red() as u32;
                let g = src.green() as u32;
                let b = src.blue() as u32;
                *dst = (a << 24) | (r << 16) | (g << 8) | b;
            }

            buffer.present().map_err(|err| {
                PlatformError::BackendError(format!("Failed to present softbuffer buffer: {err}"))
            })?;

            Ok(())
        })
    }

    fn get_global_cursor_pos(&self, _window: &Window) -> Option<Point2D> {
        // On modern Linux (specifically Wayland), arbitrary background pointer snooping
        // is forbidden for sandboxed security. Return None to rely on standard winit events.
        None
    }

    fn keymap(&self) -> Box<dyn crate::platform::PlatformKeymap> {
        Box::new(crate::platform::LinuxKeymap)
    }
}
