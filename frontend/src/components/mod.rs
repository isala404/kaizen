mod board;
mod board_column;
mod detail_panel;
mod field_manager;
mod field_pills;
mod focus_dock;
mod header;
mod quick_capture;
mod task_card;
mod undo_toast;

pub use board::Board;
pub use board_column::BoardColumn;
pub use detail_panel::DetailPanel;
pub use field_manager::FieldManager;
pub use field_pills::FieldPills;
pub use focus_dock::FocusDock;
pub use header::Header;
pub use quick_capture::QuickCapture;
pub use task_card::TaskCard;
pub use undo_toast::{UndoAction, UndoToast};

use dioxus::prelude::*;

use crate::forge::TaskStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct DropTarget {
    pub status: TaskStatus,
    pub position: i32,
}

/// Shared context for touch-based hover detection on drop zones.
/// Updated during touch drag move via document::eval hit-testing.
#[derive(Clone, Copy)]
pub struct TouchHoverZone(pub Signal<Option<(String, i32)>>);

impl TouchHoverZone {
    pub fn matches(&self, status: &str, position: i32) -> bool {
        self.0
            .read()
            .as_ref()
            .is_some_and(|(s, p)| s == status && *p == position)
    }

    pub fn matches_status(&self, status: &str) -> bool {
        self.0.read().as_ref().is_some_and(|(s, _)| s == status)
    }
}
