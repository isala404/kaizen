use dioxus::prelude::*;

use crate::forge::TaskStatus;

fn next_status(s: &TaskStatus) -> Option<TaskStatus> {
    match s {
        TaskStatus::Inbox => Some(TaskStatus::UpNext),
        TaskStatus::UpNext => Some(TaskStatus::InProgress),
        TaskStatus::InProgress => Some(TaskStatus::Done),
        TaskStatus::Paused => Some(TaskStatus::InProgress),
        _ => None,
    }
}

fn prev_status(s: &TaskStatus) -> Option<TaskStatus> {
    match s {
        TaskStatus::UpNext => Some(TaskStatus::Inbox),
        TaskStatus::InProgress => Some(TaskStatus::UpNext),
        TaskStatus::Done => Some(TaskStatus::InProgress),
        TaskStatus::Paused => Some(TaskStatus::Inbox),
        _ => None,
    }
}

#[component]
pub fn SwipeableCard(
    task_id: String,
    status: TaskStatus,
    on_swipe: EventHandler<(String, TaskStatus)>,
    children: Element,
) -> Element {
    let mut start_x = use_signal(|| 0.0f64);
    let mut offset_x = use_signal(|| 0.0f64);
    let mut swiping = use_signal(|| false);

    let offset = *offset_x.read();
    let transform = if offset.abs() > 2.0 {
        format!("translateX({offset}px)")
    } else {
        String::new()
    };

    let next = next_status(&status);
    let prev = prev_status(&status);
    let show_forward = offset > 20.0 && next.is_some();
    let show_back = offset < -20.0 && prev.is_some();

    rsx! {
        div {
            class: "swipeable-wrapper",
            // Reveal background
            if show_forward {
                div { class: "swipe-reveal swipe-reveal-forward",
                    "{next.as_ref().map(status_label).unwrap_or_default()}"
                }
            }
            if show_back {
                div { class: "swipe-reveal swipe-reveal-back",
                    "{prev.as_ref().map(status_label).unwrap_or_default()}"
                }
            }
            div {
                class: if *swiping.read() { "swipeable-inner swiping" } else { "swipeable-inner" },
                style: if !transform.is_empty() { "transform: {transform}" } else { "" },
                ontouchstart: move |e| {
                    if let Some(touch) = e.touches().first() {
                        start_x.set(touch.client_coordinates().x);
                        swiping.set(true);
                    }
                },
                ontouchmove: move |e| {
                    if *swiping.read() {
                        if let Some(touch) = e.touches().first() {
                            let dx = touch.client_coordinates().x - *start_x.read();
                            offset_x.set(dx);
                        }
                    }
                },
                ontouchend: {
                    let task_id = task_id.clone();
                    let status = status.clone();
                    move |_| {
                        swiping.set(false);
                        let dx = *offset_x.read();
                        // 80px threshold for commit
                        if dx > 80.0 {
                            if let Some(ns) = next_status(&status) {
                                on_swipe.call((task_id.clone(), ns));
                            }
                        } else if dx < -80.0 {
                            if let Some(ps) = prev_status(&status) {
                                on_swipe.call((task_id.clone(), ps));
                            }
                        }
                        offset_x.set(0.0);
                    }
                },
                {children}
            }
        }
    }
}

fn status_label(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Inbox => "Inbox",
        TaskStatus::UpNext => "Up next",
        TaskStatus::InProgress => "In progress",
        TaskStatus::Focused => "Focused",
        TaskStatus::Paused => "Paused",
        TaskStatus::Done => "Done",
        TaskStatus::Archived => "Archived",
    }
}
