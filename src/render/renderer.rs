use tiny_skia::{Color, Pixmap};

/// Manages the tiny-skia 2D rasterization surface and transfers pixels to softbuffer.
///
/// Adheres to SRP: Exclusively handles raster buffer allocation, clearing, and pixel blitting.
pub struct CanvasRenderer {
    pixmap: Pixmap,
    width: u32,
    height: u32,
}

impl CanvasRenderer {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let pixmap = Pixmap::new(width.max(1), height.max(1))?;
        Some(Self {
            pixmap,
            width: width.max(1),
            height: height.max(1),
        })
    }

    /// Resize the backing pixmap if the window dimensions changed.
    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }
        if let Some(new_pixmap) = Pixmap::new(width, height) {
            self.pixmap = new_pixmap;
            self.width = width;
            self.height = height;
        }
    }

    /// Clear the pixmap to 100% transparent pixels (alpha = 0).
    pub fn clear(&mut self) {
        self.pixmap.fill(Color::TRANSPARENT);
    }

    /// Mutable reference to the backing tiny-skia pixmap for drawing operations.
    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }

    /// Current width of the raster surface.
    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Current height of the raster surface.
    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Blit the premultiplied RGBA tiny-skia pixmap pixels into softbuffer's 0xAARRGGBB buffer.
    #[allow(dead_code)]
    pub fn blit_to_buffer(&self, buffer: &mut [u32]) {
        let count = buffer.len().min(self.pixmap.pixels().len());
        for (dst, src) in buffer[..count]
            .iter_mut()
            .zip(&self.pixmap.pixels()[..count])
        {
            let a = src.alpha() as u32;
            let r = src.red() as u32;
            let g = src.green() as u32;
            let b = src.blue() as u32;
            *dst = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }
}
