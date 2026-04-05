use std::collections::HashMap;

use dioxus::prelude::*;
use js_sys::Function;
use pulldown_cmark::{Options, Parser, html};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::forge::{
    FieldDefinition, FieldValueType, SetTaskFieldInput, Task, TaskField, TaskStatus,
    UpdateTaskInput, use_set_task_field,
};
use crate::time_utils;

fn highlight_code_blocks() {
    spawn(async {
        // hljs loads async from CDN, give it a moment
        gloo_timers::future::TimeoutFuture::new(50).await;
        if let Some(window) = web_sys::window() {
            if let Ok(hljs) = js_sys::Reflect::get(&window, &JsValue::from_str("hljs")) {
                if let Ok(func) = js_sys::Reflect::get(&hljs, &JsValue::from_str("highlightAll")) {
                    if let Ok(f) = func.dyn_into::<Function>() {
                        let _ = f.call0(&hljs);
                    }
                }
            }
        }
    });
}

fn render_markdown(input: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(input, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[component]
pub fn DetailPanel(
    task: Task,
    field_defs: Option<Vec<FieldDefinition>>,
    task_fields: Option<Vec<TaskField>>,
    on_close: EventHandler<()>,
    on_update: EventHandler<UpdateTaskInput>,
) -> Element {
    let set_field = use_set_task_field();

    let mut editing_desc = use_signal(|| false);
    let mut desc_draft = use_signal(|| task.description.clone());
    let mut title_draft = use_signal(|| task.title.clone());
    let mut field_drafts = use_signal(HashMap::<String, String>::new);
    let mut fullscreen = use_signal(|| {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("detail_fullscreen").ok().flatten())
            .is_some_and(|v| v == "true")
    });

    let desc_for_effect = task.description.clone();
    let is_editing = *editing_desc.read();
    use_effect(move || {
        if !is_editing && !desc_for_effect.is_empty() {
            highlight_code_blocks();
        }
    });

    let save_desc = {
        let id = task.id.clone();
        let desc = task.description.clone();
        move || {
            if *editing_desc.read() {
                editing_desc.set(false);
                let new_desc = desc_draft.read().clone();
                if new_desc != desc {
                    on_update.call(UpdateTaskInput::new(id.clone()).description(new_desc));
                }
            }
        }
    };

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
            onclick: {
                let mut save_desc = save_desc.clone();
                move |_| {
                    save_desc();
                    on_close.call(());
                }
            },
        }
        div {
            class: if *fullscreen.read() { "detail-panel detail-panel-full" } else { "detail-panel" },
            onclick: {
                let mut save_desc = save_desc.clone();
                move |e: Event<MouseData>| {
                    e.stop_propagation();
                    save_desc();
                }
            },

            div { class: "detail-header",
                span {}
                div { class: "detail-header-actions",
                    button {
                        class: "detail-expand",
                        title: if *fullscreen.read() { "Exit full screen" } else { "Full screen" },
                        onclick: move |_| {
                            let next = !*fullscreen.read();
                            fullscreen.set(next);
                            if let Some(storage) = web_sys::window()
                                .and_then(|w| w.local_storage().ok().flatten())
                            {
                                let _ = storage.set_item("detail_fullscreen", if next { "true" } else { "false" });
                            }
                        },
                        if *fullscreen.read() { "⊟" } else { "⊞" }
                    }
                    button {
                        class: "detail-close",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
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
                                        FieldValueType::Url => {
                                            let display_val = field_drafts.read()
                                                .get(&fd_id)
                                                .cloned()
                                                .unwrap_or_else(|| current_val.clone());
                                            rsx! {
                                                div { class: "detail-field-url",
                                                    input {
                                                        class: "detail-field-input",
                                                        r#type: "url",
                                                        value: "{display_val}",
                                                        placeholder: "https://...",
                                                        onfocus: {
                                                            let fd_id = fd_id.clone();
                                                            let current_val = current_val.clone();
                                                            move |_| {
                                                                field_drafts.write().insert(fd_id.clone(), current_val.clone());
                                                            }
                                                        },
                                                        oninput: {
                                                            let fd_id = fd_id.clone();
                                                            move |e: Event<FormData>| {
                                                                field_drafts.write().insert(fd_id.clone(), e.value());
                                                            }
                                                        },
                                                        onblur: {
                                                            let set_field = set_field.clone();
                                                            let fd_id = fd_id.clone();
                                                            let task_id = task_id.clone();
                                                            move |_| {
                                                                let val = field_drafts.write().remove(&fd_id).unwrap_or_default();
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
                                                    if !current_val.is_empty() {
                                                        a {
                                                            class: "detail-field-url-link",
                                                            href: "{current_val}",
                                                            target: "_blank",
                                                            "↗"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            let display_val = field_drafts.read()
                                                .get(&fd_id)
                                                .cloned()
                                                .unwrap_or_else(|| current_val.clone());
                                            rsx! {
                                                input {
                                                    class: "detail-field-input",
                                                    value: "{display_val}",
                                                    placeholder: "Set {fd.key}...",
                                                    onfocus: {
                                                        let fd_id = fd_id.clone();
                                                        let current_val = current_val.clone();
                                                        move |_| {
                                                            field_drafts.write().insert(fd_id.clone(), current_val.clone());
                                                        }
                                                    },
                                                    oninput: {
                                                        let fd_id = fd_id.clone();
                                                        move |e: Event<FormData>| {
                                                            field_drafts.write().insert(fd_id.clone(), e.value());
                                                        }
                                                    },
                                                    onblur: {
                                                        let set_field = set_field.clone();
                                                        let fd_id = fd_id.clone();
                                                        let task_id = task_id.clone();
                                                        move |_| {
                                                            let val = field_drafts.write().remove(&fd_id).unwrap_or_default();
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
            div { class: "detail-section detail-section-desc",
                label { class: "detail-label", "Description" }
                if *editing_desc.read() {
                    textarea {
                        class: "detail-desc-textarea",
                        onclick: move |e: Event<MouseData>| e.stop_propagation(),
                        value: "{desc_draft}",
                        rows: desc_draft.read().lines().count().max(8) as i64,
                        placeholder: "Add a description...",
                        oninput: move |e| desc_draft.set(e.value()),
                    }
                } else if task.description.is_empty() {
                    p {
                        class: "detail-placeholder",
                        onclick: move |e: Event<MouseData>| {
                            e.stop_propagation();
                            editing_desc.set(true);
                        },
                        "Add a description..."
                    }
                } else {
                    document::Link {
                        rel: "stylesheet",
                        href: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css",
                    }
                    document::Script {
                        src: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js",
                    }
                    div {
                        class: "detail-description markdown-body",
                        onclick: move |e: Event<MouseData>| {
                            e.stop_propagation();
                            editing_desc.set(true);
                        },
                        dangerous_inner_html: render_markdown(&task.description),
                    }
                }
            }
        }
    }
}
