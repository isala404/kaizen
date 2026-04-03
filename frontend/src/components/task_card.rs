use dioxus::prelude::*;

use crate::components::FieldPills;
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
    on_focus: EventHandler<String>,
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

    let time_str = time_utils::format_duration(task.time_spent_secs);
    let rel_time = time_utils::relative_time(&task.created_at);
    let has_fields = field_defs.is_some() && task_fields.is_some();

    rsx! {
        div {
            class: "{card_class}",
            draggable: if on_drag_start.is_some() { "true" } else { "false" },
            ondragstart: {
                let id = task.id.clone();
                let handler = on_drag_start.clone();
                move |_| {
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
                    if has_fields {
                        FieldPills {
                            task_id: task.id.clone(),
                            fields: field_defs.clone().unwrap_or_default(),
                            task_fields: task_fields.clone().unwrap_or_default(),
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
