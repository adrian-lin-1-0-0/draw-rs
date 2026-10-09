use serde::Deserialize;
use std::sync::LazyLock;

use crate::app::state::{AppMode, DrawingTool};

const EN_YAML: &str = include_str!("../../locales/en.yaml");
const ZH_TW_YAML: &str = include_str!("../../locales/zh-TW.yaml");

static EN_STRINGS: LazyLock<LocaleStrings> =
    LazyLock::new(|| serde_yaml::from_str(EN_YAML).expect("valid en.yaml locale"));
static ZH_TW_STRINGS: LazyLock<LocaleStrings> =
    LazyLock::new(|| serde_yaml::from_str(ZH_TW_YAML).expect("valid zh-TW.yaml locale"));

#[derive(Debug, Clone, Deserialize)]
pub struct ModeStrings {
    pub drawing: String,
    pub click_through: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShapeStrings {
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ButtonStrings {
    pub to_drawing: String,
    pub to_passthru: String,
    pub undo: String,
    pub clear: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HintStrings {
    pub drag: String,
    pub color_keys: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LabelStrings {
    pub tools: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolStrings {
    pub pen: String,
    pub circle: String,
    pub arrow: String,
    pub eraser: String,
}

/// Strongly-typed locale strings loaded from compile-time embedded JSON files.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleStrings {
    pub display_name: String,
    pub mode: ModeStrings,
    pub shapes: ShapeStrings,
    pub buttons: ButtonStrings,
    pub hints: HintStrings,
    pub labels: LabelStrings,
    pub tools: ToolStrings,
}

/// Application language for UI localization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLanguage {
    #[default]
    En,
    ZhTw,
}

impl AppLanguage {
    pub const ALL: [Self; 2] = [Self::En, Self::ZhTw];

    /// Access the embedded deserialized locale strings for this language.
    pub fn strings(&self) -> &'static LocaleStrings {
        match self {
            Self::En => &EN_STRINGS,
            Self::ZhTw => &ZH_TW_STRINGS,
        }
    }

    /// Toggle between English and Traditional Chinese.
    pub fn toggle(&self) -> Self {
        match self {
            Self::En => Self::ZhTw,
            Self::ZhTw => Self::En,
        }
    }

    /// Native display name for this language.
    pub fn display_name(&self) -> &str {
        &self.strings().display_name
    }

    /// Label display for HUD dropdown trigger button.
    pub fn dropdown_label(&self, is_open: bool) -> String {
        let arrow = if is_open { "▲" } else { "▼" };
        format!("{} {arrow}", self.display_name())
    }

    /// Item label inside the dropdown menu popover.
    pub fn dropdown_item_label(target_lang: Self, is_active: bool) -> String {
        let prefix = if is_active { "✓ " } else { "  " };
        format!("{prefix}{}", target_lang.display_name())
    }

    /// Localized mode title in HUD header.
    pub fn mode_title(&self, mode: AppMode) -> &str {
        match mode {
            AppMode::Drawing => &self.strings().mode.drawing,
            AppMode::ClickThrough => &self.strings().mode.click_through,
        }
    }

    /// Localized shape count string.
    pub fn shape_count(&self, count: usize) -> String {
        self.strings()
            .shapes
            .format
            .replace("{count}", &count.to_string())
    }

    /// Localized mode toggle button label.
    pub fn mode_toggle_btn(&self, mode: AppMode) -> &str {
        match mode {
            AppMode::Drawing => &self.strings().buttons.to_passthru,
            AppMode::ClickThrough => &self.strings().buttons.to_drawing,
        }
    }

    /// Localized drag handle indicator.
    pub fn drag_hint(&self) -> &str {
        &self.strings().hints.drag
    }

    /// Localized tools row label.
    pub fn tools_label(&self) -> &str {
        &self.strings().labels.tools
    }

    /// Localized tool button label.
    pub fn tool_name(&self, tool: DrawingTool) -> &str {
        match tool {
            DrawingTool::Pen => &self.strings().tools.pen,
            DrawingTool::Circle => &self.strings().tools.circle,
            DrawingTool::Arrow => &self.strings().tools.arrow,
            DrawingTool::Eraser => &self.strings().tools.eraser,
        }
    }

    /// Localized Undo action label.
    pub fn action_undo(&self) -> &str {
        &self.strings().buttons.undo
    }

    /// Localized Clear action label.
    pub fn action_clear(&self) -> &str {
        &self.strings().buttons.clear
    }

    /// Localized color row label.
    pub fn color_label(&self) -> &str {
        &self.strings().labels.color
    }

    /// Localized color keys shortcut hint.
    pub fn color_keys_hint(&self) -> &str {
        &self.strings().hints.color_keys
    }
}
