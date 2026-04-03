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

use crate::forge::TaskStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusChange {
    pub id: String,
    pub status: TaskStatus,
}
