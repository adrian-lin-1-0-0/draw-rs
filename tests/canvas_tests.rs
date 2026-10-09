use draw_rs::{
    ArrowShape, BitmapFont, CanvasController, CircleShape, Drawable, Point2D, StrokeShape,
    app::state::{AppMode, DrawingTool, PaletteColor},
};
use tiny_skia::{Color, Pixmap};

#[test]
fn test_point2d_geometry() {
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(3.0, 4.0);

    assert_eq!(p1.distance(&p2), 5.0);
    assert_eq!(p1.midpoint(&p2), Point2D::new(1.5, 2.0));
    assert_eq!(p1 + p2, Point2D::new(3.0, 4.0));
    assert_eq!(p2 - p1, Point2D::new(3.0, 4.0));

    // Distance to segment test
    let a = Point2D::new(0.0, 0.0);
    let b = Point2D::new(10.0, 0.0);

    // Perpendicular projection directly onto segment
    let p_mid = Point2D::new(5.0, 3.0);
    assert!((p_mid.distance_to_segment(&a, &b) - 3.0).abs() < 1e-4);

    // Closest to end point 'a'
    let p_before = Point2D::new(-4.0, 0.0);
    assert!((p_before.distance_to_segment(&a, &b) - 4.0).abs() < 1e-4);

    // Closest to end point 'b'
    let p_after = Point2D::new(15.0, 0.0);
    assert!((p_after.distance_to_segment(&a, &b) - 5.0).abs() < 1e-4);
}

#[test]
fn test_canvas_controller_history_and_undo() {
    let mut canvas = CanvasController::new();
    assert_eq!(canvas.shape_count(), 0);
    assert!(canvas.is_empty());

    let stroke = StrokeShape::new(
        vec![Point2D::new(10.0, 10.0), Point2D::new(50.0, 50.0)],
        Color::from_rgba8(255, 0, 0, 255),
        2.0,
    );
    canvas.push_shape(Box::new(stroke));
    assert_eq!(canvas.shape_count(), 1);
    assert!(!canvas.is_empty());

    let circle = CircleShape::with_translucent_fill(
        Point2D::new(100.0, 100.0),
        30.0,
        Color::from_rgba8(0, 255, 0, 255),
        2.0,
        0.5,
    );
    canvas.push_shape(Box::new(circle));
    assert_eq!(canvas.shape_count(), 2);

    // Test Undo
    assert!(canvas.undo());
    assert_eq!(canvas.shape_count(), 1);

    // Test Clear
    canvas.clear();
    assert_eq!(canvas.shape_count(), 0);
    assert!(canvas.is_empty());
}

#[test]
fn test_eraser_hit_testing() {
    let stroke = StrokeShape::new(
        vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(100.0, 0.0),
            Point2D::new(100.0, 100.0),
        ],
        Color::from_rgba8(255, 255, 255, 255),
        4.0,
    );

    // Point near horizontal segment
    assert!(stroke.intersects(Point2D::new(50.0, 5.0), 10.0));
    // Point far away
    assert!(!stroke.intersects(Point2D::new(50.0, 50.0), 10.0));

    let circle = CircleShape::with_translucent_fill(
        Point2D::new(200.0, 200.0),
        30.0,
        Color::from_rgba8(255, 255, 255, 255),
        2.0,
        0.2,
    );

    // Point inside circle
    assert!(circle.intersects(Point2D::new(200.0, 200.0), 5.0));
    // Point slightly outside circle border but within radius
    assert!(circle.intersects(Point2D::new(235.0, 200.0), 10.0));
    // Point completely outside
    assert!(!circle.intersects(Point2D::new(300.0, 300.0), 10.0));

    let arrow = ArrowShape::new(
        Point2D::new(0.0, 0.0),
        Point2D::new(50.0, 0.0),
        Color::from_rgba8(255, 255, 255, 255),
        3.0,
    );

    // Point near arrow shaft
    assert!(arrow.intersects(Point2D::new(25.0, 2.0), 5.0));
    // Point near arrow tip
    assert!(arrow.intersects(Point2D::new(48.0, 1.0), 5.0));
    // Point far away
    assert!(!arrow.intersects(Point2D::new(25.0, 50.0), 5.0));
}

