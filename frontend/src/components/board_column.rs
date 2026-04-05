use dioxus::prelude::*;
use dioxus_sdk::storage::use_persistent;

use crate::components::{DropTarget, TaskCard};
use crate::forge::{Task, TaskStatus};
use crate::time_utils;

#[component]
pub fn BoardColumn(
    label: String,
    status: TaskStatus,
    is_focused: bool,
    focused_row: Option<usize>,
    tasks: Vec<Task>,
    dragging_id: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_create: EventHandler<(String, TaskStatus)>,
    on_drag_start: EventHandler<String>,
    on_drag_end: EventHandler<()>,
    on_drop: EventHandler<DropTarget>,
) -> Element {
    let mut collapsed = use_persistent(format!("col_collapsed_{label}"), || false);
    let mut show_add = use_signal(|| false);
    let mut add_title = use_signal(String::new);
    let mut show_previous = use_signal(|| false);

    let is_dragging = dragging_id.is_some();
    let is_done = status == TaskStatus::Done;

    let visible_tasks: Vec<&Task> = if is_done && !*show_previous.read() {
        let today_start = time_utils::today_start_epoch();
        tasks
            .iter()
            .filter(|t| time_utils::parse_iso_to_epoch_secs(&t.updated_at) >= today_start)
            .collect()
    } else {
        tasks.iter().collect()
    };

    let count = tasks.len();
    let hidden_count = count - visible_tasks.len();
    let end_position = tasks.last().map(|t| t.position + 10_000).unwrap_or(10_000);
    let mut drag_over = use_signal(|| false);

    let mut col_classes = vec!["board-column"];
    if is_focused {
        col_classes.push("board-column-focused");
    }
    if *drag_over.read() && is_dragging {
        col_classes.push("board-column-drag-over");
    }
    if *collapsed.read() {
        col_classes.push("board-column-collapsed");
    }
    let col_class = col_classes.join(" ");

    let header_label = if is_done {
        "Done today".to_string()
    } else {
        label.clone()
    };

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
                    on_drop.call(DropTarget {
                        status: status.clone(),
                        position: end_position,
                    });
                }
            },
            div {
                class: "column-header",
                onclick: {
                    move |_| {
                        let new_val = !*collapsed.read();
                        collapsed.set(new_val);
                    }
                },
                h3 { class: "column-title", "{header_label}" }
                span { class: "column-count", "{count}" }
            }

            if !*collapsed.read() {
                div { class: "column-cards",
                    for (i, task) in visible_tasks.iter().enumerate() {
                        {
                            let insert_pos = if i == 0 {
                                task.position - 10_000
                            } else {
                                (visible_tasks[i - 1].position + task.position) / 2
                            };
                            rsx! {
                                CardDropZone {
                                    key: "dz-{i}",
                                    status: status.clone(),
                                    position: insert_pos,
                                    visible: is_dragging,
                                    on_drop,
                                }
                                TaskCard {
                                    key: "{task.id}",
                                    task: (*task).clone(),
                                    selected: focused_row == Some(i),
                                    dragging_id: dragging_id.clone(),
                                    on_select,
                                    on_delete,
                                    on_drag_start: on_drag_start,
                                    on_drag_end: on_drag_end,
                                }
                            }
                        }
                    }

                    // Trailing drop zone after the last card
                    CardDropZone {
                        key: "dz-end",
                        status: status.clone(),
                        position: end_position,
                        visible: is_dragging,
                        on_drop,
                    }

                    if is_done && hidden_count > 0 {
                        button {
                            class: "column-show-previous",
                            onclick: move |_| {
                                let current = *show_previous.read();
                                show_previous.set(!current);
                            },
                            if *show_previous.read() {
                                "Hide previous"
                            } else {
                                "Show {hidden_count} previous"
                            }
                        }
                    }

                    if *show_add.read() {
                        input {
                            class: "add-task-input",
                            placeholder: "Task title...",
                            value: "{add_title}",
                            autofocus: true,
                            oninput: move |e| add_title.set(e.value()),
                            onkeydown: {
                                let status = status.clone();
                                move |e: Event<KeyboardData>| {
                                    match e.key() {
                                        Key::Enter => {
                                            let title = add_title.read().trim().to_string();
                                            if !title.is_empty() {
                                                on_create.call((title, status.clone()));
                                            }
                                            add_title.set(String::new());
                                            show_add.set(false);
                                        }
                                        Key::Escape => {
                                            add_title.set(String::new());
                                            show_add.set(false);
                                        }
                                        _ => {}
                                    }
                                }
                            },
                            onblur: move |_| {
                                add_title.set(String::new());
                                show_add.set(false);
                            },
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
}

#[component]
fn CardDropZone(
    status: TaskStatus,
    position: i32,
    visible: bool,
    on_drop: EventHandler<DropTarget>,
) -> Element {
    let mut active = use_signal(|| false);

    let class = if !visible {
        "drop-zone drop-zone-hidden"
    } else if *active.read() {
        "drop-zone drop-zone-active"
    } else {
        "drop-zone"
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
            ondrop: {
                let status = status.clone();
                move |e: Event<DragData>| {
                    if visible {
                        e.prevent_default();
                        e.stop_propagation();
                        active.set(false);
                        on_drop.call(DropTarget { status: status.clone(), position });
                    }
                }
            },
        }
    }
}
