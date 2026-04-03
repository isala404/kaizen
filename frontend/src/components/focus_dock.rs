use dioxus::prelude::*;

use crate::forge::Task;
use crate::time_utils;

#[component]
pub fn FocusDock(
    task: Option<Task>,
    elapsed_secs: Option<i64>,
    is_drag_active: Option<bool>,
    on_click: EventHandler<String>,
    on_unfocus: EventHandler<()>,
    on_drop_focus: Option<EventHandler<String>>,
) -> Element {
    let mut drag_over = use_signal(|| false);
    let dragging = is_drag_active.unwrap_or(false);

    let mut dock_classes = vec!["focus-dock"];
    if *drag_over.read() && dragging {
        dock_classes.push("focus-dock-drag-over");
    }
    let dock_class = dock_classes.join(" ");

    rsx! {
        div { class: "section-label", "FOCUS" }
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
                // The dashboard handles the actual focus call via on_drop_focus
                if let Some(ref h) = on_drop_focus {
                    h.call(String::new()); // signal that a drop occurred
                }
            },
            match task {
                Some(ref t) => {
                    let timer_str = time_utils::format_timer(elapsed_secs.unwrap_or(t.time_spent_secs));
                    rsx! {
                        div {
                            class: "focus-card",
                            onclick: {
                                let id = t.id.clone();
                                move |_| on_click.call(id.clone())
                            },
                            span { class: "focused-badge", "Focused" }
                            h3 { class: "focus-card-title", "{t.title}" }
                            if !t.description.is_empty() {
                                p { class: "focus-card-desc-preview", "{t.description}" }
                            }
                            div { class: "focus-card-meta",
                                span { class: "time-today timer-active", "{timer_str}" }
                            }
                        }
                    }
                }
                None => rsx! {
                    div { class: "dock-empty",
                        "Drag a task here to start the timer"
                    }
                }
            }
        }
    }
}
