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

    // 9. Multi-selection layer movement tests (A=0, B=1, C=2)
    // Add a 4th shape so we have 4 shapes: A, B, C, D
    let r4 = RectangleShape::new(
        Point2D::new(150.0, 150.0),
        30.0,
        30.0,
        Color::from_rgba8(0, 0, 255, 255),
        None,
        2.0,
    );
    canvas.push_shape(Box::new(r4));
    assert_eq!(canvas.shape_count(), 4);

    // Multi-selection bring_forward: move adjacent shapes [1, 2] (B, C) forward as a cohesive block
    let fwd_indices = canvas.bring_forward(&[1, 2]);
    assert_eq!(fwd_indices, vec![2, 3]);

    // Undo bring_forward
    assert!(canvas.undo());

    // Multi-selection send_backward: move adjacent shapes [1, 2] (B, C) backward as a cohesive block
    let back_indices = canvas.send_backward(&[1, 2]);
    assert_eq!(back_indices, vec![0, 1]);

    // Send backward at boundary (0, 1 cannot move further back)
    let back_boundary = canvas.send_backward(&[0, 1]);
    assert_eq!(back_boundary, vec![0, 1]);

    // Undo send_backward
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

    // Quick Actions Copy & Paste test
    let copy_point = Point2D::new(inspector_pos.x + 50.0, inspector_pos.y + 325.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, copy_point, scale),
        InspectorHitTarget::Copy
    );

    let paste_point = Point2D::new(inspector_pos.x + 160.0, inspector_pos.y + 325.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, paste_point, scale),
        InspectorHitTarget::Paste
    );

    // Hitbox boundary tests: Row 1 is 288..=312, Row 2 is 318..=342.
    // 313..317 is the gap between Row 1 and Row 2, and >342 is below Row 2.
    let gap_point = Point2D::new(inspector_pos.x + 50.0, inspector_pos.y + 314.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, gap_point, scale),
        InspectorHitTarget::None
    );

    let below_row2 = Point2D::new(inspector_pos.x + 50.0, inspector_pos.y + 343.0);
    assert_eq!(
        InspectorOverlay::hit_test(inspector_pos, below_row2, scale),
        InspectorHitTarget::None
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

#[test]
fn test_canvas_copy_and_paste() {
    let mut canvas = CanvasController::new();
    assert!(!canvas.has_clipboard());
    assert_eq!(canvas.clipboard_count(), 0);

    // Paste with empty clipboard should do nothing
    let pasted_empty = canvas.paste(Point2D::new(20.0, 20.0));
    assert!(pasted_empty.is_empty());

    // Add shapes to canvas
    let rect = RectangleShape::new(
        Point2D::new(50.0, 50.0),
        100.0,
        60.0,
        Color::from_rgba8(255, 0, 0, 255),
        None,
        2.0,
    );
    let text = TextShape::new(
        Point2D::new(200.0, 200.0),
        "Copy Me",
        24.0,
        Color::from_rgba8(0, 255, 0, 255),
    );
    canvas.push_shape(Box::new(rect));
    canvas.push_shape(Box::new(text));
    assert_eq!(canvas.shape_count(), 2);

    // 1. Copy selected shapes (both index 0 and 1)
    let copied_count = canvas.copy_selected(&[0, 1]);
    assert_eq!(copied_count, 2);
    assert!(canvas.has_clipboard());
    assert_eq!(canvas.clipboard_count(), 2);

    // 2. First Paste: offset by (20.0, 20.0) * 1 = (20.0, 20.0)
    let new_indices_1 = canvas.paste(Point2D::new(20.0, 20.0));
    assert_eq!(new_indices_1, vec![2, 3]);
    assert_eq!(canvas.shape_count(), 4);

    let pasted_rect_bb = canvas.shape(2).unwrap().bounding_box().unwrap();
    assert_eq!(pasted_rect_bb.min_x, 70.0);
    assert_eq!(pasted_rect_bb.min_y, 70.0);

    assert_eq!(canvas.shape(3).unwrap().text_content(), Some("Copy Me"));

    // 3. Second Paste: offset by (20.0, 20.0) * 2 = (40.0, 40.0) from original
    let new_indices_2 = canvas.paste(Point2D::new(20.0, 20.0));
    assert_eq!(new_indices_2, vec![4, 5]);
    assert_eq!(canvas.shape_count(), 6);

    let pasted_rect_bb2 = canvas.shape(4).unwrap().bounding_box().unwrap();
    assert_eq!(pasted_rect_bb2.min_x, 90.0);
    assert_eq!(pasted_rect_bb2.min_y, 90.0);

    // 4. Undo the second paste
    assert!(canvas.undo());
    assert_eq!(canvas.shape_count(), 4);

    // 5. Redo the second paste
    assert!(canvas.redo());
    assert_eq!(canvas.shape_count(), 6);
}

#[test]
fn test_text_enlargement_beyond_previous_limit() {
    // 1. Verify text shape can be instantiated with large font size (> 120.0)
    let mut large_text = TextShape::new(
        Point2D::new(10.0, 10.0),
        "Huge Header",
        250.0,
        Color::from_rgba8(255, 255, 255, 255),
    );
    assert_eq!(large_text.font_size, 250.0);
    assert_eq!(large_text.font_size(), Some(250.0));

    // 2. Test enlargement via resize handles from old bounds to 2x larger bounds
    let old_bounds = large_text.bounding_box().unwrap();
    let new_bounds = Rect2D::from_xywh(
        old_bounds.min_x,
        old_bounds.min_y,
        old_bounds.width() * 2.0,
        old_bounds.height() * 2.0,
    );
    large_text.resize(old_bounds, new_bounds);
    assert!((large_text.font_size - 500.0).abs() < 1.0);

    // 3. Scale up to maximum limit (1000.0)
    let old_b2 = large_text.bounding_box().unwrap();
    let new_b2 = Rect2D::from_xywh(
        old_b2.min_x,
        old_b2.min_y,
        old_b2.width() * 3.0,
        old_b2.height() * 3.0,
    );
    large_text.resize(old_b2, new_b2);
    assert_eq!(large_text.font_size, 1000.0); // clamped at MAX_FONT_SIZE (1000.0)

    // 4. Test horizontal resizing also scales font size
    let old_b3 = large_text.bounding_box().unwrap();
    let new_b3 = Rect2D::from_xywh(
        old_b3.min_x,
        old_b3.min_y,
        old_b3.width() * 0.5,
        old_b3.height(),
    );
    large_text.resize(old_b3, new_b3);
    assert!((large_text.font_size - 500.0).abs() < 1.0);

    // 5. Verify stroke_width setting (e.g. from general color/style change) does not crush font size
    large_text.set_stroke_width(2.5);
    assert_eq!(large_text.font_size, 500.0);
    // And verify stroke_width() does not return font size (1000 or 500), avoiding canvas stroke_width pollution
    assert_ne!(large_text.stroke_width(), 500.0);

    // 6. Test set_font_size method
    large_text.set_font_size(600.0);
    assert_eq!(large_text.font_size, 600.0);
}

#[test]
fn test_text_font_size_scaling_undoable() {
    let mut canvas = CanvasController::new();
    let text = TextShape::new(
        Point2D::new(50.0, 50.0),
        "Undo Scaling Test",
        20.0,
        Color::from_rgba8(255, 255, 255, 255),
    );
    canvas.push_shape(Box::new(text));
    assert_eq!(canvas.shape(0).unwrap().font_size(), Some(20.0));

    // Simulate font scaling with commit_transform
    let orig = vec![(0, canvas.shape(0).unwrap().clone_box())];
    canvas.shape_mut(0).unwrap().set_font_size(24.0);
    let modified = vec![(0, canvas.shape(0).unwrap().clone_box())];
    canvas.commit_transform(orig, modified);

    assert_eq!(canvas.shape(0).unwrap().font_size(), Some(24.0));

    // Undo should restore 20.0
    assert!(canvas.undo());
    assert_eq!(canvas.shape(0).unwrap().font_size(), Some(20.0));

    // Redo should restore 24.0
    assert!(canvas.redo());
    assert_eq!(canvas.shape(0).unwrap().font_size(), Some(24.0));
}
