use dioxus::prelude::*;

use crate::components::{DropTarget, StatusChange, TaskCard};
use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};

#[component]
pub fn BoardColumn(
    label: String,
    status: TaskStatus,
    is_focused: bool,
    focused_row: Option<usize>,
    tasks: Vec<Task>,
    field_defs: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
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
    let count = tasks.len();
    let mut show_add = use_signal(|| false);
    let mut add_title = use_signal(String::new);
    let mut drag_over = use_signal(|| false);

    let is_dragging = dragging_id.is_some();

    let mut col_classes = vec!["board-column"];
    if is_focused {
        col_classes.push("board-column-focused");
    }
    if *drag_over.read() && is_dragging {
        col_classes.push("board-column-drag-over");
    }
    let col_class = col_classes.join(" ");

    let empty_hint = match status {
        TaskStatus::Inbox => "Press N to add a task",
        TaskStatus::UpNext => "Drag tasks here to plan your day",
        TaskStatus::InProgress => "Drag here when you start working",
        TaskStatus::Paused => "Park tasks you'll come back to",
        TaskStatus::Done => "Drag here when finished",
        _ => "",
    };

    // Compute drop position: end of column
    let end_position = tasks.last().map(|t| t.position + 10_000).unwrap_or(10_000);

    rsx! {
        div {
            class: "{col_class}",
            ondragover: move |e| {
                e.prevent_default();
                drag_over.set(true);
            },
            ondragleave: move |_| drag_over.set(false),
            ondrop: {
                let status = status.clone();
                move |e: Event<DragData>| {
                    e.prevent_default();
                    drag_over.set(false);
                    if let Some(ref did) = dragging_id {
                        on_drop.call(DropTarget {
                            task_id: did.clone(),
                            status: status.clone(),
                            position: end_position,
                        });
                    }
                }
            },
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
                        field_defs: field_defs.clone(),
                        task_fields: task_fields.clone(),
                        selected: focused_row == Some(i),
                        dragging_id: dragging_id.clone(),
                        on_select,
                        on_delete,
                        on_focus,
                        on_drag_start: on_drag_start,
                        on_drag_end: on_drag_end,
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
