use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// used to identify what type is current schema
///
/// ```rust
///
/// jsonschema!{
///     type: object,
///     ...
/// }
/// ```
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum JsonSchemaTypes {
    Object,
    String,
    Array,
    Number,

    // we make it the default so to know if it's fresh with ::default or has already been set
    #[default]
    None,
}

/// the main struct holding all the data about every root and nested schemas
#[derive(Clone, Debug, Default)]
pub struct JsonSchema {
    pub ty: JsonSchemaTypes,
    pub title: String,
    pub required: Option<Vec<String>>,
    pub description: Option<String>,
    pub default: Option<JsonSchemaValues>,
    pub examples: Option<Vec<String>>,
    pub enum_values: Option<Vec<JsonSchemaValues>>,
    pub const_value: Option<JsonSchemaValues>,

    // object specific keys
    pub properties: Option<HashMap<String, JsonSchema>>,

    // string specific keys
    pub min_lenght: Option<usize>,
    pub max_lenght: Option<usize>,
    pub pattern: Option<String>,
    pub format: Option<Formats>,

    // number specific keys
    pub minimum: Option<usize>,
    pub maximum: Option<usize>,

    // array specific keys
    pub items: Option<Box<JsonSchema>>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: Option<bool>,
    pub contains: Option<Box<JsonSchema>>,

    // span tracking fields
    // it's a key value spans
    pub current_key_span: Option<proc_macro2::Span>,

    /// those are used to report better errors, to exactly tell where the errors are
    pub ty_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub title_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub required_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub description_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub items_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub properties_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub default_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub examples_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub enum_values_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub const_value_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub min_lenght_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub max_lenght_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub pattern_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub format_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub minimum_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub maximum_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub min_items_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub max_items_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub unique_items_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
    pub contains_span: Option<(proc_macro2::Span, proc_macro2::Span)>,

    // indicate the depth of the schema starting from 1 as root
    pub depth: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Formats {
    Date,
    Time,
    DateTime,
    Email,
    Hostname,
    Ipv4,
    Ipv6,
    Uri,
}

impl std::fmt::Display for Formats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formats::Date => f.write_str("date"),
            Formats::Time => f.write_str("time"),
            Formats::DateTime => f.write_str("date-time"),
            Formats::Email => f.write_str("email"),
            Formats::Hostname => f.write_str("hostname"),
            Formats::Ipv4 => f.write_str("ipv4"),
            Formats::Ipv6 => f.write_str("ipv6"),
            Formats::Uri => f.write_str("uri"),
        }
    }
}

/// contains every ident that's considered as a keyword
///
/// ```rust
/// jsonschema!{
///     type: ...,
///     title: "...",
///     default: "...",
///     required: [...],
///     description: "...",
///     items: ..., // you can also use { type: ... }
///     properties: {
///         "...": {...}
///     }
///    
/// }
/// ```
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum JsonSchemaKeywords {
    Type,
    Title,
    Required,
    Description,
    Items,
    Properties,
    Default,
    Examples,
    Enum,
    Const,
    MinLength,
    MaxLenght,
    Pattern,
    Format,
    Minimum,
    Maximum,
    MinItems,
    MaxItems,
    UniqueItems,
    Contains,
}

/// stores what's after the `:`
///
/// ```rust
/// jsonschema!{
///     type: ... // the `...` is the value
/// }
///     
///    
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum JsonSchemaValues {
    Ident(syn::Ident),
    Str(String),
    Number(i64),
    Bool(bool),
    Char(char),
    Array(Vec<JsonSchemaValues>),
}

impl JsonSchemaValues {
    pub fn get_str(&self) -> Option<&String> {
        match self {
            JsonSchemaValues::Str(ref s) => Some(s),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl JsonSchemaTypes {
    pub fn is_none(&self) -> bool {
        matches!(self, JsonSchemaTypes::None)
    }
}

impl std::fmt::Display for JsonSchemaTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSchemaTypes::Array => f.write_str("array"),
            JsonSchemaTypes::Object => f.write_str("object"),
            JsonSchemaTypes::String => f.write_str("string"),
            JsonSchemaTypes::Number => f.write_str("number"),
            JsonSchemaTypes::None => f.write_str("null"),
        }
    }
}
