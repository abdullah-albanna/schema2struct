use proc_macro_error::{abort, OptionExt};
use syn::spanned::Spanned as _;

use crate::models::{Formats, JsonSchema, JsonSchemaKeywords, JsonSchemaTypes, JsonSchemaValues};

// ----
impl TryFrom<syn::Ident> for JsonSchemaTypes {
    type Error = syn::Error;

    fn try_from(value: syn::Ident) -> std::result::Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "array" => Ok(Self::Array),
            "object" => Ok(Self::Object),
            "string" => Ok(Self::String),
            "number" => Ok(Self::Number),
            _ => Err(syn::Error::new(value.span(), "Unknown type")),
        }
    }
}
// ----

// ----
impl TryFrom<syn::Expr> for JsonSchemaValues {
    type Error = syn::Error;

    fn try_from(value: syn::Expr) -> std::result::Result<Self, Self::Error> {
        match value {
            syn::Expr::Path(path) if path.path.segments.len() == 1 => {
                let ident = path
                    .path
                    .segments
                    .first()
                    .expect("We already checked the length")
                    .clone()
                    .ident;

                Ok(JsonSchemaValues::Ident(ident))
            }

            syn::Expr::Lit(literal) => match literal.lit {
                syn::Lit::Str(s) => Ok(JsonSchemaValues::Str(s.value())),
                syn::Lit::Int(int) => Ok(JsonSchemaValues::Number(
                    int.base10_parse().unwrap_or_default(),
                )),
                syn::Lit::Bool(b) => Ok(JsonSchemaValues::Bool(b.value)),
                syn::Lit::Char(ch) => Ok(JsonSchemaValues::Char(ch.value())),
                _ => Err(syn::Error::new(literal.span(), "invalid literal")),
            },
            syn::Expr::Array(array) => {
                let mut elements = vec![];
                for element in array.elems {
                    elements.push(JsonSchemaValues::try_from(element)?);
                }

                Ok(JsonSchemaValues::Array(elements))
            }

            _ => Err(syn::Error::new(value.span(), "Unsupported expression type")),
        }
    }
}
// ----

// ----
impl TryFrom<syn::Ident> for JsonSchemaKeywords {
    type Error = syn::Error;

    fn try_from(value: syn::Ident) -> std::result::Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "type" => Ok(JsonSchemaKeywords::Type),
            "title" => Ok(JsonSchemaKeywords::Title),
            "required" => Ok(JsonSchemaKeywords::Required),
            "description" => Ok(JsonSchemaKeywords::Description),
            "items" => Ok(JsonSchemaKeywords::Items),
            "properties" => Ok(JsonSchemaKeywords::Properties),
            "default" => Ok(JsonSchemaKeywords::Default),
            "examples" => Ok(JsonSchemaKeywords::Examples),
            "enum" => Ok(JsonSchemaKeywords::Enum),
            "const" => Ok(JsonSchemaKeywords::Const),
            "min_length" => Ok(JsonSchemaKeywords::MinLength),
            "max_length" => Ok(JsonSchemaKeywords::MaxLenght),
            "pattern" => Ok(JsonSchemaKeywords::Pattern),
            "format" => Ok(JsonSchemaKeywords::Format),
            "minimum" => Ok(JsonSchemaKeywords::Minimum),
            "maximum" => Ok(JsonSchemaKeywords::Maximum),
            "max_items" => Ok(JsonSchemaKeywords::MaxItems),
            "min_items" => Ok(JsonSchemaKeywords::MinItems),
            "unique_items" => Ok(JsonSchemaKeywords::UniqueItems),
            "contains" => Ok(JsonSchemaKeywords::Contains),
            "struct" => Ok(JsonSchemaKeywords::Struct),
            _ => Err(syn::Error::new(value.span(), "Unknown keyword")),
        }
    }
}
// ----

// ----

impl TryFrom<syn::Ident> for Formats {
    type Error = syn::Error;

    fn try_from(value: syn::Ident) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "date" => Ok(Formats::Date),
            "time" => Ok(Formats::Time),
            "date-time" => Ok(Formats::DateTime),
            "email" => Ok(Formats::Email),
            "hostname" => Ok(Formats::Hostname),
            "ipv4" => Ok(Formats::Ipv4),
            "ipv6" => Ok(Formats::Ipv6),
            "uri" => Ok(Formats::Uri),
            _ => {
             Err(syn::Error::new(
                    value.span(),
                    "unsupported format, avaliables are: `data`, `time`, `date-time`, `email`, `hostname`, `ipv4`, `ipv6`, `uri`",
                ))
            }
        }
    }
}

// ----

// ---
impl TryFrom<(syn::Ident, syn::Expr)> for JsonSchema {
    type Error = syn::Error;

