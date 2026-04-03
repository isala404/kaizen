use dioxus::prelude::*;

use crate::components::{BoardColumn, DropTarget, StatusChange};
use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};

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
    field_defs: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
    focused_col: Option<usize>,
    focused_row: Option<usize>,
    dragging_id: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
    on_create: EventHandler<String>,
    on_status_change: EventHandler<StatusChange>,
    on_drag_start: EventHandler<String>,
    on_drag_end: EventHandler<()>,
    on_drop: EventHandler<DropTarget>,
) -> Element {
    let total = tasks.len();
    rsx! {
        div { class: "section-label", "BOARD" }
        div { class: "board-columns",
            for (i, (label, status)) in COLUMN_STATUSES.iter().enumerate() {
                BoardColumn {
                    key: "{label}",
                    label: label.to_string(),
                    status: status.clone(),
                    total_task_count: total,
                    is_focused: focused_col == Some(i),
                    focused_row: if focused_col == Some(i) { focused_row } else { None },
                    tasks: tasks.iter()
                        .filter(|t| t.status == *status)
                        .cloned()
                        .collect::<Vec<_>>(),
                    field_defs: field_defs.clone(),
                    task_fields: task_fields.clone(),
                    dragging_id: dragging_id.clone(),
                    on_select,
                    on_delete,
                    on_focus,
                    on_create,
                    on_status_change,
                    on_drag_start,
                    on_drag_end,
                    on_drop,
                }
            }
        }
    }
}
