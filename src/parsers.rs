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
    checkers::{
        check_properties_match_required, check_that_every_key_is_in_the_right_place, other_checks,
    },
    models::{JsonSchema, JsonSchemaTypes},
};

use jsonschema_macros::update_schema_fields;

// we start with 0, and increment it from the begining by one, thus the root depth is 1
static SCHEMA_DEPTH: RwLock<usize> = RwLock::new(0);

impl Parse for JsonSchema {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if let Ok(mut w) = SCHEMA_DEPTH.try_write() {
            *w += 1;
        }

        let depth = if let Ok(i) = SCHEMA_DEPTH.try_read() {
            i.clone()
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

                    let type_schema = JsonSchema {
                        ty: items_type,
                        ..Default::default()
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

            if schema.title.is_empty() && !value.title.is_empty() {
                schema.title = value.title;
                schema.title_span = Some((key_span, value_span));
            }

            // if the main schema is none and the other is not none, then we add items
            //
            // otherwise we skip
            if matches!(schema.ty, JsonSchemaTypes::None)
                && !matches!(value.ty, JsonSchemaTypes::None)
            {
                schema.ty = value.ty;
                schema.ty_span = Some((key_span, value_span));
            }

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
                ]
            );
        }

        if schema.required.is_some() && schema.properties.is_none() {
            if let Some((_, required_span)) = schema.required_span {
                emit_error!(
                    required_span,
                    "make sure to implement what's in the required"
                );
            }
        }

        if let Some((properties, required)) =
            schema.properties.as_ref().zip(schema.required.as_ref())
        {
            if let Some(((_, properties_span), (_, required_span))) = schema
                .properties_span
                .as_ref()
                .zip(schema.required_span.as_ref())
            {
                let properties_keys = &properties.keys().collect::<Vec<_>>();
                check_properties_match_required(
                    properties_keys,
                    properties_span,
                    required,
                    required_span,
                );
            }
        }

        if matches!(schema.ty, JsonSchemaTypes::None) {
            abort!(schema.current_key_span.unwrap(), "`type` must be set");
        }

        check_that_every_key_is_in_the_right_place(&schema);
        other_checks(&schema);

        // if matches!(schema.ty, JsonSchemaTypes::Array) && schema.title.is_empty() {
        //     emit_error!(schema.current_key_span.unwrap(), "`title` must be set");
        // }

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
struct Items {
    span: Span,
    items_type: JsonSchemaTypes,
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
            return Ok(Items {
                span: type_ident_span,
                items_type,
            });

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
                items_type: nested_schema.ty,
            });

            // schema.items = Some(nested_schema.ty);
            // schema.items_span = Some((key_span, nested_tokens_span));
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
            return Ok(Contains {
                span: contains_ident_span,
                contains,
            });
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
