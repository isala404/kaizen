use dioxus::prelude::*;

use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};
use crate::time_utils;

#[component]
pub fn FocusDock(
    tasks: Vec<Task>,
    field_defs: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
    is_drag_active: Option<bool>,
    on_focus: EventHandler<String>,
    on_unfocus: EventHandler<String>,
    on_drop_focus: Option<EventHandler<String>>,
    on_reorder: Option<EventHandler<i32>>,
    on_drag_start: Option<EventHandler<String>>,
    on_drag_end: Option<EventHandler<()>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let mut drag_over = use_signal(|| false);
    let dragging = is_drag_active.unwrap_or(false);
    let count = tasks.len();

    let mut dock_classes = vec!["focus-dock"];
    if *drag_over.read() && dragging {
        dock_classes.push("focus-dock-drag-over");
    }
    let dock_class = dock_classes.join(" ");

    let label = if count > 0 {
        format!(
            "FOCUS DOCK - {} ACTIVE TASK{}",
            count,
            if count == 1 { "" } else { "S" }
        )
    } else {
        "FOCUS DOCK".to_string()
    };

    rsx! {
        div { class: "section-label", "{label}" }
        div {
            class: "{dock_class}",
            ondragover: move |e| {
                e.prevent_default();
                drag_over.set(true);
            },
            ondragleave: move |_| drag_over.set(false),
            ondrop: move |e: Event<DragData>| {
                e.prevent_default();
                drag_over.set(false);
                if let Some(ref h) = on_drop_focus {
                    h.call(String::new());
                }
            },
            if tasks.is_empty() {
                div { class: "dock-empty",
                    "Drag a task here to start working"
                }
            }
            for (i, task) in tasks.iter().enumerate() {
                {
                    let is_focused = task.status == TaskStatus::Focused;
                    let elapsed = if is_focused {
                        time_utils::focused_elapsed(task.time_spent_secs, &task.updated_at)
                    } else {
                        task.time_spent_secs
                    };
                    let time_str = if is_focused {
                        time_utils::format_timer(elapsed)
                    } else {
                        time_utils::format_duration(elapsed)
                    };
                    let card_class = if is_focused {
                        "focus-card focus-card-active"
                    } else {
                        "focus-card"
                    };
                    let pills = task_field_pills(&task.id, &field_defs, &task_fields);

                    let insert_pos = if i == 0 {
                        task.position - 10_000
                    } else {
                        (tasks[i - 1].position + task.position) / 2
                    };

                    rsx! {
                        if let Some(ref reorder_handler) = on_reorder {
                            DockDropZone {
                                position: insert_pos,
                                visible: dragging,
                                on_drop: reorder_handler.clone(),
                            }
                        }
                        div {
                            class: "{card_class}",
                            draggable: "true",
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
                                move |_| {
                                    if is_focused {
                                        on_unfocus.call(id.clone());
                                    } else {
                                        on_focus.call(id.clone());
                                    }
                                }
                            },
                            oncontextmenu: {
                                let id = task.id.clone();
                                let handler = on_select.clone();
                                move |e: Event<MouseData>| {
                                    e.prevent_default();
                                    if let Some(ref h) = handler {
                                        h.call(id.clone());
                                    }
                                }
                            },
                            if is_focused {
                                span { class: "focused-badge", "Focused" }
                            }
                            h3 { class: "focus-card-title", "{task.title}" }
                            if !task.description.is_empty() {
                                div { class: "focus-card-desc", "{task.description}" }
                            }
                            if !pills.is_empty() {
                                div { class: "focus-card-fields",
                                    for (key, value, color) in &pills {
                                        span {
                                            class: "focus-card-tag",
                                            style: "--tag-color: {color}",
                                            title: "{key}",
                                            "{value}"
                                        }
                                    }
                                }
                            }
                            div { class: "focus-card-meta",
                                if !time_str.is_empty() {
                                    span {
                                        class: if is_focused { "focus-card-time focus-card-time-active" } else { "focus-card-time" },
                                        "{time_str}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(ref reorder_handler) = on_reorder {
                DockDropZone {
                    position: tasks.last().map(|t| t.position + 10_000).unwrap_or(10_000),
                    visible: dragging,
                    on_drop: reorder_handler.clone(),
                }
            }
        }
    }
}

#[component]
fn DockDropZone(position: i32, visible: bool, on_drop: EventHandler<i32>) -> Element {
    let mut active = use_signal(|| false);

    let class = if !visible {
        "drop-zone-h drop-zone-h-hidden"
    } else if *active.read() {
        "drop-zone-h drop-zone-h-active"
    } else {
        "drop-zone-h"
    };

    rsx! {
        div {
            class,
            ondragover: move |e| {
                if visible {
                    e.prevent_default();
                    active.set(true);
                }
            },
            ondragleave: move |_| active.set(false),
            ondrop: move |e: Event<DragData>| {
                if visible {
                    e.prevent_default();
                    e.stop_propagation();
                    active.set(false);
                    on_drop.call(position);
                }
            },
        }
    }
}

pub fn task_field_pills(
    task_id: &str,
    field_defs: &[FieldDefinition],
    task_fields: &[TaskField],
) -> Vec<(String, String, String)> {
    let mut pills = Vec::new();
    for fd in field_defs {
        if let Some(tf) = task_fields
            .iter()
            .find(|tf| tf.task_id == task_id && tf.field_id == fd.id && !tf.value.is_empty())
        {
            let color = fd
                .color
                .clone()
                .unwrap_or_else(|| "#6366f1".to_string());
            pills.push((fd.key.clone(), tf.value.clone(), color));
        }
    }
    pills
}
