use super::{PlatformError, WindowPlatformController};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSColor, NSEvent, NSStatusWindowLevel, NSView, NSWindow,
    NSWindowCollectionBehavior,
};
use objc2_core_foundation::{CFData, CFRetained};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
    CGImageByteOrderInfo,
};
use objc2_quartz_core::CATransaction;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tiny_skia::Pixmap;
use winit::window::Window;

use crate::models::point::Point2D;

/// Implementation of window platform controller for macOS using modern `objc2` and `objc2-app-kit`.
///
/// Encapsulates all unsafe Objective-C runtime operations and ensures strict memory/type safety.
pub struct MacosPlatformController {
    color_space: CFRetained<CGColorSpace>,
}

impl Default for MacosPlatformController {
    fn default() -> Self {
        Self::new()
    }
}

impl MacosPlatformController {
    pub fn new() -> Self {
        let color_space = CGColorSpace::new_device_rgb().expect("Failed to create RGB color space");
        Self { color_space }
    }

    /// Helper to safely retrieve the `NSView` associated with a `winit::window::Window`.
    fn with_ns_view<F, R>(&self, window: &Window, f: F) -> Result<R, PlatformError>
    where
        F: FnOnce(&NSView) -> Result<R, PlatformError>,
    {
        let handle = window
            .window_handle()
            .map_err(|_| PlatformError::InvalidWindowHandle)?;

        match handle.as_raw() {
            RawWindowHandle::AppKit(appkit) => {
                // SAFETY:
                // appkit.ns_view is guaranteed by raw-window-handle to be a valid pointer to NSView
                // as long as the Window is alive.
                unsafe {
                    let view_ptr = appkit.ns_view.as_ptr().cast::<NSView>();
                    let view: &NSView = &*view_ptr;
                    f(view)
                }
            }
            _ => Err(PlatformError::InvalidWindowHandle),
        }
    }

    /// Helper to safely retrieve the `NSWindow` associated with a `winit::window::Window`.
    fn with_ns_window<F, R>(&self, window: &Window, f: F) -> Result<R, PlatformError>
    where
        F: FnOnce(&NSWindow) -> R,
    {
        self.with_ns_view(window, |view| {
            let ns_window_opt: Option<Retained<NSWindow>> = view.window();
            match ns_window_opt {
                Some(ns_win) => Ok(f(&ns_win)),
                None => Err(PlatformError::AppKitError(
                    "NSView is not currently attached to an NSWindow".to_string(),
                )),
            }
        })
    }
}

impl WindowPlatformController for MacosPlatformController {
    fn configure_overlay(&self, window: &Window) -> Result<(), PlatformError> {
        // 1. Configure the NSView and its layer for transparency
        self.with_ns_view(window, |view| {
            view.setWantsLayer(true);
            if let Some(layer) = view.layer() {
                layer.setOpaque(false);
            }
            Ok(())
        })?;

        // 2. Configure the NSWindow for frameless, floating transparency
        self.with_ns_window(window, |ns_win| {
            // Complete transparency
            ns_win.setOpaque(false);
            ns_win.setBackgroundColor(Some(&NSColor::clearColor()));
            ns_win.setHasShadow(false);

            // High floating level (NSStatusWindowLevel stays above regular floating windows)
            ns_win.setLevel(NSStatusWindowLevel);

            // Collection behavior: Visible across all macOS spaces & fullscreen apps without switching spaces
            let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary;
            ns_win.setCollectionBehavior(behavior);

            // Default to intercepting mouse events (drawing mode)
            ns_win.setIgnoresMouseEvents(false);
        })
    }

    fn set_click_through(&self, window: &Window, click_through: bool) -> Result<(), PlatformError> {
        self.with_ns_window(window, |ns_win| {
            ns_win.setIgnoresMouseEvents(click_through);
        })
    }

    fn activate_window(&self, window: &Window) -> Result<(), PlatformError> {
        window.focus_window();

        self.with_ns_window(window, |ns_win| {
            if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                #[allow(deprecated)]
                app.activateIgnoringOtherApps(true);
                ns_win.makeKeyAndOrderFront(None);
            }
        })
    }

    fn present_pixmap(&self, window: &Window, pixmap: &Pixmap) -> Result<(), PlatformError> {
        self.with_ns_view(window, |view| {
            let Some(layer) = view.layer() else {
                return Err(PlatformError::AppKitError(
                    "NSView has no CALayer".to_string(),
                ));
            };

            let width = pixmap.width() as usize;
            let height = pixmap.height() as usize;
            let data = pixmap.data();

            // Wrap pixel buffer safely into CFData
            let cf_data = unsafe { CFData::new(None, data.as_ptr(), data.len() as isize) }
                .ok_or_else(|| {
                    PlatformError::AppKitError("Failed to allocate CFData".to_string())
                })?;

            let provider = CGDataProvider::with_cf_data(Some(&cf_data)).ok_or_else(|| {
                PlatformError::AppKitError("Failed to create CGDataProvider".to_string())
            })?;

            // CRITICAL: PremultipliedLast ensures standard premultiplied RGBA with true alpha blending!
            // Unlike softbuffer which hardcodes NoneSkipFirst (forcing opaque black background),
            // this enables genuine native alpha transparency composited by Quartz.
            let bitmap_info = CGBitmapInfo(
                CGImageAlphaInfo::PremultipliedLast.0 | CGImageByteOrderInfo::OrderDefault.0,
            );

            let cg_image = unsafe {
                CGImage::new(
                    width,
                    height,
                    8,
                    32,
                    width * 4,
                    Some(&self.color_space),
                    bitmap_info,
                    Some(&provider),
                    std::ptr::null(),
                    false,
                    CGColorRenderingIntent::RenderingIntentDefault,
                )
            }
            .ok_or_else(|| PlatformError::AppKitError("Failed to create CGImage".to_string()))?;

            // Set contents onto CALayer inside a transaction with disabled animations for 60fps responsiveness
            unsafe {
                CATransaction::begin();
                CATransaction::setDisableActions(true);
                layer.setContents(Some(cg_image.as_ref()));
                CATransaction::commit();
            }

            Ok(())
        })
    }

    fn get_global_cursor_pos(&self, window: &Window) -> Option<Point2D> {
        let loc = NSEvent::mouseLocation();
        let scale = window.scale_factor();
        let size = window.inner_size();
        let x_phys = (loc.x * scale) as f32;
        let y_phys = (size.height as f64 - loc.y * scale) as f32;
        Some(Point2D::new(x_phys, y_phys))
    }
}
