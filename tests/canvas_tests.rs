use draw_rs::{
    ArrowShape, BitmapFont, CanvasController, CircleShape, Drawable, HudHitTarget, HudOverlay,
    HudRenderParams, Point2D, StrokeShape,
    app::{
        AppLanguage,
        state::{AppMode, DrawingTool, PaletteColor},
    },
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

    // Test macOS font loading with fontdue
    let font_bytes = std::fs::read("/System/Library/Fonts/Hiragino Sans GB.ttc")
        .or_else(|_| std::fs::read("/System/Library/Fonts/STHeiti Light.ttc"))
        .or_else(|_| std::fs::read("/System/Library/Fonts/STHeiti Medium.ttc"));

    if let Ok(bytes) = font_bytes {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default());
        assert!(font.is_ok());
        let font = font.unwrap();
        for ch in ['畫', '▼', '▲', '✓', '繁', '體'] {
            let (metrics, bitmap) = font.rasterize(ch, 16.0);
            assert!(metrics.width > 0 && metrics.height > 0);
            assert!(!bitmap.is_empty());
        }
    }
}

#[test]
fn test_palette_colors() {
    assert_eq!(PaletteColor::Cyan.name(), "Cyan");
    assert_eq!(PaletteColor::Emerald.name(), "Emerald");
    assert_eq!(PaletteColor::Coral.name(), "Coral");
    assert_eq!(PaletteColor::Amber.name(), "Amber");
    assert_eq!(PaletteColor::Violet.name(), "Violet");
}

#[test]
fn test_hud_menu_hit_testing() {
    let hud_pos = Point2D::new(50.0, 50.0);
    let scale = 1.0;
    let buttons = HudOverlay::get_buttons(hud_pos, scale);
    assert_eq!(buttons.len(), 13);

    // Test hitting each button at its center (when dropdown is closed)
    for btn in &buttons {
        let center = Point2D::new(btn.x + btn.width / 2.0, btn.y + btn.height / 2.0);
        let hit = HudOverlay::hit_test(hud_pos, center, scale, false);
        assert_eq!(hit, btn.target);
    }

    // Specific button target tests
    assert_eq!(buttons[0].target, HudHitTarget::ToggleMode);
    assert_eq!(buttons[1].target, HudHitTarget::Tool(DrawingTool::Pen));
    assert_eq!(buttons[2].target, HudHitTarget::Tool(DrawingTool::Circle));
    assert_eq!(buttons[3].target, HudHitTarget::Tool(DrawingTool::Arrow));
    assert_eq!(buttons[4].target, HudHitTarget::Tool(DrawingTool::Eraser));
    assert_eq!(buttons[5].target, HudHitTarget::Undo);
    assert_eq!(buttons[6].target, HudHitTarget::Clear);
    assert_eq!(buttons[7].target, HudHitTarget::Color(PaletteColor::Cyan));
    assert_eq!(
        buttons[8].target,
        HudHitTarget::Color(PaletteColor::Emerald)
    );
    assert_eq!(buttons[9].target, HudHitTarget::Color(PaletteColor::Coral));
    assert_eq!(buttons[10].target, HudHitTarget::Color(PaletteColor::Amber));
    assert_eq!(
        buttons[11].target,
        HudHitTarget::Color(PaletteColor::Violet)
    );
    assert_eq!(buttons[12].target, HudHitTarget::ToggleLanguageMenu);

    // Test dropdown popover items when menu is open
    let dropdown_items = HudOverlay::get_lang_dropdown_items(hud_pos, scale);
    assert_eq!(dropdown_items.len(), 2);
    assert_eq!(
        dropdown_items[0].target,
        HudHitTarget::SelectLanguage(AppLanguage::En)
    );
    assert_eq!(
        dropdown_items[1].target,
        HudHitTarget::SelectLanguage(AppLanguage::ZhTw)
    );

    for item in &dropdown_items {
        let center = Point2D::new(item.x + item.width / 2.0, item.y + item.height / 2.0);
        let hit = HudOverlay::hit_test(hud_pos, center, scale, true);
        assert_eq!(hit, item.target);
    }

    // Test blank card header area -> DragHeader
    let drag_point = Point2D::new(hud_pos.x + 10.0, hud_pos.y + 10.0);
    assert_eq!(
        HudOverlay::hit_test(hud_pos, drag_point, scale, false),
        HudHitTarget::DragHeader
    );

    // Test point far outside -> None
    let outside_point = Point2D::new(0.0, 0.0);
    assert_eq!(
        HudOverlay::hit_test(hud_pos, outside_point, scale, false),
        HudHitTarget::None
    );

    // Test contains_point with dropdown open vs closed
    let (mx, my, mw, mh) = HudOverlay::get_lang_dropdown_rect(hud_pos, scale);
    let dropdown_point = Point2D::new(mx + mw / 2.0, my + mh / 2.0);
    assert!(HudOverlay::contains_point(
        hud_pos,
        dropdown_point,
        scale,
        true
    ));

    // Test rendering HUD onto pixmap with both English (closed) and Traditional Chinese (open dropdown)
    let mut pixmap_en = Pixmap::new(600, 300).expect("failed to create pixmap");
    let params_en = HudRenderParams {
        mode: AppMode::Drawing,
        tool: DrawingTool::Pen,
        color: PaletteColor::Cyan,
        shape_count: 5,
        language: AppLanguage::En,
        is_lang_menu_open: false,
    };
    HudOverlay::render(&mut pixmap_en, &params_en, 1.0, hud_pos);
    assert!(pixmap_en.pixels().iter().any(|p| p.alpha() > 0));

    let mut pixmap_zh = Pixmap::new(600, 300).expect("failed to create pixmap");
    let params_zh = HudRenderParams {
        mode: AppMode::Drawing,
        tool: DrawingTool::Eraser,
        color: PaletteColor::Violet,
        shape_count: 3,
        language: AppLanguage::ZhTw,
        is_lang_menu_open: true,
    };
    HudOverlay::render(&mut pixmap_zh, &params_zh, 1.0, hud_pos);
    assert!(pixmap_zh.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn test_app_language_i18n() {
    let mut lang = AppLanguage::En;
    assert_eq!(lang.dropdown_label(false), "English ▼");
    assert_eq!(lang.dropdown_label(true), "English ▲");
    assert_eq!(
        AppLanguage::dropdown_item_label(AppLanguage::En, true),
        "✓ English"
    );
    assert_eq!(
        AppLanguage::dropdown_item_label(AppLanguage::ZhTw, false),
        "  繁體中文"
    );
    assert_eq!(lang.mode_title(AppMode::Drawing), "DRAWING");
    assert_eq!(lang.tool_name(DrawingTool::Eraser), "Eraser[E]");
    assert_eq!(lang.action_undo(), "Undo[Z]");
    assert_eq!(lang.action_clear(), "Clear[C]");

    lang = lang.toggle();
    assert_eq!(lang, AppLanguage::ZhTw);
    assert_eq!(lang.dropdown_label(false), "繁體中文 ▼");
    assert_eq!(lang.dropdown_label(true), "繁體中文 ▲");
    assert_eq!(lang.mode_title(AppMode::Drawing), "繪圖模式");
    assert_eq!(lang.tool_name(DrawingTool::Eraser), "橡皮擦[E]");
    assert_eq!(lang.action_undo(), "復原[Z]");
    assert_eq!(lang.action_clear(), "清除[C]");
    assert_eq!(lang.shape_count(5), "(5 個圖形)");

    lang = lang.toggle();
    assert_eq!(lang, AppLanguage::En);
}
