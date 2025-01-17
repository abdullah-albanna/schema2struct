use proc_macro2::Span;
use proc_macro_error::{abort, emit_error};

use crate::models::{JsonSchema, JsonSchemaTypes};

pub fn check_properties_match_required(schema: &JsonSchema) {
    let Some((properties, properties_span)) = schema
        .properties
        .as_ref()
        .zip(schema.properties_span.as_ref())
    else {
        return;
    };

    let Some((required, required_span)) =
        schema.required.as_ref().zip(schema.required_span.as_ref())
    else {
        return;
    };

    let properties_keys: Vec<&String> = properties
        .iter()
        .map(|(property_key, _)| property_key)
        .collect();

    if required.len() != properties_keys.len() {
        abort!(
            required_span.0,
            "make sure to implement all the required properties"
        )
    }

    if !properties_keys.iter().all(|key| required.contains(*key)) {
        abort!(
            properties_span.0,
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
    have_span
        .expect("every key should have the span of it, this is a bug")
        .0
}

pub fn other_checks(schema: &JsonSchema) {
    if !matches!(schema.ty, JsonSchemaTypes::Object) && schema.struct_name.is_some() {
        if let Some((struct_name_span, _)) = schema.struct_name_span {
            emit_error!(
                struct_name_span,
                "`struct` is only allowed in an object type"
            )
        }
    }

    if schema.depth == 1 && schema.struct_name.is_none() {
        if let Some((type_span, _)) = schema.ty_span {
            abort!(
                type_span,
                "the first `struct` key is required, consider adding it"
            );
        }
    }

    // Check if the struct_name exists and if depth is 1
    if let Some(struct_name) = &schema.struct_name {
        if schema.depth == 1 {
            // If struct_name is "key", abort with an error message
            if struct_name == "key" {
                if let Some((_, struct_span)) = schema.struct_name_span {
                    abort!(struct_span, "you can't use `key` from the root schema");
                }
            }
        }
    }
}
