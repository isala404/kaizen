use rand::Rng;

use crate::forge::FieldValueType;

const FIELD_COLOR_PALETTE: &[&str] = &[
    "#6366f1", "#8b5cf6", "#a855f7", "#d946ef", "#ec4899", "#f43f5e", "#ef4444", "#f97316",
    "#eab308", "#84cc16", "#22c55e", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6",
];

pub fn random_field_color() -> String {
    let index = rand::rng().random_range(0..FIELD_COLOR_PALETTE.len());
    FIELD_COLOR_PALETTE[index].to_string()
}

pub fn parse_field_value_type(value: &str) -> FieldValueType {
    match value {
        "enum" => FieldValueType::Enum,
        "bool" => FieldValueType::Bool,
        "int" => FieldValueType::Int,
        "decimal" => FieldValueType::Decimal,
        "list" => FieldValueType::List,
        "url" => FieldValueType::Url,
        _ => FieldValueType::Text,
    }
}

pub fn format_field_value_type(value_type: &FieldValueType) -> &'static str {
    match value_type {
        FieldValueType::Text => "text",
        FieldValueType::Enum => "enum",
        FieldValueType::Bool => "bool",
        FieldValueType::Int => "int",
        FieldValueType::Decimal => "decimal",
        FieldValueType::List => "list",
        FieldValueType::Url => "url",
    }
}

pub fn parse_field_options(value_type: &FieldValueType, raw_options: &str) -> Option<Vec<String>> {
    if *value_type != FieldValueType::Enum {
        return None;
    }

    let options = raw_options
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    Some(options)
}
