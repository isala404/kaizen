use dioxus::prelude::*;

use crate::components::{BoardColumn, DropTarget};
use crate::forge::{Task, TaskStatus};

const COLUMN_STATUSES: [(&str, TaskStatus); 4] = [
    ("Inbox", TaskStatus::Inbox),
    ("Up next", TaskStatus::UpNext),
    ("Paused", TaskStatus::Paused),
    ("Done", TaskStatus::Done),
];

#[component]
pub fn Board(
    tasks: Vec<Task>,
    focused_col: Option<usize>,
    focused_row: Option<usize>,
    dragging_id: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_create: EventHandler<(String, TaskStatus)>,
    on_drag_start: EventHandler<String>,
    on_drag_end: EventHandler<()>,
    on_drop: EventHandler<DropTarget>,
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
                    tasks: {
                        let mut col: Vec<_> = tasks.iter()
                            .filter(|t| t.status == *status)
                            .cloned()
                            .collect();
                        col.sort_by_key(|t| t.position);
                        col
                    },
                    dragging_id: dragging_id.clone(),
                    on_select,
                    on_delete,
                    on_create,
                    on_drag_start,
                    on_drag_end,
                    on_drop,
                }
            }
        }
    }
}
