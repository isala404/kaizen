use dioxus::prelude::*;

use crate::forge::{FieldDefinition, FieldValueType, TaskField};

#[component]
pub fn FieldPills(
    task_id: String,
    field_defs: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
) -> Element {
    let pills: Vec<_> = field_defs
        .iter()
        .flat_map(|def| {
            let tf = match task_fields
                .iter()
                .find(|tf| tf.task_id == task_id && tf.field_id == def.id)
            {
                Some(tf) if !tf.value.is_empty() => tf,
                _ => return vec![],
            };

            let color = def.color.as_deref().unwrap_or("#6366f1");

            match def.value_type {
                FieldValueType::Bool => {
                    if tf.value != "true" {
                        return vec![];
                    }
                    vec![rsx! {
                        span {
                            key: "{def.id}",
                            class: "task-card-pill",
                            style: "--pill-color: {color};",
                            "{def.key}"
                        }
                    }]
                }
                FieldValueType::Url => vec![rsx! {
                    span {
                        key: "{def.id}",
                        class: "task-card-pill",
                        style: "--pill-color: {color};",
                        "{def.key} \u{2197}"
                    }
                }],
                FieldValueType::List => {
                    let joined = if tf.value.starts_with('[') {
                        serde_json::from_str::<Vec<String>>(&tf.value)
                            .unwrap_or_default()
                            .join(" | ")
                    } else {
                        tf.value
                            .split(',')
                            .map(|item| item.trim())
                            .filter(|item| !item.is_empty())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    };
                    if joined.is_empty() {
                        return vec![];
                    }
                    vec![rsx! {
                        span {
                            key: "{def.id}",
                            class: "task-card-pill",
                            style: "--pill-color: {color};",
                            "{joined}"
                        }
                    }]
                }
                _ => vec![rsx! {
                    span {
                        key: "{def.id}",
                        class: "task-card-pill",
                        style: "--pill-color: {color};",
                        "{tf.value}"
                    }
                }],
            }
        })
        .collect();

    if pills.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "task-card-pills",
            {pills.into_iter()}
        }
    }
}
