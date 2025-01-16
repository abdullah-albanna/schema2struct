use proc_macro2::Span;
use proc_macro_error::{abort, emit_error};

use crate::models::{JsonSchema, JsonSchemaTypes};

pub fn check_properties_match_required(schema: &JsonSchema) {
    if (schema.properties.is_none() && schema.properties_span.is_none())
        || (schema.required.is_none() && schema.required_span.is_none())
    {
        return;
    }

    let properties_keys: Vec<&String> = schema
        .properties
        .as_ref()
        .unwrap()
        .iter()
        .map(|(property_key, _)| property_key)
        .collect();

    let (properties_span, _) = schema.properties_span.as_ref().unwrap();

    let required = schema.required.as_ref().unwrap();
    let (required_span, _) = schema.required_span.as_ref().unwrap();

    if required.len() != properties_keys.len() {
        abort!(
            required_span,
            "make sure to implement all the required properties"
        )
    }

    if !properties_keys.iter().all(|key| required.contains(*key)) {
        abort!(
            properties_span,
            "make sure all the properties keys match what's in the required"
        );
    }
}

pub fn validate_keys(schema: &JsonSchema) {
    check_string_type(schema);
    check_number_type(schema);
    check_array_type(schema);
    check_object_type(schema);
    other_checks(schema);
}

fn check_object_type(schema: &JsonSchema) {
    fn report_error(span: Span, key: &str) {
        emit_error!(span, "you can't use `{} in a non object type`", key);
    }

    if !matches!(schema.ty, JsonSchemaTypes::Object) {
        if schema.required.is_some() {
            report_error(get_key_span(schema.required_span), "required");
        }

        if schema.properties.is_some() {
            report_error(get_key_span(schema.properties_span), "properties");
        }
    }
}

fn check_array_type(schema: &JsonSchema) {
    fn report_error(span: Span, key: &str) {
        emit_error!(span, "you can't use `{}` in a non array type", key);
    }

    if !matches!(schema.ty, JsonSchemaTypes::Array) {
        if schema.items.is_some() {
            report_error(get_key_span(schema.items_span), "items");
        }

        if schema.min_items.is_some() {
            report_error(get_key_span(schema.min_items_span), "min_items");
        }

        if schema.max_items.is_some() {
            report_error(get_key_span(schema.max_items_span), "max_items");
        }

        if schema.unique_items.is_some() {
            report_error(get_key_span(schema.unique_items_span), "unique_items");
        }

        if schema.contains.is_some() {
            report_error(get_key_span(schema.contains_span), "contains");
        }
    }
}

fn check_number_type(schema: &JsonSchema) {
    fn report_error(span: Span, key: &str) {
        emit_error!(span, "you can't use `{} in a non number type`", key);
    }

    if !matches!(schema.ty, JsonSchemaTypes::Number) {
        if schema.minimum.is_some() {
            report_error(get_key_span(schema.minimum_span), "minimum");
        }

        if schema.maximum.is_some() {
            report_error(get_key_span(schema.maximum_span), "maximum");
        }
    }
}

fn check_string_type(schema: &JsonSchema) {
    fn report_error(span: Span, key: &str) {
        emit_error!(span, "you can't use `{}` in a non string type", key);
    }

    if !matches!(schema.ty, JsonSchemaTypes::String) {
        if schema.min_lenght.is_some() {
            report_error(get_key_span(schema.min_lenght_span), "min_lenght");
        }

        if schema.max_lenght.is_some() {
            report_error(get_key_span(schema.max_lenght_span), "max_lenght");
        }

        if schema.pattern.is_some() {
            report_error(get_key_span(schema.pattern_span), "pattern");
        }

        if schema.format.is_some() {
            report_error(get_key_span(schema.format_span), "format");
        }
    }
}

fn get_key_span(have_span: Option<(Span, Span)>) -> Span {
    have_span.unwrap().0
}

pub fn other_checks(schema: &JsonSchema) {
    if schema.depth == 1 && schema.title.is_none() {
        if let Some((type_span, _)) = schema.ty_span {
            abort!(type_span, "the first title is required, consider adding it");
        }
    }
}
