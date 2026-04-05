use dioxus::prelude::*;

use crate::forge::{Task, TaskStatus};
use crate::time_utils;

#[component]
pub fn TaskCard(
    task: Task,
    selected: Option<bool>,
    dragging_id: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_drag_start: Option<EventHandler<String>>,
    on_drag_end: Option<EventHandler<()>>,
    on_touch_drag_start: Option<EventHandler<(String, f64, f64)>>,
    on_touch_drag_move: Option<EventHandler<(f64, f64)>>,
    on_touch_drag_end: Option<EventHandler<(f64, f64)>>,
) -> Element {
    let is_done = task.status == TaskStatus::Done;
    let is_selected = selected.unwrap_or(false);
    let is_dragging = dragging_id.as_ref() == Some(&task.id);
    let mut long_press_task = use_signal(|| Option::<dioxus::core::Task>::None);
    let mut touch_active = use_signal(|| false);
    let mut touch_start_pos = use_signal(|| (0.0f64, 0.0f64));
    let mut last_touch_pos = use_signal(|| (0.0f64, 0.0f64));

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
            format!("Added {}", time_utils::relative_time(&task.updated_at))
        }
        TaskStatus::UpNext => {
            format!("Queued {}", time_utils::relative_time(&task.updated_at))
        }
        TaskStatus::Paused => {
            format!("Paused {}", time_utils::relative_time(&task.updated_at))
        }
        TaskStatus::Done => {
            let when = time_utils::relative_time(&task.updated_at);
            let duration = time_utils::format_duration(task.time_spent_secs);
            if duration.is_empty() {
                format!("Done {when}")
            } else {
                format!("Done {when} in {duration}")
            }
        }
        _ => {
            time_utils::relative_time(&task.updated_at)
        }
    };

    rsx! {
        div {
            class: "{card_class}",
            draggable: if on_drag_start.is_some() { "true" } else { "false" },
            ondragstart: {
                let id = task.id.clone();
                let handler = on_drag_start.clone();
                move |e: Event<DragData>| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let dt = e.data().data_transfer();
                        let _ = dt.set_data("text/plain", &id);
                        dt.set_effect_allowed("move");
                    }
                    let _ = &e;
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
            ontouchstart: {
                let id = task.id.clone();
                let handler = on_touch_drag_start.clone();
                move |e| {
                    if let Some(ref h) = handler {
                        if let Some(touch) = e.touches().first() {
                            let coords = touch.client_coordinates();
                            touch_start_pos.set((coords.x, coords.y));
                            last_touch_pos.set((coords.x, coords.y));
                            let id = id.clone();
                            let h = h.clone();
                            let task = spawn(async move {
                                dioxus_sdk::time::sleep(std::time::Duration::from_millis(300)).await;
                                touch_active.set(true);
                                h.call((id, coords.x, coords.y));
                            });
                            long_press_task.set(Some(task));
                        }
                    }
                }
            },
            ontouchmove: {
                let move_handler = on_touch_drag_move.clone();
                move |e| {
                    if let Some(touch) = e.touches().first() {
                        let coords = touch.client_coordinates();
                        last_touch_pos.set((coords.x, coords.y));
                        if long_press_task.read().is_some() {
                            let (sx, sy) = *touch_start_pos.read();
                            let dx = coords.x - sx;
                            let dy = coords.y - sy;
                            if (dx * dx + dy * dy) > 100.0 {
                                if let Some(task) = long_press_task.take() {
                                    task.cancel();
                                }
                            }
                            return;
                        }
                        if *touch_active.read() {
                            e.prevent_default();
                            if let Some(ref h) = move_handler {
                                h.call((coords.x, coords.y));
                            }
                        }
                    }
                }
            },
            ontouchend: {
                let end_handler = on_touch_drag_end.clone();
                move |_| {
                    if let Some(task) = long_press_task.take() {
                        task.cancel();
                        return;
                    }
                    if *touch_active.read() {
                        touch_active.set(false);
                        if let Some(ref h) = end_handler {
                            let pos = *last_touch_pos.read();
                            h.call(pos);
                        }
                    }
                }
            },
            onclick: {
                let id = task.id.clone();
                move |_| on_select.call(id.clone())
            },
            h4 { class: "task-card-title", "{task.title}" }
            p { class: "task-card-subtitle", "{subtitle}" }
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
