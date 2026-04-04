use dioxus::prelude::*;

use crate::components::focus_dock::task_field_pills;
use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};
use crate::time_utils;

#[component]
pub fn TaskCard(
    task: Task,
    field_defs: Option<Vec<FieldDefinition>>,
    task_fields: Option<Vec<TaskField>>,
    selected: Option<bool>,
    dragging_id: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_drag_start: Option<EventHandler<String>>,
    on_drag_end: Option<EventHandler<()>>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let is_selected = selected.unwrap_or(false);
    let is_dragging = dragging_id.as_ref() == Some(&task.id);

    let mut classes = vec!["task-card"];
    if is_done {
        classes.push("task-card-done");
    }
    if is_selected {
        classes.push("task-card-selected");
    }
    if is_dragging {
        classes.push("task-card-dragging");
    }
    let card_class = classes.join(" ");

    let subtitle = match task.status {
        TaskStatus::Inbox => {
            Some(format!("Added {}", time_utils::relative_time(&task.created_at)))
        }
        TaskStatus::Paused => {
            if !task.description.is_empty() {
                Some(task.description.clone())
            } else {
                Some(format!(
                    "Paused {}",
                    time_utils::relative_time(&task.updated_at)
                ))
            }
        }
        _ => None,
    };

    let pills = match (field_defs.as_ref(), task_fields.as_ref()) {
        (Some(fds), Some(tfs)) => task_field_pills(&task.id, fds, tfs),
        _ => Vec::new(),
    };

    rsx! {
        div {
            class: "{card_class}",
            draggable: if on_drag_start.is_some() { "true" } else { "false" },
            ondragstart: {
                let id = task.id.clone();
                let handler = on_drag_start.clone();
                move |e: Event<DragData>| {
                    let dt = e.data().data_transfer();
                    let _ = dt.set_data("text/plain", &id);
                    dt.set_effect_allowed("move");
                    if let Some(ref h) = handler {
                        h.call(id.clone());
                    }
                }
            },
            ondragend: {
                let handler = on_drag_end.clone();
                move |_| {
                    if let Some(ref h) = handler {
                        h.call(());
                    }
                }
            },
            onclick: {
                let id = task.id.clone();
                move |_| on_select.call(id.clone())
            },
            h4 { class: "task-card-title", "{task.title}" }
            if let Some(ref sub) = subtitle {
                p { class: "task-card-subtitle", "{sub}" }
            }
            if !pills.is_empty() {
                div { class: "task-card-fields",
                    for (key, value, color) in &pills {
                        span {
                            class: "task-card-pill",
                            style: "--tag-color: {color}",
                            title: "{key}",
                            "{value}"
                        }
                    }
                }
            }
            div { class: "task-actions",
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
