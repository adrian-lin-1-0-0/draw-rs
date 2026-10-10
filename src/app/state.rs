use crate::models::{
    ArrowShape, CircleShape, CornerStyle, DiamondShape, Drawable, LineShape, Point2D, Rect2D,
    RectangleShape, Sloppiness, StrokeShape, StrokeStyle, TransformHandle,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawingTool {
    Selection,
    Hand,
    Rectangle,
    Diamond,
    Circle,
    Arrow,
    Line,
    #[default]
    Pen,
    Text,
    Eraser,
}

impl DrawingTool {
    /// Cycle to next tool preserving the legacy Pen -> Circle -> Arrow -> Eraser -> Pen sequence.
    pub fn cycle(&self) -> Self {
        match self {
            Self::Pen => Self::Circle,
            Self::Circle => Self::Arrow,
            Self::Arrow => Self::Eraser,
            Self::Eraser => Self::Pen,
            Self::Selection => Self::Hand,
            Self::Hand => Self::Rectangle,
            Self::Rectangle => Self::Diamond,
            Self::Diamond => Self::Line,
            Self::Line => Self::Text,
            Self::Text => Self::Pen,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Selection => "Select [V]",
            Self::Hand => "Hand [H]",
            Self::Rectangle => "Rectangle [R]",
            Self::Diamond => "Diamond [D]",
            Self::Circle => "Circle [O]",
            Self::Arrow => "Arrow [A]",
            Self::Line => "Line [L]",
            Self::Pen => "Pen [P]",
            Self::Text => "Text [T]",
            Self::Eraser => "Eraser [E]",
        }
    }
}

/// Curated color palette optimized for high-contrast diagramming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteColor {
    #[default]
    Cyan,
    Emerald,
    Coral,
    Amber,
    Violet,
}

impl PaletteColor {
    pub const ALL: [Self; 5] = [
        Self::Cyan,
        Self::Emerald,
        Self::Coral,
        Self::Amber,
        Self::Violet,
    ];

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
    DrawingRectangle {
        start: Point2D,
        current: Point2D,
    },
    DrawingDiamond {
        start: Point2D,
        current: Point2D,
    },
    DrawingLine {
        start: Point2D,
        current: Point2D,
    },
    BoxSelecting {
        start: Point2D,
        current: Point2D,
    },
    MovingSelection {
        start_pos: Point2D,
        last_pos: Point2D,
        initial_shapes: Vec<(usize, Box<dyn Drawable>)>,
    },
    ResizingSelection {
        handle: TransformHandle,
        start_bounds: Rect2D,
        start_mouse: Point2D,
        initial_shapes: Vec<(usize, Box<dyn Drawable>)>,
    },
    RotatingSelection {
        center: Point2D,
        start_angle: f32,
        initial_shapes: Vec<(usize, Box<dyn Drawable>)>,
    },
    Erasing {
        removed: Vec<(usize, Box<dyn Drawable>)>,
    },
    DraggingHud {
        drag_offset: Point2D,
    },
    DraggingInspector {
        drag_offset: Point2D,
    },
    PanningCanvas {
        last_pos: Point2D,
        initial_shapes: Vec<(usize, Box<dyn Drawable>)>,
    },
}

impl DragState {
    /// Produces a live preview drawable object using legacy palette settings.
    pub fn to_preview_drawable(
        &self,
        color: PaletteColor,
        stroke_width: f32,
    ) -> Option<Box<dyn Drawable>> {
        self.to_preview_drawable_styled(
            color.to_color(),
            Some(
                Color::from_rgba(
                    color.to_color().red(),
                    color.to_color().green(),
                    color.to_color().blue(),
                    0.25,
                )
                .unwrap_or(Color::TRANSPARENT),
            ),
            stroke_width,
            StrokeStyle::Solid,
            Sloppiness::Artist,
            CornerStyle::Sharp,
            1.0,
        )
    }

    /// Produces a live preview drawable object with custom styling properties.
    #[allow(clippy::too_many_arguments)]
    pub fn to_preview_drawable_styled(
        &self,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
        stroke_style: StrokeStyle,
        sloppiness: Sloppiness,
        corner_style: CornerStyle,
        opacity: f32,
    ) -> Option<Box<dyn Drawable>> {
        match self {
            Self::Idle
            | Self::Erasing { .. }
            | Self::DraggingHud { .. }
            | Self::DraggingInspector { .. }
            | Self::BoxSelecting { .. }
            | Self::MovingSelection { .. }
            | Self::ResizingSelection { .. }
            | Self::RotatingSelection { .. }
            | Self::PanningCanvas { .. } => None,

            Self::DrawingStroke(points) => {
                if points.is_empty() {
                    None
                } else {
                    let mut s = StrokeShape::new(points.clone(), stroke_color, stroke_width);
                    s.stroke_style = stroke_style;
                    s.sloppiness = sloppiness;
                    s.opacity = opacity;
                    Some(Box::new(s))
                }
            }

            Self::DrawingCircle { start, current } => {
                let radius = start.distance(current);
                let mut c = CircleShape::with_translucent_fill(
                    *start,
                    radius,
                    stroke_color,
                    stroke_width,
                    0.25,
                );
                c.fill_color = fill_color;
                c.stroke_style = stroke_style;
                c.sloppiness = sloppiness;
                c.opacity = opacity;
                Some(Box::new(c))
            }

            Self::DrawingArrow { start, current } => {
                let mut a = ArrowShape::new(*start, *current, stroke_color, stroke_width);
                a.stroke_style = stroke_style;
                a.sloppiness = sloppiness;
                a.opacity = opacity;
                Some(Box::new(a))
            }

            Self::DrawingRectangle { start, current } => {
                let min_x = start.x.min(current.x);
                let min_y = start.y.min(current.y);
                let w = (current.x - start.x).abs();
                let h = (current.y - start.y).abs();
                let mut r = RectangleShape::new(
                    Point2D::new(min_x, min_y),
                    w,
                    h,
                    stroke_color,
                    fill_color,
                    stroke_width,
                );
                r.corner_style = corner_style;
                r.stroke_style = stroke_style;
                r.sloppiness = sloppiness;
                r.opacity = opacity;
                Some(Box::new(r))
            }

            Self::DrawingDiamond { start, current } => {
                let center = start.midpoint(current);
                let w = (current.x - start.x).abs();
                let h = (current.y - start.y).abs();
                let mut d = DiamondShape::new(center, w, h, stroke_color, fill_color, stroke_width);
                d.stroke_style = stroke_style;
                d.sloppiness = sloppiness;
                d.opacity = opacity;
                Some(Box::new(d))
            }

            Self::DrawingLine { start, current } => {
                let mut l = LineShape::new(*start, *current, stroke_color, stroke_width);
                l.stroke_style = stroke_style;
                l.sloppiness = sloppiness;
                l.opacity = opacity;
                Some(Box::new(l))
            }
        }
    }
}
