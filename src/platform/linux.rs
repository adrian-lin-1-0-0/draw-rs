#[cfg(target_os = "linux")]
mod imp {
    use memmap2::MmapMut;
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use std::fs::File;
    use std::num::NonZeroU32;
    use std::os::fd::{AsFd, AsRawFd};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use tiny_skia::Pixmap;
    use wayland_backend::client::{Backend, ObjectId};
    use wayland_client::globals::{GlobalListContents, registry_queue_init};
    use wayland_client::protocol::{wl_buffer, wl_registry, wl_shm, wl_shm_pool, wl_surface};
    use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
    use winit::window::{Window, WindowLevel};

    use super::super::{PlatformError, WindowPlatformController};
    use crate::models::point::Point2D;

    /// Container wrapping raw display and window handles with `'static` lifetime.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct RawHandleWrapper {
        display: RawDisplayHandle,
        window: RawWindowHandle,
    }

    // SAFETY: All window operations and handles on Linux occur exclusively on the main event loop thread.
    unsafe impl Send for RawHandleWrapper {}
    unsafe impl Sync for RawHandleWrapper {}

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

    // ---------------------------------------------------------------------------
    // Native Wayland ARGB8888 Transparent SHM Presenter
    // ---------------------------------------------------------------------------

    struct WaylandState;

    impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
        fn event(
            _: &mut WaylandState,
            _: &wl_registry::WlRegistry,
            _: wl_registry::Event,
            _: &GlobalListContents,
            _: &Connection,
            _: &QueueHandle<WaylandState>,
        ) {
        }
    }

    impl Dispatch<wl_shm::WlShm, ()> for WaylandState {
        fn event(
            _: &mut WaylandState,
            _: &wl_shm::WlShm,
            _: wl_shm::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<WaylandState>,
        ) {
        }
    }

    impl Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandState {
        fn event(
            _: &mut WaylandState,
            _: &wl_shm_pool::WlShmPool,
            _: wl_shm_pool::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<WaylandState>,
        ) {
        }
    }

    impl Dispatch<wl_buffer::WlBuffer, Arc<AtomicBool>> for WaylandState {
        fn event(
            _: &mut WaylandState,
            _: &wl_buffer::WlBuffer,
            event: wl_buffer::Event,
            released: &Arc<AtomicBool>,
            _: &Connection,
            _: &QueueHandle<WaylandState>,
        ) {
            if let wl_buffer::Event::Release = event {
                released.store(true, Ordering::SeqCst);
            }
        }
    }

    fn create_memfile() -> Result<File, PlatformError> {
        use rustix::fs::{MemfdFlags, SealFlags};

        let name = c"draw-rs-shm";
        match rustix::fs::memfd_create(name, MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING) {
            Ok(fd) => {
                let _ = rustix::fs::fcntl_add_seals(&fd, SealFlags::SHRINK | SealFlags::SEAL);
                Ok(File::from(fd))
            }
            Err(_) => {
                use rustix::fs::Mode;
                use rustix::shm::OFlags;
                let shm_name = format!("/draw-rs-shm-{}", std::process::id());
                let c_shm_name = std::ffi::CString::new(shm_name)
                    .map_err(|e| PlatformError::BackendError(e.to_string()))?;
                let fd = rustix::shm::open(
                    &c_shm_name,
                    OFlags::RDWR | OFlags::CREATE | OFlags::EXCL,
                    Mode::RWXU,
                )
                .map_err(|e| {
                    PlatformError::BackendError(format!("Failed to create POSIX shm: {e}"))
                })?;
                let _ = rustix::shm::unlink(&c_shm_name);
                Ok(File::from(fd))
            }
        }
    }

    struct WaylandBuffer {
        tempfile: File,
        map: MmapMut,
        pool: wl_shm_pool::WlShmPool,
        pool_size: i32,
        buffer: wl_buffer::WlBuffer,
        width: i32,
        height: i32,
        released: Arc<AtomicBool>,
        qh: QueueHandle<WaylandState>,
    }

    impl WaylandBuffer {
        fn new(
            shm: &wl_shm::WlShm,
            width: i32,
            height: i32,
            qh: &QueueHandle<WaylandState>,
        ) -> Result<Self, PlatformError> {
            let pool_size = ((width * height * 4) as u32).next_power_of_two() as i32;
            let tempfile = create_memfile()?;
            let _ = tempfile.set_len(pool_size as u64);
            let map = unsafe {
                MmapMut::map_mut(tempfile.as_raw_fd()).map_err(|e| {
                    PlatformError::BackendError(format!("Failed to map Wayland shared memory: {e}"))
                })?
            };

            let pool = shm.create_pool(tempfile.as_fd(), pool_size, qh, ());
            let released = Arc::new(AtomicBool::new(true));
            // CRITICAL: wl_shm::Format::Argb8888 provides per-pixel premultiplied alpha transparency.
            // Unlike softbuffer which hardcodes Xrgb8888 (forcing solid black background on Wayland),
            // Argb8888 allows the Wayland compositor (e.g. GNOME Mutter) to composite with zero artifacts!
            let buffer = pool.create_buffer(
                0,
                width,
                height,
                width * 4,
                wl_shm::Format::Argb8888,
                qh,
                released.clone(),
            );

            Ok(Self {
                tempfile,
                map,
                pool,
                pool_size,
                buffer,
                width,
                height,
                released,
                qh: qh.clone(),
            })
        }

        fn resize(&mut self, width: i32, height: i32) -> Result<(), PlatformError> {
            if self.width == width && self.height == height {
                return Ok(());
            }

            self.buffer.destroy();

            let needed = ((width * height * 4) as u32).next_power_of_two() as i32;
            if needed > self.pool_size {
                let _ = self.tempfile.set_len(needed as u64);
                self.pool.resize(needed);
                self.pool_size = needed;
                self.map = unsafe {
                    MmapMut::map_mut(self.tempfile.as_raw_fd()).map_err(|e| {
                        PlatformError::BackendError(format!(
                            "Failed to remap Wayland shared memory: {e}"
                        ))
                    })?
                };
            }

            self.buffer = self.pool.create_buffer(
                0,
                width,
                height,
                width * 4,
                wl_shm::Format::Argb8888,
                &self.qh,
                self.released.clone(),
            );
            self.width = width;
            self.height = height;

            Ok(())
        }

        fn mapped_mut(&mut self) -> &mut [u32] {
            let len = (self.width * self.height) as usize;
            unsafe { std::slice::from_raw_parts_mut(self.map.as_mut_ptr() as *mut u32, len) }
        }
    }

    impl Drop for WaylandBuffer {
        fn drop(&mut self) {
            self.buffer.destroy();
            self.pool.destroy();
        }
    }

    struct WaylandSurface {
        conn: Connection,
        event_queue: Mutex<EventQueue<WaylandState>>,
        qh: QueueHandle<WaylandState>,
        shm: wl_shm::WlShm,
        surface: wl_surface::WlSurface,
        buffers: Option<(WaylandBuffer, WaylandBuffer)>,
    }

    unsafe impl Send for WaylandSurface {}

    impl WaylandSurface {
        fn new(
            display_ptr: *mut std::ffi::c_void,
            surface_ptr: *mut std::ffi::c_void,
        ) -> Result<Self, PlatformError> {
            let backend = unsafe { Backend::from_foreign_display(display_ptr.cast()) };
            let conn = Connection::from_backend(backend);
            let (globals, event_queue) = registry_queue_init(&conn).map_err(|e| {
                PlatformError::BackendError(format!("Failed to initialize Wayland registry: {e}"))
            })?;
            let qh = event_queue.handle();
            let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ()).map_err(|e| {
                PlatformError::BackendError(format!("Failed to bind Wayland Shm global: {e}"))
            })?;

            let surface_id = unsafe {
                ObjectId::from_ptr(wl_surface::WlSurface::interface(), surface_ptr.cast())
            }
            .map_err(|e| {
                PlatformError::BackendError(format!("Failed to resolve Wayland surface ID: {e}"))
            })?;

            let surface = wl_surface::WlSurface::from_id(&conn, surface_id).map_err(|e| {
                PlatformError::BackendError(format!("Failed to create Wayland surface proxy: {e}"))
            })?;

            Ok(Self {
                conn,
                event_queue: Mutex::new(event_queue),
                qh,
                shm,
                surface,
                buffers: None,
            })
        }

        fn present_pixmap(&mut self, pixmap: &Pixmap) -> Result<(), PlatformError> {
            let width = pixmap.width() as i32;
            let height = pixmap.height() as i32;

            // 1. Dispatch pending events to catch buffer release notifications
            if let Ok(mut eq) = self.event_queue.lock() {
                let _ = eq.dispatch_pending(&mut WaylandState);
            }

            // 2. Prepare double buffers (resize or create)
            if let Some((front, back)) = &mut self.buffers {
                front.resize(width, height)?;
                back.resize(width, height)?;
            } else {
                let b1 = WaylandBuffer::new(&self.shm, width, height, &self.qh)?;
                let b2 = WaylandBuffer::new(&self.shm, width, height, &self.qh)?;
                self.buffers = Some((b1, b2));
            }

            let (front, back) = self.buffers.as_mut().expect("Buffers initialized");

            // Swap ping-pong buffers: back buffer will receive new pixels
            std::mem::swap(front, back);

            // 3. Blit premultiplied ARGB8888 pixels directly into mapped SHM buffer
            let slice = back.mapped_mut();
            let count = slice.len().min(pixmap.pixels().len());
            for (dst, src) in slice[..count].iter_mut().zip(&pixmap.pixels()[..count]) {
                let a = src.alpha() as u32;
                let r = src.red() as u32;
                let g = src.green() as u32;
                let b = src.blue() as u32;
                *dst = (a << 24) | (r << 16) | (g << 8) | b;
            }

            // 4. Attach, damage, and commit to Wayland surface
            back.released.store(false, Ordering::SeqCst);
            self.surface.attach(Some(&back.buffer), 0, 0);

            if self.surface.version() < 4 {
                self.surface.damage(0, 0, width, height);
            } else {
                self.surface.damage_buffer(0, 0, width, height);
            }

            self.surface.commit();

            let _ = self.conn.flush();

            Ok(())
        }
    }

    // ---------------------------------------------------------------------------
    // X11 32-bit Visual Fallback (softbuffer depth 32)
    // ---------------------------------------------------------------------------

    struct X11Surface {
        _context: softbuffer::Context<RawHandleWrapper>,
        surface: softbuffer::Surface<RawHandleWrapper, RawHandleWrapper>,
        width: u32,
        height: u32,
    }

    unsafe impl Send for X11Surface {}

    impl X11Surface {
        fn new(handles: RawHandleWrapper) -> Result<Self, PlatformError> {
            let context = softbuffer::Context::new(handles).map_err(|err| {
                PlatformError::BackendError(format!(
                    "Failed to initialize softbuffer context: {err}"
                ))
            })?;
            let surface = softbuffer::Surface::new(&context, handles).map_err(|err| {
                PlatformError::BackendError(format!(
                    "Failed to initialize softbuffer surface: {err}"
                ))
            })?;
            Ok(Self {
                _context: context,
                surface,
                width: 0,
                height: 0,
            })
        }

        fn present_pixmap(&mut self, pixmap: &Pixmap) -> Result<(), PlatformError> {
            let width = NonZeroU32::new(pixmap.width()).ok_or_else(|| {
                PlatformError::BackendError("Pixmap width must be non-zero".to_string())
            })?;
            let height = NonZeroU32::new(pixmap.height()).ok_or_else(|| {
                PlatformError::BackendError("Pixmap height must be non-zero".to_string())
            })?;

            if self.width != width.get() || self.height != height.get() {
                self.surface.resize(width, height).map_err(|err| {
                    PlatformError::BackendError(format!(
                        "Failed to resize softbuffer surface: {err}"
                    ))
                })?;
                self.width = width.get();
                self.height = height.get();
            }

            let mut buffer = self.surface.buffer_mut().map_err(|err| {
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
        }
    }

    // ---------------------------------------------------------------------------
    // Unified Linux Platform Controller
    // ---------------------------------------------------------------------------

    enum SurfaceBackend {
        Wayland(Box<WaylandSurface>),
        X11(X11Surface),
    }

    struct ActiveSurface {
        handles: RawHandleWrapper,
        backend: SurfaceBackend,
    }

    unsafe impl Send for ActiveSurface {}
    unsafe impl Sync for ActiveSurface {}

    /// Native window platform controller for Linux (supporting both X11 and Wayland compositors).
    pub struct LinuxPlatformController {
        surface_state: Mutex<Option<ActiveSurface>>,
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

        fn set_click_through(
            &self,
            window: &Window,
            click_through: bool,
        ) -> Result<(), PlatformError> {
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

            let mut lock = self.surface_state.lock().map_err(|_| {
                PlatformError::BackendError("Failed to lock surface mutex".to_string())
            })?;

            let needs_init = match lock.as_ref() {
                Some(active) => active.handles != current_handles,
                None => true,
            };

            if needs_init {
                let backend = match (display_raw, window_raw) {
                    (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => {
                        let wayland_surface =
                            WaylandSurface::new(d.display.as_ptr(), w.surface.as_ptr())?;
                        SurfaceBackend::Wayland(Box::new(wayland_surface))
                    }
                    (
                        RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_),
                        RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_),
                    ) => {
                        let x11_surface = X11Surface::new(current_handles)?;
                        SurfaceBackend::X11(x11_surface)
                    }
                    _ => return Err(PlatformError::InvalidWindowHandle),
                };

                *lock = Some(ActiveSurface {
                    handles: current_handles,
                    backend,
                });
            }

            let active = lock.as_mut().expect("Active surface must be initialized");
            match &mut active.backend {
                SurfaceBackend::Wayland(surface) => surface.present_pixmap(pixmap),
                SurfaceBackend::X11(surface) => surface.present_pixmap(pixmap),
            }
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
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::super::{PlatformError, WindowPlatformController};
    use crate::models::point::Point2D;
    use tiny_skia::Pixmap;
    use winit::window::Window;

    pub struct LinuxPlatformController;

    impl Default for LinuxPlatformController {
        fn default() -> Self {
            Self::new()
        }
    }

    impl LinuxPlatformController {
        pub fn new() -> Self {
            Self
        }
    }

    impl WindowPlatformController for LinuxPlatformController {
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

        fn keymap(&self) -> Box<dyn crate::platform::PlatformKeymap> {
            Box::new(crate::platform::LinuxKeymap)
        }
    }
}

pub use imp::LinuxPlatformController;
