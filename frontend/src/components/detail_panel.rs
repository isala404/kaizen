use dioxus::prelude::*;

use crate::components::StatusChange;
use crate::forge::{
    FieldDefinition, FieldValueType, SetTaskFieldInput, Task, TaskField, TaskStatus,
    UpdateTaskInput, use_set_task_field,
};
use crate::time_utils;

#[component]
pub fn DetailPanel(
    task: Task,
    field_defs: Option<Vec<FieldDefinition>>,
    task_fields: Option<Vec<TaskField>>,
    on_close: EventHandler<()>,
    on_status_change: EventHandler<StatusChange>,
    on_focus: EventHandler<String>,
    on_update: EventHandler<UpdateTaskInput>,
) -> Element {
    let set_field = use_set_task_field();

    let mut editing_desc = use_signal(|| false);
    let mut desc_draft = use_signal(|| task.description.clone());
    let mut title_draft = use_signal(|| task.title.clone());

    let status_label = match task.status {
        TaskStatus::Inbox => "Inbox",
        TaskStatus::UpNext => "Up next",
        TaskStatus::InProgress => "In progress",
        TaskStatus::Focused => "Focused",
        TaskStatus::Paused => "Paused",
        TaskStatus::Done => "Done",
        TaskStatus::Archived => "Archived",
    };

    let statuses = [
        ("Inbox", TaskStatus::Inbox),
        ("Up next", TaskStatus::UpNext),
        ("In progress", TaskStatus::InProgress),
        ("Focused", TaskStatus::Focused),
        ("Paused", TaskStatus::Paused),
        ("Done", TaskStatus::Done),
        ("Archived", TaskStatus::Archived),
    ];

    let time_str = time_utils::format_duration(task.time_spent_secs);
    let due_date_val = task
        .due_at
        .as_ref()
        .and_then(|d| d.get(..10).map(|s| s.to_string()))
        .unwrap_or_default();

    let fields = field_defs.unwrap_or_default();
    let tfs = task_fields.unwrap_or_default();

    rsx! {
        div {
            class: "detail-overlay",
            onclick: move |_| on_close.call(()),
        }
        div {
            class: "detail-panel",
            onclick: move |e| e.stop_propagation(),

            div { class: "detail-header",
                span { class: "status-pill", "{status_label}" }
                button {
                    class: "detail-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            input {
                class: "detail-title-input",
                value: "{title_draft}",
                oninput: move |e| title_draft.set(e.value()),
                onblur: {
                    let id = task.id.clone();
                    move |_| {
                        let new_title = title_draft.read().trim().to_string();
                        if !new_title.is_empty() && new_title != task.title {
                            on_update.call(UpdateTaskInput::new(id.clone()).title(new_title));
                        }
                    }
                },
            }

            // Status
            div { class: "detail-section",
                label { class: "detail-label", "Status" }
                div { class: "detail-status-options",
                    for (label, status) in statuses {
                        button {
                            class: if task.status == status { "status-option active" } else { "status-option" },
                            onclick: {
                                let id = task.id.clone();
                                let s = status.clone();
                                move |_| {
                                    if s == TaskStatus::Focused {
                                        on_focus.call(id.clone());
                                    } else {
                                        on_status_change.call(StatusChange {
                                            id: id.clone(),
                                            status: s.clone(),
                                        });
                                    }
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }

            // Fields
            if !fields.is_empty() {
                div { class: "detail-section",
                    label { class: "detail-label", "Fields" }
                    for fd in &fields {
                        {
                            let current_val = tfs.iter()
                                .find(|tf| tf.task_id == task.id && tf.field_id == fd.id)
                                .map(|tf| tf.value.clone())
                                .unwrap_or_default();
                            let fd_id = fd.id.clone();
                            let task_id = task.id.clone();
                            rsx! {
                                div { class: "detail-field-row",
                                    span { class: "detail-field-key", "{fd.key}" }
                                    match fd.value_type {
                                        FieldValueType::Enum => {
                                            let options = fd.options.clone().unwrap_or_default();
                                            rsx! {
                                                select {
                                                    class: "field-select",
                                                    value: "{current_val}",
                                                    onchange: {
                                                        let set_field = set_field.clone();
                                                        let fd_id = fd_id.clone();
                                                        let task_id = task_id.clone();
                                                        move |e: Event<FormData>| {
                                                            let set_field = set_field.clone();
                                                            let input = SetTaskFieldInput {
                                                                task_id: task_id.clone(),
                                                                field_id: fd_id.clone(),
                                                                value: e.value(),
                                                            };
                                                            spawn(async move {
                                                                let _ = set_field.call(input).await;
                                                            });
                                                        }
                                                    },
                                                    option { value: "", "—" }
                                                    for opt in &options {
                                                        option { value: "{opt}", "{opt}" }
                                                    }
                                                }
                                            }
                                        }
                                        FieldValueType::Bool => rsx! {
                                            input {
                                                class: "field-checkbox",
                                                r#type: "checkbox",
                                                checked: current_val == "true",
                                                onchange: {
                                                    let set_field = set_field.clone();
                                                    let fd_id = fd_id.clone();
                                                    let task_id = task_id.clone();
                                                    let cv = current_val.clone();
                                                    move |_| {
                                                        let new_val = if cv == "true" { "false" } else { "true" };
                                                        let set_field = set_field.clone();
                                                        let input = SetTaskFieldInput {
                                                            task_id: task_id.clone(),
                                                            field_id: fd_id.clone(),
                                                            value: new_val.to_string(),
                                                        };
                                                        spawn(async move {
                                                            let _ = set_field.call(input).await;
                                                        });
                                                    }
                                                },
                                            }
                                        },
                                        _ => rsx! {
                                            input {
                                                class: "detail-field-input",
                                                value: "{current_val}",
                                                placeholder: "Set {fd.key}...",
                                                onblur: {
                                                    let set_field = set_field.clone();
                                                    let fd_id = fd_id.clone();
                                                    let task_id = task_id.clone();
                                                    move |e: Event<FocusData>| {
                                                        // Read from the input value via web_sys
                                                        // For now, we use a simple approach
                                                    }
                                                },
                                                onchange: {
                                                    let set_field = set_field.clone();
                                                    let fd_id = fd_id.clone();
                                                    let task_id = task_id.clone();
                                                    move |e: Event<FormData>| {
                                                        let val = e.value();
                                                        if !val.is_empty() {
                                                            let set_field = set_field.clone();
                                                            let input = SetTaskFieldInput {
                                                                task_id: task_id.clone(),
                                                                field_id: fd_id.clone(),
                                                                value: val,
                                                            };
                                                            spawn(async move {
                                                                let _ = set_field.call(input).await;
                                                            });
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Due date
            div { class: "detail-section",
                label { class: "detail-label", "Due date" }
                input {
                    class: "detail-date-input",
                    r#type: "date",
                    value: "{due_date_val}",
                    onchange: {
                        let id = task.id.clone();
                        move |e: Event<FormData>| {
                            let val = e.value();
                            let due = if val.is_empty() {
                                UpdateTaskInput::new(id.clone()).due_at(None)
                            } else {
                                UpdateTaskInput::new(id.clone())
                                    .due_at(Some(format!("{val}T00:00:00Z")))
                            };
                            on_update.call(due);
                        }
                    },
                }
            }

            // Time spent
            if task.time_spent_secs > 0 || task.status == TaskStatus::Focused {
                div { class: "detail-section",
                    label { class: "detail-label", "Time spent" }
                    span { class: "detail-time", "{time_str}" }
                }
            }

            // Description
            div { class: "detail-section",
                label { class: "detail-label", "Description" }
                if *editing_desc.read() {
                    textarea {
                        class: "detail-desc-textarea",
                        value: "{desc_draft}",
                        rows: 8,
                        placeholder: "Add a description...",
                        oninput: move |e| desc_draft.set(e.value()),
                        onblur: {
                            let id = task.id.clone();
                            move |_| {
                                editing_desc.set(false);
                                let new_desc = desc_draft.read().clone();
                                if new_desc != task.description {
                                    on_update.call(
                                        UpdateTaskInput::new(id.clone()).description(new_desc),
                                    );
                                }
                            }
                        },
                    }
                } else if task.description.is_empty() {
                    p {
                        class: "detail-placeholder",
                        onclick: move |_| editing_desc.set(true),
                        "Add a description..."
                    }
                } else {
                    p {
                        class: "detail-description",
                        onclick: move |_| editing_desc.set(true),
                        "{task.description}"
                    }
                }
            }
        }
    }
}
