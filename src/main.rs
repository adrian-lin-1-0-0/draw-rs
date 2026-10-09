use draw_rs::{
    app::{AppEvent, DrawApp},
    platform::MacosPlatformController,
};
use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("      draw-rs: macOS LeetCode Transparent Doodle Board      ");
    println!("============================================================");
    println!("Hotkeys & Controls:");
    println!("  [F1]        : Toggle Draw Mode <-> Click-Through Mode (Global)");
    println!("  [F2]        : Cycle Tool (Pen -> Circle -> Arrow)");
    println!("  [1 - 5]     : Switch Colors (Cyan, Emerald, Coral, Amber, Violet)");
    println!("  [Z]         : Undo last shape");
    println!("  [C]         : Clear all doodles");
    println!("  [Esc]       : Exit application");
    println!("------------------------------------------------------------");
    println!("macOS Transparency & Permissions Note:");
    println!("  • Window background is 100% natively transparent via Quartz Compositor.");
    println!("  • To allow global F1 hotkey toggling while typing in IDE/browser,");
    println!("    ensure Terminal/draw-rs has 'Accessibility' permission if prompted.");
    println!("    (System Settings -> Privacy & Security -> Accessibility)");
    println!("============================================================");

    // 1. Initialize event loop with custom AppEvent support
    let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();

    // 2. Setup global hotkey for F1 (allowing mode toggling while unfocused / clicking through)
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let hotkey_f1 = HotKey::new(None, Code::F1);
    hotkey_manager.register(hotkey_f1)?;

    let hotkey_f1_id = hotkey_f1.id();
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if event.id == hotkey_f1_id && event.state == HotKeyState::Released {
                let _ = proxy.send_event(AppEvent::ToggleMode);
            }
        }
    });

    // 3. Dependency Inversion: Inject concrete platform controller into DrawApp
    let platform = Box::new(MacosPlatformController::new());
    let mut app = DrawApp::new(platform);

    // 4. Run application event loop
    event_loop.run_app(&mut app)?;

    Ok(())
}