    fn try_from(value: (syn::Ident, syn::Expr)) -> std::result::Result<Self, Self::Error> {
        let key = value.0;
        let value = value.1;
        let value_span = value.span();

        let schema_key = JsonSchemaKeywords::try_from(key)?;
        let schema_value = JsonSchemaValues::try_from(value)?;

        let mut schema = Self::default();

        match schema_key {
            JsonSchemaKeywords::Type => match schema_value {
                JsonSchemaValues::Ident(ident) => schema.ty = JsonSchemaTypes::try_from(ident)?,
                _ => return Err(syn::Error::new(value_span, "Invalid type")),
            },

            JsonSchemaKeywords::Struct => match schema_value {
                JsonSchemaValues::Ident(ident) => schema.struct_name = Some(ident.to_string()),
                _ => return Err(syn::Error::new(value_span, "only idents are allowed")),
            },

            JsonSchemaKeywords::UniqueItems => match schema_value {
                JsonSchemaValues::Bool(b) => schema.unique_items = Some(b),
                _ => return Err(syn::Error::new(value_span, "only boolean is allowed")),
            },

            JsonSchemaKeywords::MinItems => match schema_value {
                JsonSchemaValues::Number(num) => schema.min_items = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },

            JsonSchemaKeywords::MaxItems => match schema_value {
                JsonSchemaValues::Number(num) => schema.max_items = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },

            JsonSchemaKeywords::Minimum => match schema_value {
                JsonSchemaValues::Number(num) => schema.minimum = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },
            JsonSchemaKeywords::Maximum => match schema_value {
                JsonSchemaValues::Number(num) => schema.maximum = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },

            JsonSchemaKeywords::MinLength => match schema_value {
                JsonSchemaValues::Number(num) => schema.min_lenght = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },

            JsonSchemaKeywords::MaxLenght => match schema_value {
                JsonSchemaValues::Number(num) => schema.max_lenght = Some(num as usize),
                _ => return Err(syn::Error::new(value_span, "only number is allowed")),
            },

            JsonSchemaKeywords::Pattern => match schema_value {
                JsonSchemaValues::Str(s) => schema.pattern = Some(s),
                _ => return Err(syn::Error::new(value_span, "only string is allowed")),
            },

            JsonSchemaKeywords::Format => match schema_value {
                JsonSchemaValues::Ident(ident) => {
                    let format = Formats::try_from(ident)?;

                    schema.format = Some(format);
                }
                _ => return Err(syn::Error::new(value_span, "only idents are supported")),
            },
            JsonSchemaKeywords::Examples => match schema_value {
                JsonSchemaValues::Array(examples) => {
                    for example in examples.iter() {
                        if !matches!(example, JsonSchemaValues::Str(_)) {
                            return Err(syn::Error::new(
                                value_span,
                                "examples should all be string",
                            ));
                        }
                    }

                    let examples = examples
                        .iter()
                        .map(|value| {
                            value
                                .get_str()
                                .cloned()
                                .expect_or_abort("couldn't get the strings from the examples array")
                        })
                        .collect();

                    schema.examples = Some(examples);
                }
                _ => {
                    return Err(syn::Error::new(
                        value_span,
                        "examples should be inside of an array",
                    ))
                }
            },

            JsonSchemaKeywords::Enum => match schema_value {
                JsonSchemaValues::Array(enum_values) => {
                    for value in enum_values.iter() {
                        if let JsonSchemaValues::Ident(ident) = value {
                            return Err(syn::Error::new(
                                ident.span(),
                                "enum should contain values, not idents",
                            ));
                        }
                    }

                    schema.enum_values = Some(enum_values)
                }

                _ => {
                    return Err(syn::Error::new(
                        value_span,
                        "enum should be inside of an array",
                    ))
                }
            },

            JsonSchemaKeywords::Const => match schema_value {
                JsonSchemaValues::Ident(ident) => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "const value can't be an ident",
                    ))
                }
                JsonSchemaValues::Array(_) => {
                    return Err(syn::Error::new(value_span, "const value can't be an array"))
                }
                value => schema.const_value = Some(value),
            },

            JsonSchemaKeywords::Default => match schema_value {
                JsonSchemaValues::Ident(ident) => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "default value can't be an ident",
                    ))
                }
                value => schema.default = Some(value),
            },

            JsonSchemaKeywords::Title => match schema_value {
                JsonSchemaValues::Str(s) => {
                    schema.title = Some(s);
                }
                _ => return Err(syn::Error::new(value_span, "title must be a string")),
            },
            JsonSchemaKeywords::Description => match schema_value {
                JsonSchemaValues::Str(s) => schema.description = Some(s),
                _ => return Err(syn::Error::new(value_span, "description must be a string")),
            },

            JsonSchemaKeywords::Required => match schema_value {
                JsonSchemaValues::Array(array) => {
                    let are_all_str = array.iter().all(|v| matches!(v, JsonSchemaValues::Str(_)));

                    if !are_all_str {
                        abort!(value_span, "the array must be all string");
                    }

                    let mut collected_items = vec![];

                    for item in array {
                        match item {
                            JsonSchemaValues::Str(s) => collected_items.push(s),
                            _ => {
                                abort!(value_span, "the array must be all string");
                            }
                        }
                    }

                    schema.required = Some(collected_items);
                }
                _ => {
                    abort!(value_span, "the `required` field must be an array");
                }
            },

            JsonSchemaKeywords::Properties => unreachable!("it's already handled at parsing"),
            JsonSchemaKeywords::Items => unreachable!("it's already handled at parsing"),
            JsonSchemaKeywords::Contains => unreachable!("it's already handled at parsing"),
        }

        Ok(schema)
    }
}
