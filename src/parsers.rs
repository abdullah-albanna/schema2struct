use std::{collections::HashMap, sync::RwLock};

use proc_macro2::Span;
use proc_macro_error::{abort, emit_error};
use syn::{
    braced,
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    Result as SynResult, Token,
};

use crate::{
    checkers::{check_properties_match_required, validate_keys},
    models::{JsonSchema, JsonSchemaTypes},
};

use schema2struct_macros::update_schema_fields;

/// Tracks the current depth of schema parsing
///
/// Starts at 0 and increments for each nested schema level
/// Ensures proper handling of nested and root-level schemas
static SCHEMA_DEPTH: RwLock<usize> = RwLock::new(0);

impl Parse for JsonSchema {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if let Ok(mut w) = SCHEMA_DEPTH.try_write() {
            *w += 1;
        }

        let depth = if let Ok(i) = SCHEMA_DEPTH.try_read() {
            *i
        } else {
            0
        };

        let mut schema = JsonSchema {
            current_key_span: Some(input.span()),
            depth,
            ..Default::default()
        };

        let mut first_item = true;

        while !input.is_empty() {
            if !first_item {
                input.parse::<Token![,]>()?;
            }

            if input.is_empty() {
                break;
            }

            first_item = false;
            let key = input.call(syn::Ident::parse_any)?;
            let key_str = key.to_string();
            let key_span = key.span();

            if let Err(e) = input.parse::<Token![:]>() {
                emit_error!(e.span(), e);
            }

            let is_brace = input.peek(syn::token::Brace);

            if matches!(key_str.as_str(), "properties") && !is_brace {
                abort!(key, "expected `properties: {key: {...}, ...}`");
            }

            match key_str.as_str() {
                "properties" => {
                    let Properties { span, properties } = handle_properties(&input)?;

                    schema.properties = Some(properties);
                    schema.properties_span = Some((key_span, span));

                    // we must continue and not further parse, as it's not really needed
                    continue;
                }
                "items" => {
                    let Items { span, items_type } = handle_items(&mut schema, &input, &key_span)?;

                    // we can either use
                    //
                    // items: string
                    //
                    // or
                    //
                    // items: { type: string }
                    let type_schema = match items_type {
                        ItemsValue::Block(s) => s,
                        ItemsValue::Type(t) => JsonSchema {
                            ty: t,
                            ..Default::default()
                        },
                    };

                    schema.items = Some(Box::new(type_schema));
                    schema.items_span = Some((key_span, span));
                    continue;
                }

                "contains" => {
                    let Contains { span, contains } =
                        handle_contains(&mut schema, &input, &key_span)?;

                    let contains_schema = JsonSchema {
                        ty: contains,
                        ..Default::default()
                    };

                    schema.contains = Some(Box::new(contains_schema));
                    schema.contains_span = Some((key_span, span));
                    continue;
                }

                _ => {}
            };

            _ = input.parse::<Token![,]>();

            let value_expr: syn::Expr = input.parse()?;
            let value_span = value_expr.span();

            let value = JsonSchema::try_from((key, value_expr))?;

            // if the main schema is none and the other is not none, then we add items
            //
            // otherwise we skip
            if matches!(schema.ty, JsonSchemaTypes::None)
                && !matches!(value.ty, JsonSchemaTypes::None)
            {
                schema.ty = value.ty;
                schema.ty_span = Some((key_span, value_span));
            }

            // a helper macro that basiclly does
            //
            // ```rust
            //  if schema.#something.is_none() && value.#something.is_some() {
            //      schema.#something = value.#something;
            //      schema.#something_span = Some((key, value_span));
            //  }
            // ```
            update_schema_fields!(
                schema,
                value,
                key_span,
                value_span,
                [
                    minimum,
                    maximum,
                    min_items,
                    max_items,
                    unique_items,
                    contains,
                    default,
                    examples,
                    enum_values,
                    min_lenght,
                    max_lenght,
                    pattern,
                    format,
                    const_value,
                    description,
                    required,
                    properties,
                    title,
                    struct_name,
                ]
            );
        }

        if schema.required.is_some() && schema.properties.is_none() {
            if let Some((_, required_span)) = schema.required_span {
                abort!(
                    required_span,
                    "make sure to implement what's in the required"
                );
            }
        }

        if matches!(schema.ty, JsonSchemaTypes::None) {
            if let Some(current_key_span) = schema.current_key_span {
                abort!(current_key_span, "`type` must be set");
            }
        }

