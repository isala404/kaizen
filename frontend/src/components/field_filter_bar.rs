use dioxus::prelude::*;
use std::collections::HashMap;

use crate::forge::{FieldDefinition, FieldValueType, TaskField};

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFilters(pub HashMap<String, Vec<String>>);

impl ActiveFilters {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn toggle(&mut self, field_id: &str, value: &str) {
        let entry = self.0.entry(field_id.to_string()).or_default();
        if let Some(pos) = entry.iter().position(|v| v == value) {
            entry.remove(pos);
            if entry.is_empty() {
                self.0.remove(field_id);
            }
        } else {
            entry.push(value.to_string());
        }
    }

    pub fn is_active(&self, field_id: &str, value: &str) -> bool {
        self.0
            .get(field_id)
            .is_some_and(|vals| vals.contains(&value.to_string()))
    }

    /// Check if a task passes all active filters.
    pub fn matches(&self, task_id: &str, task_fields: &[TaskField]) -> bool {
        if self.0.is_empty() {
            return true;
        }
        for (field_id, required_values) in &self.0 {
            let task_value = task_fields
                .iter()
                .find(|tf| tf.task_id == task_id && tf.field_id == *field_id)
                .map(|tf| tf.value.as_str());

            match task_value {
                Some(val) => {
                    if !required_values.contains(&val.to_string()) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }
}

#[component]
pub fn FieldFilterBar(
    fields: Vec<FieldDefinition>,
    filters: ActiveFilters,
    on_toggle: EventHandler<(String, String)>,
) -> Element {
    if fields.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "field-filter-bar",
            for field in &fields {
                match field.value_type {
                    FieldValueType::Enum => {
                        let options = field.options.clone().unwrap_or_default();
                        let color = field.color.clone().unwrap_or_else(|| "#6366f1".into());
                        rsx! {
                            for opt in options {
                                {
                                    let active = filters.is_active(&field.id, &opt);
                                    let fid = field.id.clone();
                                    let val = opt.clone();
                                    rsx! {
                                        button {
                                            class: if active { "filter-pill filter-pill-active" } else { "filter-pill" },
                                            style: "--pill-color: {color}",
                                            onclick: move |_| on_toggle.call((fid.clone(), val.clone())),
                                            "{opt}"
                                            if active {
                                                span { class: "filter-pill-x", " ×" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    FieldValueType::Bool => {
                        let active = filters.is_active(&field.id, "true");
                        let fid = field.id.clone();
                        rsx! {
                            button {
                                class: if active { "filter-pill filter-pill-active" } else { "filter-pill" },
                                onclick: move |_| on_toggle.call((fid.clone(), "true".to_string())),
                                "{field.key}"
                            }
                        }
                    }
                    _ => rsx! {}
                }
            }
        }
    }
}
