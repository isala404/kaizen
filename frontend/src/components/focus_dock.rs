use dioxus::prelude::*;

use crate::components::TouchHoverZone;
use crate::forge::{Task, TaskStatus};
use crate::time_utils;

#[component]
pub fn FocusDock(
    tasks: Vec<Task>,
    is_drag_active: Option<bool>,
    on_focus: EventHandler<String>,
    on_unfocus: EventHandler<String>,
    on_drop_focus: Option<EventHandler<String>>,
    on_reorder: Option<EventHandler<i32>>,
    on_drag_start: Option<EventHandler<String>>,
    on_drag_end: Option<EventHandler<()>>,
    on_select: Option<EventHandler<String>>,
    on_touch_drag_start: Option<EventHandler<(String, f64, f64)>>,
    on_touch_drag_move: Option<EventHandler<(f64, f64)>>,
    on_touch_drag_end: Option<EventHandler<(f64, f64)>>,
) -> Element {
    let mut drag_over = use_signal(|| false);
    let mut last_tap_time = use_signal(|| 0.0f64);
    let mut long_press_task = use_signal(|| Option::<dioxus::core::Task>::None);
    let mut touch_active = use_signal(|| false);
    let mut touch_start_pos = use_signal(|| (0.0f64, 0.0f64));
    let mut last_touch_pos = use_signal(|| (0.0f64, 0.0f64));
    let dragging = is_drag_active.unwrap_or(false);
    let count = tasks.len();

    let touch_hover = try_consume_context::<TouchHoverZone>();
    let is_touch_hover_dock = touch_hover.is_some_and(|h| h.matches_status("focus_dock"));

    let mut dock_classes = vec!["focus-dock"];
    if (*drag_over.read() || is_touch_hover_dock) && dragging {
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
            "data-drop-status": "focus_dock",
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
                                let select_handler = on_select.clone();
                                move |_| {
                                    let now = chrono::Utc::now().timestamp_millis() as f64;
                                    let last = *last_tap_time.read();
                                    last_tap_time.set(now);

                                    if now - last < 350.0 {
                                        if let Some(ref h) = select_handler {
                                            h.call(id.clone());
                                        }
                                    } else if is_focused {
                                        on_unfocus.call(id.clone());
                                    } else {
                                        on_focus.call(id.clone());
                                    }
                                }
                            },
                            if is_focused {
                                span { class: "focused-badge", "Focused" }
                            }
                            h3 { class: "focus-card-title", "{task.title}" }
                            div { class: "focus-card-meta",
                                if !time_str.is_empty() {
                                    span {
                                        class: if is_focused { "focus-card-time focus-card-time-active" } else { "focus-card-time" },
                                        "Worked {time_str}"
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

    let touch_hover = try_consume_context::<TouchHoverZone>();
    let is_touch_active = touch_hover.is_some_and(|h| h.matches("dock_reorder", position));

    let class = if !visible {
        "drop-zone-h drop-zone-h-hidden"
    } else if *active.read() || is_touch_active {
        "drop-zone-h drop-zone-h-active"
    } else {
        "drop-zone-h"
    };

    rsx! {
        div {
            class,
            "data-drop-status": "dock_reorder",
            "data-drop-position": "{position}",
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
