use dioxus::prelude::*;

use crate::forge::{Task, TaskStatus};

#[component]
pub fn StatusTabs(
    tasks: Vec<Task>,
    active: TaskStatus,
    on_change: EventHandler<TaskStatus>,
) -> Element {
    let tabs = vec![
        ("Inbox", TaskStatus::Inbox),
        ("Up next", TaskStatus::UpNext),
        ("In progress", TaskStatus::InProgress),
        ("Paused", TaskStatus::Paused),
        ("Done", TaskStatus::Done),
    ];

    rsx! {
        div { class: "status-tabs",
            for (label, status) in &tabs {
                {
                    let count = tasks.iter().filter(|t| t.status == *status).count();
                    let cls = if active == *status {
                        "status-tab status-tab-active"
                    } else {
                        "status-tab"
                    };
                    rsx! {
                        button {
                            class: "{cls}",
                            onclick: {
                                let s = status.clone();
                                move |_| on_change.call(s.clone())
                            },
                            "{label} ({count})"
                        }
                    }
                }
            }
        }
    }
}
