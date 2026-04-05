use dioxus::prelude::*;

use crate::components::{BoardColumn, DropTarget};
use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};

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
    field_defs: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_create: EventHandler<(String, TaskStatus)>,
    on_drag_start: EventHandler<String>,
    on_drag_end: EventHandler<()>,
    on_drop: EventHandler<DropTarget>,
    on_touch_drag_start: Option<EventHandler<(String, f64, f64)>>,
    on_touch_drag_move: Option<EventHandler<(f64, f64)>>,
    on_touch_drag_end: Option<EventHandler<(f64, f64)>>,
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
                    field_defs: field_defs.clone(),
                    task_fields: task_fields.clone(),
                    on_select,
                    on_delete,
                    on_create,
                    on_drag_start,
                    on_drag_end,
                    on_drop,
                    on_touch_drag_start,
                    on_touch_drag_move,
                    on_touch_drag_end,
                }
            }
        }
    }
}
