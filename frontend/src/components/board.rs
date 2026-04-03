use dioxus::prelude::*;

use crate::components::{BoardColumn, StatusChange};
use crate::forge::{Task, TaskStatus};

const COLUMN_STATUSES: [(&str, TaskStatus); 5] = [
    ("Inbox", TaskStatus::Inbox),
    ("Up next", TaskStatus::UpNext),
    ("In progress", TaskStatus::InProgress),
    ("Paused", TaskStatus::Paused),
    ("Done", TaskStatus::Done),
];

#[component]
pub fn Board(
    tasks: Vec<Task>,
    focused_col: Option<usize>,
    focused_row: Option<usize>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
    on_create: EventHandler<String>,
    on_status_change: EventHandler<StatusChange>,
) -> Element {
    rsx! {
        div { class: "section-label", "BOARD" }
        div { class: "board-columns",
            for (i, (label, status)) in COLUMN_STATUSES.iter().enumerate() {
                BoardColumn {
                    key: "{label}",
                    label: label.to_string(),
                    status: status.clone(),
                    is_focused: focused_col == Some(i),
                    focused_row: if focused_col == Some(i) { focused_row } else { None },
                    tasks: tasks.iter()
                        .filter(|t| t.status == *status)
                        .cloned()
                        .collect::<Vec<_>>(),
                    on_select,
                    on_delete,
                    on_focus,
                    on_create,
                    on_status_change,
                }
            }
        }
    }
}
