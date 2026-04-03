use dioxus::prelude::*;

use crate::components::{BoardColumn, StatusChange};
use crate::forge::{Task, TaskStatus};

#[component]
pub fn Board(
    tasks: Vec<Task>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
    on_create: EventHandler<String>,
    on_status_change: EventHandler<StatusChange>,
) -> Element {
    let columns = vec![
        ("Inbox", TaskStatus::Inbox),
        ("Up next", TaskStatus::UpNext),
        ("In progress", TaskStatus::InProgress),
        ("Paused", TaskStatus::Paused),
        ("Done", TaskStatus::Done),
    ];

    rsx! {
        div { class: "section-label", "BOARD" }
        div { class: "board-columns",
            for (label, status) in columns {
                BoardColumn {
                    key: "{label}",
                    label: label.to_string(),
                    status: status.clone(),
                    tasks: tasks.iter()
                        .filter(|t| t.status == status)
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
