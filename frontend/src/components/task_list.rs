use dioxus::prelude::*;

use crate::components::{StatusChange, TaskCard};
use crate::forge::{Task, TaskStatus};

#[component]
pub fn TaskList(
    tasks: Vec<Task>,
    active_status: TaskStatus,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
    on_status_change: EventHandler<StatusChange>,
) -> Element {
    let filtered: Vec<&Task> = tasks.iter().filter(|t| t.status == active_status).collect();

    let empty_hint = match active_status {
        TaskStatus::Inbox => "Tap + to add a task",
        TaskStatus::UpNext => "Swipe tasks here from Inbox",
        TaskStatus::InProgress => "Swipe right on a task to start",
        TaskStatus::Paused => "Park tasks you'll come back to",
        TaskStatus::Done => "Swipe right to complete",
        _ => "",
    };

    rsx! {
        div { class: "task-list-mobile",
            if filtered.is_empty() {
                p { class: "column-empty", "{empty_hint}" }
            }
            for task in filtered {
                TaskCard {
                    key: "{task.id}",
                    task: task.clone(),
                    on_select,
                    on_delete,
                    on_focus,
                }
            }
        }
    }
}
