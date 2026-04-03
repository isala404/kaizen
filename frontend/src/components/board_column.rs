use dioxus::prelude::*;

use crate::components::{StatusChange, TaskCard};
use crate::forge::{Task, TaskStatus};

#[component]
pub fn BoardColumn(
    label: String,
    status: TaskStatus,
    is_focused: bool,
    focused_row: Option<usize>,
    tasks: Vec<Task>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_focus: EventHandler<String>,
    on_create: EventHandler<String>,
    on_status_change: EventHandler<StatusChange>,
) -> Element {
    let count = tasks.len();
    let mut show_add = use_signal(|| false);
    let mut add_title = use_signal(String::new);

    let col_class = if is_focused {
        "board-column board-column-focused"
    } else {
        "board-column"
    };

    let empty_hint = match status {
        TaskStatus::Inbox => "Press N to add a task",
        TaskStatus::UpNext => "Drag tasks here to plan your day",
        TaskStatus::InProgress => "Drag here when you start working",
        TaskStatus::Paused => "Park tasks you'll come back to",
        TaskStatus::Done => "Drag here when finished",
        _ => "",
    };

    rsx! {
        div { class: "{col_class}",
            div { class: "column-header",
                h3 { class: "column-title", "{label}" }
                span { class: "column-count", "{count}" }
            }
            div { class: "column-cards",
                if tasks.is_empty() {
                    p { class: "column-empty", "{empty_hint}" }
                }
                for (i, task) in tasks.iter().enumerate() {
                    TaskCard {
                        key: "{task.id}",
                        task: task.clone(),
                        selected: focused_row == Some(i),
                        on_select,
                        on_delete,
                        on_focus,
                    }
                }

                if *show_add.read() {
                    form {
                        class: "column-add-form",
                        onsubmit: move |e| {
                            e.prevent_default();
                            let title = add_title.read().trim().to_string();
                            if !title.is_empty() {
                                on_create.call(title);
                                add_title.set(String::new());
                                show_add.set(false);
                            }
                        },
                        input {
                            class: "add-task-input",
                            placeholder: "Task title...",
                            value: "{add_title}",
                            oninput: move |e| add_title.set(e.value()),
                        }
                        button {
                            class: "add-task-submit",
                            r#type: "submit",
                            "Add"
                        }
                    }
                } else {
                    button {
                        class: "column-add-btn",
                        onclick: move |_| show_add.set(true),
                        "+"
                    }
                }
            }
        }
    }
}
