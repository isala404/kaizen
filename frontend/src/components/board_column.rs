use dioxus::prelude::*;

use crate::components::{DropTarget, StatusChange, TaskCard};
use crate::forge::{FieldDefinition, Task, TaskField, TaskStatus};
use crate::time_utils;

#[component]
pub fn BoardColumn(
    label: String,
    status: TaskStatus,
    total_task_count: Option<usize>,
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
    let mut collapsed = use_signal(|| {
        // Read from localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(val)) = storage.get_item(&format!("col_collapsed_{label}")) {
                    return val == "true";
                }
            }
        }
        false
    });
    let mut show_add = use_signal(|| false);
    let mut add_title = use_signal(String::new);
    let mut drag_over = use_signal(|| false);
    let mut show_previous = use_signal(|| false);

    let is_dragging = dragging_id.is_some();
    let is_done = status == TaskStatus::Done;
    let total = total_task_count.unwrap_or(0);

    // Filter done tasks: only show today's unless "show previous" toggled
    let visible_tasks: Vec<&Task> = if is_done && !*show_previous.read() {
        let today_start = today_start_epoch();
        tasks
            .iter()
            .filter(|t| time_utils::parse_iso_to_epoch_secs(&t.updated_at) >= today_start)
            .collect()
    } else {
        tasks.iter().collect()
    };

    let count = tasks.len();
    let visible_count = visible_tasks.len();
    let hidden_count = count - visible_count;

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

    let empty_hint = if total >= 5 {
        ""
    } else {
        match status {
            TaskStatus::Inbox => "Press N to add a task",
            TaskStatus::UpNext => "Drag tasks here to plan your day",
            TaskStatus::InProgress => "Drag here when you start working",
            TaskStatus::Paused => "Park tasks you'll come back to",
            TaskStatus::Done => "Drag here when finished",
            _ => "",
        }
    };

    let end_position = tasks.last().map(|t| t.position + 10_000).unwrap_or(10_000);

    let header_label = if is_done {
        format!("{label} — Today")
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
                    let label = label.clone();
                    move |_| {
                        let new_val = !*collapsed.read();
                        collapsed.set(new_val);
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.set_item(
                                    &format!("col_collapsed_{label}"),
                                    if new_val { "true" } else { "false" },
                                );
                            }
                        }
                    }
                },
                h3 { class: "column-title", "{header_label}" }
                span { class: "column-count", "{count}" }
            }

            if !*collapsed.read() {
                div { class: "column-cards",
                    if visible_tasks.is_empty() && !empty_hint.is_empty() {
                        p { class: "column-empty", "{empty_hint}" }
                    }
                    for (i, task) in visible_tasks.iter().enumerate() {
                        TaskCard {
                            key: "{task.id}",
                            task: (*task).clone(),
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
}

fn today_start_epoch() -> f64 {
    // Get start of today in UTC using js_sys
    let now = js_sys::Date::new_0();
    let today = js_sys::Date::new_with_year_month_day(
        now.get_full_year(),
        now.get_month() as i32,
        now.get_date() as i32,
    );
    today.get_time() / 1000.0
}
