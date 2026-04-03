mod board;
mod board_column;
mod detail_panel;
mod field_filter_bar;
mod field_manager;
mod field_pills;
mod focus_dock;
mod header;
mod quick_capture;
mod status_tabs;
mod task_card;
mod task_list;

pub use board::Board;
pub use board_column::BoardColumn;
pub use detail_panel::DetailPanel;
pub use field_filter_bar::{ActiveFilters, FieldFilterBar};
pub use field_manager::FieldManager;
pub use field_pills::FieldPills;
pub use focus_dock::FocusDock;
pub use header::Header;
pub use quick_capture::QuickCapture;
pub use status_tabs::StatusTabs;
pub use task_card::TaskCard;
pub use task_list::TaskList;

mod swipeable_card;
mod undo_toast;

pub use swipeable_card::SwipeableCard;
pub use undo_toast::{UndoAction, UndoToast};

use crate::forge::TaskStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusChange {
    pub id: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragPayload {
    pub task_id: String,
    pub source_status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTarget {
    pub status: TaskStatus,
    pub position: i32,
}
