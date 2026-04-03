use dioxus::prelude::*;

use crate::forge::{Task, TaskStatus};
use crate::time_utils;

#[component]
pub fn TaskCard(
    task: Task,
    selected: Option<bool>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let is_selected = selected.unwrap_or(false);
    let card_class = match (is_done, is_selected) {
        (true, true) => "task-card task-card-done task-card-selected",
        (true, false) => "task-card task-card-done",
        (false, true) => "task-card task-card-selected",
        (false, false) => "task-card",
    };

    let time_str = time_utils::format_duration(task.time_spent_secs);
    let rel_time = time_utils::relative_time(&task.created_at);

    rsx! {
        div { class: "{card_class}",
            div { class: "task-card-content",
                h4 {
                    class: "task-card-title",
                    onclick: {
                        let id = task.id.clone();
                        move |e: Event<MouseData>| {
                            e.stop_propagation();
                            on_select.call(id.clone());
                        }
                    },
                    "{task.title}"
                }
                div { class: "task-card-footer",
                    span { class: "task-time", "{rel_time}" }
                    if !time_str.is_empty() {
                        span { class: "task-total-time", "{time_str}" }
                    }
                }
            }
            div { class: "task-actions",
                if task.status != TaskStatus::Focused {
                    button {
                        class: "task-action-btn",
                        title: "Focus",
                        onclick: {
                            let id = task.id.clone();
                            move |e: Event<MouseData>| {
                                e.stop_propagation();
                                on_focus.call(id.clone());
                            }
                        },
                        "▶"
                    }
                }
                button {
                    class: "task-action-btn task-action-delete",
                    title: "Delete",
                    onclick: {
                        let id = task.id.clone();
                        move |e: Event<MouseData>| {
                            e.stop_propagation();
                            on_delete.call(id.clone());
                        }
                    },
                    "×"
                }
            }
        }
    }
}
