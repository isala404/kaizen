mod board;
mod board_column;
mod detail_panel;
mod field_manager;
mod focus_dock;
mod header;
mod quick_capture;
mod task_card;
mod undo_toast;

pub use board::Board;
pub use board_column::BoardColumn;
pub use detail_panel::DetailPanel;
pub use field_manager::FieldManager;
pub use focus_dock::FocusDock;
pub use header::Header;
pub use quick_capture::QuickCapture;
pub use task_card::TaskCard;
pub use undo_toast::{UndoAction, UndoToast};

use crate::forge::TaskStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct DropTarget {
    pub status: TaskStatus,
    pub position: i32,
}
