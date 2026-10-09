use crate::app::state::{AppMode, DrawingTool};

/// Application language for UI localization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLanguage {
    #[default]
    En,
    ZhTw,
}

impl AppLanguage {
    /// Toggle between English and Traditional Chinese.
    pub fn toggle(&self) -> Self {
        match self {
            Self::En => Self::ZhTw,
            Self::ZhTw => Self::En,
        }
    }

    /// Language code display for HUD toggle button.
    pub fn lang_btn_label(&self) -> &'static str {
        match self {
            Self::En => "EN/中[L]",
            Self::ZhTw => "中/EN[L]",
        }
    }

    /// Localized mode title in HUD header.
    pub fn mode_title(&self, mode: AppMode) -> &'static str {
        match (self, mode) {
            (Self::En, AppMode::Drawing) => "DRAWING",
            (Self::En, AppMode::ClickThrough) => "PASS-THRU",
            (Self::ZhTw, AppMode::Drawing) => "繪圖模式",
            (Self::ZhTw, AppMode::ClickThrough) => "滑鼠穿透",
        }
    }

    /// Localized shape count string.
    pub fn shape_count(&self, count: usize) -> String {
        match self {
            Self::En => format!("({count} shapes)"),
            Self::ZhTw => format!("({count} 個圖形)"),
        }
    }

    /// Localized mode toggle button label.
    pub fn mode_toggle_btn(&self, mode: AppMode) -> &'static str {
        match (self, mode) {
            (Self::En, AppMode::Drawing) => "[Pass-Thru F1]",
            (Self::En, AppMode::ClickThrough) => "[Draw Mode F1]",
            (Self::ZhTw, AppMode::Drawing) => "[切換穿透 F1]",
            (Self::ZhTw, AppMode::ClickThrough) => "[切換繪圖 F1]",
        }
    }

    /// Localized drag handle indicator.
    pub fn drag_hint(&self) -> &'static str {
        match self {
            Self::En => "::: Drag",
            Self::ZhTw => "::: 拖移",
        }
    }

    /// Localized tools row label.
    pub fn tools_label(&self) -> &'static str {
        match self {
            Self::En => "TOOLS:",
            Self::ZhTw => "工具:",
        }
    }

    /// Localized tool button label.
    pub fn tool_name(&self, tool: DrawingTool) -> &'static str {
        match (self, tool) {
            (Self::En, DrawingTool::Pen) => "Pen[P]",
            (Self::En, DrawingTool::Circle) => "Circle",
            (Self::En, DrawingTool::Arrow) => "Arrow",
            (Self::En, DrawingTool::Eraser) => "Eraser[E]",
            (Self::ZhTw, DrawingTool::Pen) => "畫筆[P]",
            (Self::ZhTw, DrawingTool::Circle) => "節點",
            (Self::ZhTw, DrawingTool::Arrow) => "箭頭",
            (Self::ZhTw, DrawingTool::Eraser) => "橡皮擦[E]",
        }
    }

    /// Localized Undo action label.
    pub fn action_undo(&self) -> &'static str {
        match self {
            Self::En => "Undo[Z]",
            Self::ZhTw => "復原[Z]",
        }
    }

    /// Localized Clear action label.
    pub fn action_clear(&self) -> &'static str {
        match self {
            Self::En => "Clear[C]",
            Self::ZhTw => "清除[C]",
        }
    }

    /// Localized color row label.
    pub fn color_label(&self) -> &'static str {
        match self {
            Self::En => "COLOR:",
            Self::ZhTw => "顏色:",
        }
    }
}
