use draw_rs::{
    AppLanguage, ArrowShape, CanvasController, CircleShape, CornerStyle, DiamondShape, Drawable,
    InspectorHitTarget, InspectorOverlay, InspectorRenderParams, LineShape, Point2D, Rect2D,
    RectangleShape, SelectionRenderer, Sloppiness, StrokeStyle, TextShape, TransformHandle,
};
use tiny_skia::{Color, Pixmap};

#[test]
fn test_rectangle_shape_geometry_and_rough_rendering() {
    let mut rect = RectangleShape::new(
        Point2D::new(50.0, 50.0),
        100.0,
        60.0,
        Color::from_rgba8(56, 189, 248, 255),
        Some(Color::from_rgba8(56, 189, 248, 64)),
        3.0,
    );

    let bb = rect.bounding_box().expect("bounding box should exist");
    assert_eq!(bb.min_x, 50.0);
    assert_eq!(bb.min_y, 50.0);
    assert_eq!(bb.width(), 100.0);
    assert_eq!(bb.height(), 60.0);

    // Hit test interior (since filled)
    assert!(rect.hit_test(Point2D::new(80.0, 80.0)));
    // Hit test outside
    assert!(!rect.hit_test(Point2D::new(200.0, 200.0)));

    // Intersects edge
    assert!(rect.intersects(Point2D::new(50.0, 70.0), 5.0));

    // Test Round corners
    rect.set_corner_style(CornerStyle::Round);
    assert_eq!(rect.corner_style(), CornerStyle::Round);

    // Test Sloppiness
    rect.set_sloppiness(Sloppiness::Cartoonist);
    assert_eq!(rect.sloppiness(), Sloppiness::Cartoonist);

    // Test rasterization
    let mut pixmap = Pixmap::new(200, 200).expect("failed pixmap");
    rect.draw(&mut pixmap);
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));

    // Test resize
    let old_bounds = Rect2D::new(50.0, 50.0, 150.0, 110.0);
    let new_bounds = Rect2D::new(100.0, 100.0, 300.0, 220.0);
    rect.resize(old_bounds, new_bounds);
    let new_bb = rect.bounding_box().unwrap();
    assert_eq!(new_bb.min_x, 100.0);
    assert_eq!(new_bb.min_y, 100.0);
    assert_eq!(new_bb.width(), 200.0);
    assert_eq!(new_bb.height(), 120.0);
}

