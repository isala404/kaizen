use dioxus::prelude::*;

use crate::forge::Task;
use crate::pages::dashboard::format_time;

#[component]
pub fn FocusDock(
    task: Option<Task>,
    on_click: EventHandler<String>,
    on_unfocus: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "section-label", "FOCUS" }
        div { class: "focus-dock",
            match task {
                Some(ref t) => rsx! {
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
                            span { class: "time-today timer-active",
                                "{format_time(t.time_spent_secs)}"
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "dock-empty",
                        "Drag a task here to start the timer"
                    }
                }
            }
        }
    }
}
