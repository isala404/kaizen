use dioxus::prelude::*;

use crate::forge::{
    CreateFieldDefinitionInput, DeleteFieldDefinitionInput, FieldValueType,
    use_create_field_definition, use_delete_field_definition, use_list_field_definitions_live,
};

fn random_color() -> String {
    use rand::Rng;
    const PALETTE: &[&str] = &[
        "#6366f1", "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
        "#f43f5e", "#ef4444", "#f97316", "#eab308", "#84cc16",
        "#22c55e", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6",
    ];
    let idx = rand::rng().random_range(0..PALETTE.len());
    PALETTE[idx].to_string()
}

#[component]
pub fn FieldManager(on_close: EventHandler<()>) -> Element {
    let fields_state = use_list_field_definitions_live();
    let create = use_create_field_definition();
    let delete = use_delete_field_definition();

    let mut new_key = use_signal(String::new);
    let mut new_type = use_signal(|| "text".to_string());
    let mut new_color = use_signal(random_color);
    let mut new_options = use_signal(String::new);

    let fields = fields_state.data.clone().unwrap_or_default();

    let mut on_create = {
        let create = create.clone();
        move |_: ()| {
            let key = new_key.read().trim().to_string();
            if key.is_empty() {
                return;
            }
            let vt = match new_type.read().as_str() {
                "enum" => FieldValueType::Enum,
                "bool" => FieldValueType::Bool,
                "int" => FieldValueType::Int,
                "decimal" => FieldValueType::Decimal,
                "list" => FieldValueType::List,
                "url" => FieldValueType::Url,
                _ => FieldValueType::Text,
            };
            let color = Some(new_color.read().clone());
            let options = if vt == FieldValueType::Enum {
                let opts: Vec<String> = new_options
                    .read()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Some(opts)
            } else {
                None
            };

            let create = create.clone();
            spawn(async move {
                let _ = create
                    .call(CreateFieldDefinitionInput {
                        key,
                        value_type: vt,
                        color,
                        options,
                    })
                    .await;
            });

            new_key.set(String::new());
            new_options.set(String::new());
            new_color.set(random_color());
        }
    };

    rsx! {
        div { class: "detail-overlay", onclick: move |_| on_close.call(()) }
        div {
            class: "field-manager-panel",
            onclick: move |e| e.stop_propagation(),

            div { class: "detail-header",
                h2 { class: "detail-title", "Manage Fields" }
                button { class: "detail-close", onclick: move |_| on_close.call(()), "×" }
            }

            // Existing fields
            div { class: "field-list",
                for field in &fields {
                    div { class: "field-list-item",
                        span { class: "field-list-key", "{field.key}" }
                        span { class: "field-list-type", "{format_type(&field.value_type)}" }
                        if let Some(ref opts) = field.options {
                            span { class: "field-list-opts", "{opts.join(\", \")}" }
                        }
                        button {
                            class: "task-action-btn task-action-delete",
                            onclick: {
                                let delete = delete.clone();
                                let id = field.id.clone();
                                move |_| {
                                    let delete = delete.clone();
                                    let id = id.clone();
                                    spawn(async move {
                                        let _ = delete.call(DeleteFieldDefinitionInput::new(id)).await;
                                    });
                                }
                            },
                            "×"
                        }
                    }
                }
            }

            // Add new field form
            div { class: "field-add-form",
                h3 { class: "detail-label", "Add field" }
                input {
                    class: "login-input",
                    placeholder: "Field key (e.g. priority)",
                    value: "{new_key}",
                    oninput: move |e| new_key.set(e.value()),
                }
                select {
                    class: "field-select",
                    value: "{new_type}",
                    onchange: move |e| new_type.set(e.value()),
                    option { value: "text", "Text" }
                    option { value: "enum", "Enum" }
                    option { value: "bool", "Boolean" }
                    option { value: "int", "Integer" }
                    option { value: "decimal", "Decimal" }
                    option { value: "list", "List" }
                    option { value: "url", "URL" }
                }
                if *new_type.read() == "enum" {
                    input {
                        class: "login-input",
                        placeholder: "Options (comma separated)",
                        value: "{new_options}",
                        oninput: move |e| new_options.set(e.value()),
                    }
                }
                input {
                    class: "field-color-input",
                    r#type: "color",
                    value: "{new_color}",
                    oninput: move |e| new_color.set(e.value()),
                }
                button {
                    class: "login-button",
                    onclick: move |_| on_create(()),
                    "Add Field"
                }
            }
        }
    }
}

fn format_type(vt: &FieldValueType) -> &'static str {
    match vt {
        FieldValueType::Text => "text",
        FieldValueType::Enum => "enum",
        FieldValueType::Bool => "bool",
        FieldValueType::Int => "int",
        FieldValueType::Decimal => "decimal",
        FieldValueType::List => "list",
        FieldValueType::Url => "url",
    }
}