#[test]
fn test_diamond_shape_geometry() {
    let mut diamond = DiamondShape::new(
        Point2D::new(100.0, 100.0),
        80.0,
        60.0,
        Color::from_rgba8(52, 211, 153, 255),
        Some(Color::from_rgba8(52, 211, 153, 50)),
        2.5,
    );

    let bb = diamond.bounding_box().expect("diamond bb");
    assert_eq!(bb.center(), Point2D::new(100.0, 100.0));
    assert_eq!(bb.width(), 80.0);
    assert_eq!(bb.height(), 60.0);

    // Center should hit
    assert!(diamond.hit_test(Point2D::new(100.0, 100.0)));
    // Corner should hit
    assert!(diamond.hit_test(Point2D::new(140.0, 100.0)));
    // Far corner of bounding box (outside diamond) should not hit
    assert!(!diamond.hit_test(Point2D::new(138.0, 128.0)));

    // Test translate
    diamond.translate(Point2D::new(20.0, 30.0));
    assert_eq!(diamond.center, Point2D::new(120.0, 130.0));

    let mut pixmap = Pixmap::new(250, 250).unwrap();
    diamond.draw(&mut pixmap);
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn test_line_and_arrow_polyline() {
    // Line
    let line = LineShape::new(
        Point2D::new(10.0, 10.0),
        Point2D::new(90.0, 90.0),
        Color::from_rgba8(248, 113, 113, 255),
        3.0,
    );
    assert!(line.hit_test(Point2D::new(50.0, 50.0)));
    assert!(!line.hit_test(Point2D::new(10.0, 90.0)));

    let mut pixmap = Pixmap::new(120, 120).unwrap();
    line.draw(&mut pixmap);
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));

    // Arrow with polyline waypoints (elbow connector)
    let waypoints = vec![
        Point2D::new(10.0, 10.0),
        Point2D::new(60.0, 10.0),
        Point2D::new(60.0, 80.0),
    ];
    let arrow = ArrowShape::with_waypoints(waypoints, Color::from_rgba8(192, 132, 252, 255), 3.5);
    // Should hit horizontal segment
    assert!(arrow.hit_test(Point2D::new(35.0, 10.0)));
    // Should hit vertical segment
    assert!(arrow.hit_test(Point2D::new(60.0, 45.0)));
    // Should hit arrowhead tip
    assert!(arrow.hit_test(Point2D::new(60.0, 80.0)));

    let mut arrow_pixmap = Pixmap::new(120, 120).unwrap();
    arrow.draw(&mut arrow_pixmap);
    assert!(arrow_pixmap.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn test_circle_and_ellipse() {
    // True ellipse
    let ellipse = CircleShape::new_ellipse(
        Point2D::new(100.0, 100.0),
        60.0,
        30.0,
        Color::from_rgba8(251, 191, 36, 255),
        Some(Color::from_rgba8(251, 191, 36, 60)),
        2.0,
    );
    assert_eq!(ellipse.rx(), 60.0);
    assert_eq!(ellipse.ry(), 30.0);

    // Center is inside
    assert!(ellipse.is_inside(Point2D::new(100.0, 100.0)));
    // Horizontal edge is inside
    assert!(ellipse.is_inside(Point2D::new(150.0, 100.0)));
    // Point outside ellipse boundary
    assert!(!ellipse.is_inside(Point2D::new(150.0, 125.0)));

    let bb = ellipse.bounding_box().unwrap();
    assert_eq!(bb.width(), 120.0);
    assert_eq!(bb.height(), 60.0);
}

#[test]
fn test_text_shape_vector_rendering() {
    let mut text = TextShape::new(
        Point2D::new(20.0, 20.0),
        "Algorithm\nWhiteboard",
        22.0,
        Color::from_rgba8(255, 255, 255, 255),
    );

    let (w, h) = text.measure();
    assert!(w > 20.0);
    assert!(h > 20.0);

    assert!(text.hit_test(Point2D::new(25.0, 25.0)));

    let mut pixmap = Pixmap::new(200, 100).unwrap();
    text.draw(&mut pixmap);
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));

    // Translate
    text.translate(Point2D::new(10.0, 15.0));
    assert_eq!(text.position, Point2D::new(30.0, 35.0));
}

#[test]
fn test_deterministic_seed_avoids_flicker() {
    let seed = 987654321;
    let rect1 = RectangleShape {
        top_left: Point2D::new(20.0, 20.0),
        width: 80.0,
        height: 50.0,
        corner_radius: 8.0,
        stroke_color: Color::from_rgba8(56, 189, 248, 255),
        fill_color: None,
        stroke_width: 2.0,
        stroke_style: StrokeStyle::Solid,
        sloppiness: Sloppiness::Artist,
        corner_style: CornerStyle::Sharp,
        opacity: 1.0,
        seed,
    };
    let rect2 = rect1.clone();

    let mut pixmap1 = Pixmap::new(120, 100).unwrap();
    let mut pixmap2 = Pixmap::new(120, 100).unwrap();

    rect1.draw(&mut pixmap1);
    rect2.draw(&mut pixmap2);

    // Exact pixel identity across redraws: identical seeds => identical pixels!
    assert_eq!(pixmap1.data(), pixmap2.data());
}

