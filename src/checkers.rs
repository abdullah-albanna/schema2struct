use proc_macro_error::emit_error;

use crate::models::{JsonSchema, JsonSchemaTypes};

pub fn check_properties_match_required(
    properties: &[&String],
    properties_span: &proc_macro2::Span,
    required: &[String],
    required_span: &proc_macro2::Span,
) {
    if properties.len() != required.len() {
        emit_error!(
            required_span,
            "make sure you implement all the required in the properties"
        );
    }

    let result = properties
        .iter()
        .zip(required)
        .all(|(properties_title, required_name)| properties_title.eq(&required_name));

    if !result {
        emit_error!(
            properties_span,
            "make sure the properties titles match the required fields"
        );
    }
}

pub fn check_that_every_key_is_in_the_right_place(schema: &JsonSchema) {
    // makes sure that an item is only used in a an array type schema
    if !matches!(schema.ty, JsonSchemaTypes::Array) && schema.items.is_some() {
        if let Some((item_span, _)) = schema.items_span {
            emit_error!(item_span, "you can't use `items` in a non array type")
        }
    }

    if !matches!(schema.ty, JsonSchemaTypes::Object) && schema.properties.is_some() {
        if let Some((property_span, _)) = schema.properties_span {
            emit_error!(
                property_span,
                "you can't use `properties` in a non object type"
            );
        }
    }

    if !matches!(schema.ty, JsonSchemaTypes::Object) && schema.required.is_some() {
        if let Some((required_span, _)) = schema.required_span {
            emit_error!(
                required_span,
                "you can't use `required` in a non object type"
            )
        }
    }
}

pub fn other_checks(schema: &JsonSchema) {
    if schema.depth == 1 && schema.title.is_empty() {
        if let Some((type_span, _)) = schema.ty_span {
            emit_error!(type_span, "the first title is required, consider adding it");
        }
    }
}