        check_properties_match_required(&schema);

        validate_keys(&schema);

        Ok(schema)
    }
}

/// used for the result of properties handlation
struct Properties {
    span: Span,
    properties: HashMap<String, JsonSchema>,
}

fn handle_properties(input: &ParseStream) -> Result<Properties, syn::Error> {
    let content;
    braced!(content in input);

    let mut properties = HashMap::new();
    let properties_span = content.span();

    let mut in_property_first_item = true;

    // TODO: improve this, doing too much
    // this is because a tralling comma will be counted as a tokenstream thus the
    // while check is still fine and get's in, but we check after the comma, if
    // it's empty then we stop because that's because of the tralling comma
    while !content.is_empty() {
        if !in_property_first_item {
            _ = content.parse::<Token![,]>()?;
        }

        if content.is_empty() {
            break;
        }

        let property_key: syn::LitStr = content.parse()?;

        _ = content.parse::<Token![:]>()?;

        let group: proc_macro2::Group = content.parse()?;

        if group.delimiter() != proc_macro2::Delimiter::Brace {
            abort!(group.span(), "Expected a brace-delimited group");
        }

        let nested_tokens = group.stream();
        let property_schema = syn::parse2::<JsonSchema>(nested_tokens)?;

        properties.insert(property_key.value(), property_schema);

        in_property_first_item = false;
    }
    _ = content.parse::<Token![,]>();

    Ok(Properties {
        span: properties_span,
        properties,
    })
}

/// used as a result for handling the items values
enum ItemsValue {
    Block(JsonSchema),
    Type(JsonSchemaTypes),
}

struct Items {
    span: Span,
    items_type: ItemsValue,
}

fn handle_items(
    schema: &mut JsonSchema,
    input: &ParseStream,
    key_span: &Span,
) -> Result<Items, syn::Error> {
    if input.peek(syn::Ident) {
        let type_ident: syn::Ident = input.parse()?;
        let type_ident_span = type_ident.span();

        let items_type = JsonSchemaTypes::try_from(type_ident)?;

        if schema.items.is_none() {
            Ok(Items {
                span: type_ident_span,
                items_type: ItemsValue::Type(items_type),
            })

            // schema.items = Some(items_type);
            // schema.items_span = Some((key_span, type_ident_span));
        } else {
            abort!(type_ident_span, "remove duplicated keys");
        }
    } else if input.peek(syn::token::Brace) {
        let group: proc_macro2::Group = input.parse()?;

        let nested_tokens = group.stream();
        let nested_tokens_span = nested_tokens.span();

        let nested_schema = syn::parse2::<JsonSchema>(nested_tokens)?;

        if schema.items.is_none() {
            return Ok(Items {
                span: nested_tokens_span,
                items_type: ItemsValue::Block(nested_schema),
            });
        } else {
            abort!(nested_tokens_span, "remove duplicated keys");
        }
    } else {
        abort!(
            key_span,
            "`items` value must be eithr a type `items: string` or a nested schema"
        );
    }
}

/// used as a result for handling the contains values
struct Contains {
    span: Span,
    contains: JsonSchemaTypes,
}

fn handle_contains(
    schema: &mut JsonSchema,
    input: &ParseStream,
    key_span: &Span,
) -> Result<Contains, syn::Error> {
    if input.peek(syn::Ident) {
        let contains_ident: syn::Ident = input.parse()?;
        let contains_ident_span = contains_ident.span();

        let contains = JsonSchemaTypes::try_from(contains_ident)?;

        if schema.contains.is_none() {
            Ok(Contains {
                span: contains_ident_span,
                contains,
            })
        } else {
            abort!(contains_ident_span, "remove duplicated keys");
        }
    } else if input.peek(syn::token::Brace) {
        let group: proc_macro2::Group = input.parse()?;

        let nested_tokens = group.stream();
        let nested_tokens_span = nested_tokens.span();

        let nested_schema = syn::parse2::<JsonSchema>(nested_tokens)?;

        if schema.contains.is_none() {
            return Ok(Contains {
                span: nested_tokens_span,
                contains: nested_schema.ty,
            });
        } else {
            abort!(nested_tokens_span, "remove duplicated keys");
        }
    } else {
        abort!(
            key_span,
            "`items` value must be eithr a type `items: string` or a nested schema"
        );
    }
}