#[test]
fn test_selection_and_transform_handles() {
    let bounds = Rect2D::new(100.0, 100.0, 300.0, 200.0);
    assert_eq!(bounds.center(), Point2D::new(200.0, 150.0));
    assert_eq!(bounds.width(), 200.0);
    assert_eq!(bounds.height(), 100.0);

    // Check 9 handles
    let handles = bounds.handles();
    assert_eq!(handles.len(), 9);

    // NW handle at (100, 100)
    assert_eq!(
        bounds.hit_test_handle(Point2D::new(100.0, 100.0), 6.0),
        Some(TransformHandle::Nw)
    );
    // SE handle at (300, 200)
    assert_eq!(
        bounds.hit_test_handle(Point2D::new(300.0, 200.0), 6.0),
        Some(TransformHandle::Se)
    );
    // Rotate handle at (200, 76)
    assert_eq!(
        bounds.hit_test_handle(Point2D::new(200.0, 76.0), 6.0),
        Some(TransformHandle::Rotate)
    );
    // Center is not a handle
    assert_eq!(
        bounds.hit_test_handle(Point2D::new(200.0, 150.0), 6.0),
        None
    );

    // Test SelectionRenderer
    let mut pixmap = Pixmap::new(400, 300).unwrap();
    SelectionRenderer::render_selection_box(&mut pixmap, bounds, 1.0);
    SelectionRenderer::render_marquee(
        &mut pixmap,
        Point2D::new(10.0, 10.0),
        Point2D::new(80.0, 80.0),
        1.0,
    );
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn test_full_undo_redo_and_duplicate_and_delete() {
    let mut canvas = CanvasController::new();

    // 1. Push 2 shapes
    let r1 = RectangleShape::new(
        Point2D::new(10.0, 10.0),
        50.0,
        50.0,
        Color::from_rgba8(255, 0, 0, 255),
        None,
        2.0,
    );
    let r2 = RectangleShape::new(
        Point2D::new(100.0, 100.0),
        50.0,
        50.0,
        Color::from_rgba8(0, 255, 0, 255),
        None,
        2.0,
    );
    canvas.push_shape(Box::new(r1));
    canvas.push_shape(Box::new(r2));
    assert_eq!(canvas.shape_count(), 2);

    // 2. Duplicate shape at index 0
    let dup_indices = canvas.duplicate(&[0], Point2D::new(20.0, 20.0));
    assert_eq!(dup_indices, vec![2]);
    assert_eq!(canvas.shape_count(), 3);

    // 3. Undo duplicate
    assert!(canvas.undo());
    assert_eq!(canvas.shape_count(), 2);

    // 4. Redo duplicate
    assert!(canvas.redo());
    assert_eq!(canvas.shape_count(), 3);

    // 5. Delete shape at index 1
    canvas.delete_selected(&[1]);
    assert_eq!(canvas.shape_count(), 2);

    // 6. Undo delete
    assert!(canvas.undo());
    assert_eq!(canvas.shape_count(), 3);

    // 7. Layer reordering: Bring index 0 to front
    let new_idx = canvas.bring_to_front(&[0]);
    assert_eq!(new_idx, vec![2]);

    // 8. Undo bring to front
    assert!(canvas.undo());
}

#[test]
fn test_inspector_hud_hit_testing_and_rendering() {
    let inspector_pos = Point2D::new(24.0, 150.0);
    let scale = 1.0;

    assert!(InspectorOverlay::contains_point(
        inspector_pos,
        Point2D::new(50.0, 200.0),
        scale
    ));
    assert!(!InspectorOverlay::contains_point(
        inspector_pos,
        Point2D::new(400.0, 200.0),
        scale
    ));

    // Header drag test
    let header_point = Point2D::new(inspector_pos.x + 20.0, inspector_pos.y + 15.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, header_point, scale),
        InspectorHitTarget::DragHeader
    );

    // Header close button test
    let close_point = Point2D::new(inspector_pos.x + 215.0, inspector_pos.y + 15.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, close_point, scale),
        InspectorHitTarget::Close
    );

    // Stroke width buttons test
    let width_point = Point2D::new(inspector_pos.x + 40.0, inspector_pos.y + 155.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, width_point, scale),
        InspectorHitTarget::StrokeWidth(1.5)
    );

    // Quick Actions Duplicate & Delete test
    let dup_point = Point2D::new(inspector_pos.x + 50.0, inspector_pos.y + 295.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, dup_point, scale),
        InspectorHitTarget::Duplicate
    );

    let del_point = Point2D::new(inspector_pos.x + 160.0, inspector_pos.y + 295.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, del_point, scale),
        InspectorHitTarget::Delete
    );

    // Render inspector onto pixmap (English)
    let mut pixmap = Pixmap::new(300, 750).unwrap();
    let params = InspectorRenderParams {
        selected_count: 2,
        stroke_color: Color::from_rgba8(56, 189, 248, 255),
        fill_color: Some(Color::from_rgba8(56, 189, 248, 60)),
        stroke_width: 3.5,
        stroke_style: StrokeStyle::Solid,
        language: AppLanguage::En,
    };
    InspectorOverlay::render(&mut pixmap, &params, scale, inspector_pos);
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));

    // Render inspector onto pixmap (Traditional Chinese)
    let params_zh = InspectorRenderParams {
        selected_count: 0,
        language: AppLanguage::ZhTw,
        ..params
    };
    let mut pixmap_zh = Pixmap::new(300, 750).unwrap();
    InspectorOverlay::render(&mut pixmap_zh, &params_zh, scale, inspector_pos);
    assert!(pixmap_zh.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn test_diamond_and_hand_manipulation() {
    let mut canvas = CanvasController::new();
    let diamond = DiamondShape::new(
        Point2D::new(100.0, 100.0),
        80.0,
        50.0,
        Color::from_rgba8(56, 189, 248, 255),
        Some(Color::from_rgba8(56, 189, 248, 40)),
        2.0,
    );
    canvas.push_shape(Box::new(diamond));
    assert_eq!(canvas.shape_count(), 1);

    // Hit test diamond shape
    let hit_idx = canvas.hit_test_all(Point2D::new(100.0, 100.0));
    assert_eq!(hit_idx, Some(0));

    // Manipulate (move / adjust via translate)
    let shape = canvas.shape_mut(0).expect("shape should exist");
    shape.translate(Point2D::new(50.0, 30.0));

    let bounds = canvas.shape(0).unwrap().bounding_box().unwrap();
    assert_eq!(bounds.center(), Point2D::new(150.0, 130.0));
}

#[test]
fn test_canvas_pan_and_text_interactive_input() {
    let mut canvas = CanvasController::new();

    // 1. Add text shape with no initial text
    let text = TextShape::new(
        Point2D::new(40.0, 40.0),
        "",
        24.0,
        Color::from_rgba8(56, 189, 248, 255),
    );
    canvas.push_shape(Box::new(text));
    assert_eq!(canvas.shape(0).unwrap().text_content(), Some(""));

    // 2. User types into text shape
    let shape = canvas.shape_mut(0).unwrap();
    shape.set_text_content("Interactive Text Input".to_string());
    assert_eq!(
        canvas.shape(0).unwrap().text_content(),
        Some("Interactive Text Input")
    );

    // 3. Add another shape to test canvas panning (Hand tool)
    let rect = RectangleShape::new(
        Point2D::new(100.0, 100.0),
        50.0,
        50.0,
        Color::from_rgba8(255, 255, 255, 255),
        None,
        2.0,
    );
    canvas.push_shape(Box::new(rect));

    // Pan entire canvas by (30.0, -20.0)
    canvas.translate_all(Point2D::new(30.0, -20.0));

    let text_bb = canvas.shape(0).unwrap().bounding_box().unwrap();
    assert_eq!(text_bb.min_x, 66.0);
    assert_eq!(text_bb.min_y, 16.0);

    let rect_bb = canvas.shape(1).unwrap().bounding_box().unwrap();
    assert_eq!(rect_bb.min_x, 130.0);
    assert_eq!(rect_bb.min_y, 80.0);
}
