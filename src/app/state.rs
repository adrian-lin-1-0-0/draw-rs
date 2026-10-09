use crate::models::{
    Drawable, arrow::ArrowShape, circle::CircleShape, point::Point2D, stroke::StrokeShape,
};
use tiny_skia::Color;

/// Operational mode of the overlay application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Active drawing mode: Window captures mouse input for drawing doodles and shapes.
    Drawing,
    /// Passive click-through mode: Mouse clicks pass directly through to background applications.
    ClickThrough,
}

impl AppMode {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Drawing => Self::ClickThrough,
            Self::ClickThrough => Self::Drawing,
        }
    }
}

/// Active drawing tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingTool {
    Pen,
    Circle,
    Arrow,
    Eraser,
}

impl DrawingTool {
    /// Cycle to the next tool in sequence (Pen -> Circle -> Arrow -> Eraser -> Pen).
    pub fn cycle(&self) -> Self {
        match self {
            Self::Pen => Self::Circle,
            Self::Circle => Self::Arrow,
            Self::Arrow => Self::Eraser,
            Self::Eraser => Self::Pen,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pen => "Pen",
            Self::Circle => "Circle (Node)",
            Self::Arrow => "Arrow (Pointer)",
            Self::Eraser => "Eraser",
        }
    }
}

/// Curated color palette optimized for high-contrast algorithm diagramming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteColor {
    Cyan,
    Emerald,
    Coral,
    Amber,
    Violet,
}

impl PaletteColor {
    pub fn to_color(self) -> Color {
        match self {
            Self::Cyan => Color::from_rgba8(56, 189, 248, 255), // #38BDF8
            Self::Emerald => Color::from_rgba8(52, 211, 153, 255), // #34D399
            Self::Coral => Color::from_rgba8(248, 113, 113, 255), // #F87171
            Self::Amber => Color::from_rgba8(251, 191, 36, 255), // #FBBF24
            Self::Violet => Color::from_rgba8(192, 132, 252, 255), // #C084FC
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Cyan => "Cyan",
            Self::Emerald => "Emerald",
            Self::Coral => "Coral",
            Self::Amber => "Amber",
            Self::Violet => "Violet",
        }
    }
}

/// State of an active mouse dragging operation.
pub enum DragState {
    Idle,
    DrawingStroke(Vec<Point2D>),
    DrawingCircle {
        start: Point2D,
        current: Point2D,
    },
    DrawingArrow {
        start: Point2D,
        current: Point2D,
    },
    Erasing {
        removed: Vec<(usize, Box<dyn Drawable>)>,
    },
    DraggingHud {
        drag_offset: Point2D,
    },
}

impl DragState {
    /// Produces a live preview drawable object if currently in an active drawing drag.
    pub fn to_preview_drawable(
        &self,
        color: PaletteColor,
        stroke_width: f32,
    ) -> Option<Box<dyn Drawable>> {
        match self {
            Self::Idle | Self::Erasing { .. } | Self::DraggingHud { .. } => None,
            Self::DrawingStroke(points) => {
                if points.is_empty() {
                    None
                } else {
                    Some(Box::new(StrokeShape::new(
                        points.clone(),
                        color.to_color(),
                        stroke_width,
                    )))
                }
            }
            Self::DrawingCircle { start, current } => {
                let radius = start.distance(current);
                Some(Box::new(CircleShape::with_translucent_fill(
                    *start,
                    radius,
                    color.to_color(),
                    stroke_width,
                    0.25, // 25% fill opacity
                )))
            }
            Self::DrawingArrow { start, current } => Some(Box::new(ArrowShape::new(
                *start,
                *current,
                color.to_color(),
                stroke_width,
            ))),
        }
    }
}