#[test]
fn test_eraser_action_and_undo() {
    let mut canvas = CanvasController::new();

    let stroke1 = StrokeShape::new(
        vec![Point2D::new(10.0, 10.0), Point2D::new(20.0, 20.0)],
        Color::from_rgba8(255, 0, 0, 255),
        2.0,
    );
    let stroke2 = StrokeShape::new(
        vec![Point2D::new(100.0, 100.0), Point2D::new(200.0, 200.0)],
        Color::from_rgba8(0, 255, 0, 255),
        2.0,
    );
    canvas.push_shape(Box::new(stroke1));
    canvas.push_shape(Box::new(stroke2));
    assert_eq!(canvas.shape_count(), 2);

    // Erase stroke1 at (15, 15)
    let erased = canvas.erase_at(Point2D::new(15.0, 15.0), 10.0);
    assert_eq!(erased.len(), 1);
    assert_eq!(canvas.shape_count(), 1);

    // Commit erase action to history
    canvas.commit_erase(erased);

    // Undo should restore stroke1 back into the canvas
    assert!(canvas.undo());
    assert_eq!(canvas.shape_count(), 2);

    // Erase with nothing in range should be empty
    let empty_erased = canvas.erase_at(Point2D::new(500.0, 500.0), 10.0);
    assert!(empty_erased.is_empty());
}

#[test]
fn test_drawable_rasterization() {
    let mut pixmap = Pixmap::new(200, 200).expect("failed to create pixmap");

    // Test Stroke
    let stroke = StrokeShape::new(
        vec![
            Point2D::new(10.0, 10.0),
            Point2D::new(50.0, 20.0),
            Point2D::new(90.0, 80.0),
        ],
        Color::from_rgba8(56, 189, 248, 255),
        4.0,
    );
    stroke.draw(&mut pixmap);

    // Test Circle
    let circle = CircleShape::with_translucent_fill(
        Point2D::new(100.0, 100.0),
        40.0,
        Color::from_rgba8(52, 211, 153, 255),
        3.0,
        0.25,
    );
    circle.draw(&mut pixmap);

    // Test Arrow
    let arrow = ArrowShape::new(
        Point2D::new(20.0, 150.0),
        Point2D::new(160.0, 150.0),
        Color::from_rgba8(248, 113, 113, 255),
        3.5,
    );
    arrow.draw(&mut pixmap);

    // Verify non-zero alpha in pixmap (pixels were drawn)
    let has_colored_pixel = pixmap.pixels().iter().any(|p| p.alpha() > 0);
    assert!(has_colored_pixel);
}

#[test]
fn test_state_transitions() {
    let mut mode = AppMode::Drawing;
    mode = mode.toggle();
    assert_eq!(mode, AppMode::ClickThrough);
    mode = mode.toggle();
    assert_eq!(mode, AppMode::Drawing);

    let mut tool = DrawingTool::Pen;
    tool = tool.cycle();
    assert_eq!(tool, DrawingTool::Circle);
    tool = tool.cycle();
    assert_eq!(tool, DrawingTool::Arrow);
    tool = tool.cycle();
    assert_eq!(tool, DrawingTool::Eraser);
    tool = tool.cycle();
    assert_eq!(tool, DrawingTool::Pen);
}

#[test]
fn test_font_rendering() {
    let mut pixmap = Pixmap::new(100, 50).expect("failed to create pixmap");
    BitmapFont::draw_text(
        &mut pixmap,
        5.0,
        5.0,
        "TEST 123",
        Color::from_rgba8(255, 255, 255, 255),
        1.0,
    );

    let has_pixels = pixmap.pixels().iter().any(|p| p.alpha() > 0);
    assert!(has_pixels);

    let (w, h) = BitmapFont::measure_text("ABC", 1.0);
    assert!(w > 0.0 && h > 0.0);
}

#[test]
fn test_palette_colors() {
    assert_eq!(PaletteColor::Cyan.name(), "Cyan");
    assert_eq!(PaletteColor::Emerald.name(), "Emerald");
    assert_eq!(PaletteColor::Coral.name(), "Coral");
    assert_eq!(PaletteColor::Amber.name(), "Amber");
    assert_eq!(PaletteColor::Violet.name(), "Violet");
}
