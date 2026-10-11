use winit::keyboard::{KeyCode, ModifiersState};

/// High-level user actions triggered by keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    Copy,
    Paste,
    Duplicate,
    Undo,
    Redo,
    Delete,
    Clear,
    ToggleMode,
    CycleTool,
    IncreaseFontSize,
    DecreaseFontSize,
}

/// Abstract contract for platform-specific keyboard shortcuts and modifier handling.
///
/// Follows Interface Segregation & Dependency Inversion principles:
/// Decouples keybinding logic, modifier interpretation, and UI shortcut display
/// from specific operating systems (e.g., macOS Command vs Linux/Windows Control).
pub trait PlatformKeymap: Send + Sync {
    /// Human-readable name of the primary command modifier (e.g. "Cmd" on macOS, "Ctrl" on Linux).
    fn primary_modifier_name(&self) -> &'static str;

    /// Returns true if the primary modifier key (Command on macOS, Control on Linux/Windows) is active.
    fn is_primary_modifier(&self, modifiers: &ModifiersState) -> bool;

    /// Returns true if the primary modifier key AND Shift are active (e.g. Cmd+Shift or Ctrl+Shift).
    fn is_primary_modifier_with_shift(&self, modifiers: &ModifiersState) -> bool;

    /// Returns true if no command modifier is pressed (allowing plain text input or single-key tool switches).
    fn has_no_command_modifier(&self, modifiers: &ModifiersState) -> bool;

    /// Checks if the given key and modifier state match a specific high-level action on this platform.
    fn matches_action(
        &self,
        action: ShortcutAction,
        modifiers: &ModifiersState,
        key: KeyCode,
    ) -> bool;

    /// Formats a shortcut string given the base key label (e.g. "C" -> "Cmd+C" on macOS, "Ctrl+C" on Linux).
    fn format_shortcut(&self, key: &str) -> String {
        format!("{}+{key}", self.primary_modifier_name())
    }

    /// Formats a template string containing `{cmd}` into the platform's primary modifier name.
    fn format_template(&self, template: &str) -> String {
        template.replace("{cmd}", self.primary_modifier_name())
    }
}

/// macOS keymap implementation:
/// Uses Command (Super) as primary modifier, with Control as secondary fallback.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacosKeymap;

impl PlatformKeymap for MacosKeymap {
    fn primary_modifier_name(&self) -> &'static str {
        "Cmd"
    }

    fn is_primary_modifier(&self, modifiers: &ModifiersState) -> bool {
        modifiers.super_key() || modifiers.control_key()
    }

    fn is_primary_modifier_with_shift(&self, modifiers: &ModifiersState) -> bool {
        self.is_primary_modifier(modifiers) && modifiers.shift_key()
    }

    fn has_no_command_modifier(&self, modifiers: &ModifiersState) -> bool {
        !modifiers.super_key() && !modifiers.control_key()
    }

    fn matches_action(
        &self,
        action: ShortcutAction,
        modifiers: &ModifiersState,
        key: KeyCode,
    ) -> bool {
        match action {
            ShortcutAction::Copy => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyC
            }
            ShortcutAction::Paste => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyV
            }
            ShortcutAction::Duplicate => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyD
            }
            ShortcutAction::Undo => !modifiers.shift_key() && key == KeyCode::KeyZ,
            ShortcutAction::Redo => {
                self.is_primary_modifier_with_shift(modifiers) && key == KeyCode::KeyZ
            }
            ShortcutAction::Delete => {
                !self.is_primary_modifier(modifiers)
                    && (key == KeyCode::Delete || key == KeyCode::Backspace)
            }
            ShortcutAction::Clear => {
                self.has_no_command_modifier(modifiers) && key == KeyCode::KeyC
            }
            ShortcutAction::ToggleMode => key == KeyCode::F1,
            ShortcutAction::CycleTool => key == KeyCode::F2,
            ShortcutAction::IncreaseFontSize => key == KeyCode::Equal || key == KeyCode::NumpadAdd,
            ShortcutAction::DecreaseFontSize => {
                key == KeyCode::Minus || key == KeyCode::NumpadSubtract
            }
        }
    }
}

/// Linux keymap implementation:
/// Uses Control as primary modifier. Supports both Ctrl+Shift+Z and Ctrl+Y for Redo.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxKeymap;

impl PlatformKeymap for LinuxKeymap {
    fn primary_modifier_name(&self) -> &'static str {
        "Ctrl"
    }

    fn is_primary_modifier(&self, modifiers: &ModifiersState) -> bool {
        modifiers.control_key()
    }

    fn is_primary_modifier_with_shift(&self, modifiers: &ModifiersState) -> bool {
        modifiers.control_key() && modifiers.shift_key()
    }

    fn has_no_command_modifier(&self, modifiers: &ModifiersState) -> bool {
        !modifiers.control_key() && !modifiers.super_key()
    }

    fn matches_action(
        &self,
        action: ShortcutAction,
        modifiers: &ModifiersState,
        key: KeyCode,
    ) -> bool {
        match action {
            ShortcutAction::Copy => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyC
            }
            ShortcutAction::Paste => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyV
            }
            ShortcutAction::Duplicate => {
                self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyD
            }
            ShortcutAction::Undo => !modifiers.shift_key() && key == KeyCode::KeyZ,
            ShortcutAction::Redo => {
                // Linux standard redo: Ctrl+Y OR Ctrl+Shift+Z
                (self.is_primary_modifier(modifiers)
                    && !modifiers.shift_key()
                    && key == KeyCode::KeyY)
                    || (self.is_primary_modifier_with_shift(modifiers) && key == KeyCode::KeyZ)
            }
            ShortcutAction::Delete => {
                !self.is_primary_modifier(modifiers)
                    && (key == KeyCode::Delete || key == KeyCode::Backspace)
            }
            ShortcutAction::Clear => {
                self.has_no_command_modifier(modifiers) && key == KeyCode::KeyC
            }
            ShortcutAction::ToggleMode => key == KeyCode::F1,
            ShortcutAction::CycleTool => key == KeyCode::F2,
            ShortcutAction::IncreaseFontSize => key == KeyCode::Equal || key == KeyCode::NumpadAdd,
            ShortcutAction::DecreaseFontSize => {
                key == KeyCode::Minus || key == KeyCode::NumpadSubtract
            }
        }
    }
}

/// Factory creating the default keymap for the active operating system.
pub fn default_platform_keymap() -> Box<dyn PlatformKeymap> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacosKeymap)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(LinuxKeymap)
    }
}
