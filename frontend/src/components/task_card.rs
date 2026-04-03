use dioxus::prelude::*;

use crate::forge::{Task, TaskStatus};
use crate::pages::dashboard::format_time;

#[component]
pub fn TaskCard(
    task: Task,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let card_class = if is_done {
        "task-card task-card-done"
    } else {
        "task-card"
    };

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
                    span { class: "task-time", "{relative_time(&task.created_at)}" }
                    {
                        let time_str = format_time(task.time_spent_secs);
                        if !time_str.is_empty() {
                            rsx! { span { class: "task-total-time", "{time_str}" } }
                        } else {
                            rsx! {}
                        }
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

fn relative_time(_iso: &str) -> String {
    // Simple relative time without pulling in a date library on WASM
    // The server sends ISO 8601 timestamps
    "just now".to_string()
}
