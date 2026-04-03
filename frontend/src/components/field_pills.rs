use dioxus::prelude::*;

use crate::forge::{FieldDefinition, FieldValueType, TaskField};

#[component]
pub fn FieldPills(
    task_id: String,
    fields: Vec<FieldDefinition>,
    task_fields: Vec<TaskField>,
) -> Element {
    let pills: Vec<Element> = fields
        .iter()
        .filter_map(|fd| {
            let tf = task_fields
                .iter()
                .find(|tf| tf.task_id == task_id && tf.field_id == fd.id)?;

            let val = tf.value.as_str();
            if val.is_empty() {
                return None;
            }

            let color = fd.color.as_deref().unwrap_or("#6366f1");

            Some(match fd.value_type {
                FieldValueType::Enum => rsx! {
                    span {
                        class: "field-pill-enum",
                        style: "--pill-color: {color}",
                        "{val}"
                    }
                },
                FieldValueType::Bool if val == "true" => rsx! {
                    span { class: "field-pill-bool" }
                },
                FieldValueType::Int | FieldValueType::Decimal => rsx! {
                    span { class: "field-pill-number", "{val}" }
                },
                FieldValueType::Text => rsx! {
                    span { class: "field-pill-text", "{val}" }
                },
                _ => rsx! {},
            })
        })
        .collect();

    if pills.is_empty() {
        return rsx! {};
    }

    rsx! {
        for pill in pills {
            {pill}
        }
    }
}
