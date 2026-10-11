use draw_rs::WindowPlatformController;
use draw_rs::app::{AppEvent, DrawApp};
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use draw_rs::platform::DummyPlatformController;
#[cfg(target_os = "linux")]
use draw_rs::platform::LinuxPlatformController;
#[cfg(target_os = "macos")]
use draw_rs::platform::MacosPlatformController;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Dependency Inversion: Inject concrete platform controller into DrawApp
    #[cfg(target_os = "macos")]
    let platform: Box<dyn WindowPlatformController> = Box::new(MacosPlatformController::new());
    #[cfg(target_os = "linux")]
    let platform: Box<dyn WindowPlatformController> = Box::new(LinuxPlatformController::new());
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let platform: Box<dyn WindowPlatformController> = Box::new(DummyPlatformController::new());

    let keymap = platform.keymap();

    println!("============================================================");
    #[cfg(target_os = "macos")]
    println!("      draw-rs: macOS LeetCode Transparent Doodle Board      ");
    #[cfg(target_os = "linux")]
    println!("      draw-rs: Linux LeetCode Transparent Doodle Board      ");
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    println!("      draw-rs: Transparent Doodle Board                     ");
    println!("============================================================");
    println!("Hotkeys & Controls:");
    println!("  [F1]        : Toggle Draw Mode <-> Click-Through Mode (Global)");
    println!("  [F2]        : Cycle Tool (Pen -> Circle -> Arrow -> Eraser)");
    println!("  [E]         : Switch directly to Eraser (橡皮擦)");
    println!("  [P]         : Switch directly to Pen (手繪筆)");
    println!("  [1 - 5]     : Switch Colors (Cyan, Emerald, Coral, Amber, Violet)");
    println!(
        "  [{}] : Copy & Paste selected shapes (複製 / 貼上)",
        keymap.format_shortcut("C / V")
    );
    println!(
        "  [{}]     : Duplicate selected shapes (再製)",
        keymap.format_shortcut("D")
    );
    println!("  [Z]         : Undo last action (Draw, Erase, Clear)");
    println!("  [C]         : Clear all doodles");
    println!("  [Esc]       : Exit application");
    println!("  [Drag HUD]  : Click & Drag HUD card anytime (even in pass-through!)");
    println!("------------------------------------------------------------");
    #[cfg(target_os = "macos")]
    {
        println!("macOS Transparency & Permissions Note:");
        println!("  • Window background is 100% natively transparent via Quartz Compositor.");
        println!("  • To allow global F1 hotkey toggling while typing in IDE/browser,");
        println!("    ensure Terminal/draw-rs has 'Accessibility' permission if prompted.");
        println!("    (System Settings -> Privacy & Security -> Accessibility)");
    }
    #[cfg(target_os = "linux")]
    {
        println!("Linux Transparency & Window Manager Note:");
        println!("  • True alpha transparency requires a compositing window manager");
        println!("    (e.g., Picom / Compton on X11, or native Wayland compositors).");
        println!("  • Global F1 hotkey toggling is supported across your desktop environment.");
    }
    println!("============================================================");

    // 2. Initialize event loop with custom AppEvent support
    let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();

    // 3. Setup global hotkey for F1 (allowing mode toggling while unfocused / clicking through)
    match GlobalHotKeyManager::new() {
        Ok(hotkey_manager) => {
            let hotkey_f1 = HotKey::new(None, Code::F1);
            if let Err(err) = hotkey_manager.register(hotkey_f1) {
                eprintln!("[draw-rs] Warning: Could not register global F1 hotkey: {err}");
            } else {
                let hotkey_f1_id = hotkey_f1.id();
                std::thread::spawn(move || {
                    let receiver = GlobalHotKeyEvent::receiver();
                    while let Ok(event) = receiver.recv() {
                        if event.id == hotkey_f1_id && event.state == HotKeyState::Released {
                            let _ = proxy.send_event(AppEvent::ToggleMode);
                        }
                    }
                });
            }
        }
        Err(err) => {
            eprintln!(
                "[draw-rs] Notice: Global hotkey manager not available on this desktop server: {err}"
            );
            eprintln!("          (In-window F1 hotkey remains fully operational)");
        }
    }

    let mut app = DrawApp::new(platform);

    // 4. Run application event loop
    event_loop.run_app(&mut app)?;

    Ok(())
}
