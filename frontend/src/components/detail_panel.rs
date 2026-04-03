use dioxus::prelude::*;

use crate::components::StatusChange;
use crate::forge::{Task, TaskStatus};
use crate::pages::dashboard::format_time;

#[component]
pub fn DetailPanel(
    task: Task,
    on_close: EventHandler<()>,
    on_status_change: EventHandler<StatusChange>,
    on_focus: EventHandler<String>,
) -> Element {
    let status_label = match task.status {
        TaskStatus::Inbox => "Inbox",
        TaskStatus::UpNext => "Up next",
        TaskStatus::InProgress => "In progress",
        TaskStatus::Focused => "Focused",
        TaskStatus::Paused => "Paused",
        TaskStatus::Done => "Done",
        TaskStatus::Archived => "Archived",
    };

    let statuses = vec![
        ("Inbox", TaskStatus::Inbox),
        ("Up next", TaskStatus::UpNext),
        ("In progress", TaskStatus::InProgress),
        ("Focused", TaskStatus::Focused),
        ("Paused", TaskStatus::Paused),
        ("Done", TaskStatus::Done),
        ("Archived", TaskStatus::Archived),
    ];

    rsx! {
        div {
            class: "detail-overlay",
            onclick: move |_| on_close.call(()),
        }
        div { class: "detail-panel",
            div { class: "detail-header",
                div { class: "detail-status",
                    span { class: "status-pill", "{status_label}" }
                }
                button {
                    class: "detail-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            h2 { class: "detail-title", "{task.title}" }

            div { class: "detail-section",
                label { class: "detail-label", "Status" }
                div { class: "detail-status-options",
                    for (label, status) in &statuses {
                        button {
                            class: if task.status == *status { "status-option active" } else { "status-option" },
                            onclick: {
                                let id = task.id.clone();
                                let s = status.clone();
                                move |_| {
                                    if s == TaskStatus::Focused {
                                        on_focus.call(id.clone());
                                    } else {
                                        on_status_change.call(StatusChange {
                                            id: id.clone(),
                                            status: s.clone(),
                                        });
                                    }
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }

            if task.time_spent_secs > 0 || task.status == TaskStatus::Focused {
                div { class: "detail-section",
                    label { class: "detail-label", "Time spent" }
                    span { class: "detail-time", "{format_time(task.time_spent_secs)}" }
                }
            }

            div { class: "detail-section",
                label { class: "detail-label", "Description" }
                if task.description.is_empty() {
                    p { class: "detail-placeholder", "Add a description..." }
                } else {
                    p { class: "detail-description", "{task.description}" }
                }
            }
        }
    }
}
