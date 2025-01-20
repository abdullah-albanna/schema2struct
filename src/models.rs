use std::collections::HashMap;

use create_json_schema_struct_macro::create_json_schema_struct;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Number, Value};

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
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaTypes {
    Object,
    String,
    Array,
    Number,

    // we make it the default so to know if it's fresh with ::default or has already been set
    #[default]
    None,
}

// the main struct holding all the data about every root and nested schemas
// creates the schema struct but adds the *_span for every key
//
// it also puts the depth field and current_key_span filed
create_json_schema_struct! {
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct JsonSchema {
        #[serde(rename = "type")]
        pub ty: JsonSchemaTypes,

        pub title: Option<String>,
        pub description: Option<String>,

        #[serde(rename = "default")]
        pub default: Option<JsonSchemaValues>,

        #[serde(rename = "examples")]
        pub examples: Option<Vec<String>>,

        #[serde(rename = "enum")]
        pub enum_values: Option<Vec<JsonSchemaValues>>,

        #[serde(rename = "const")]
        pub const_value: Option<JsonSchemaValues>,

        pub properties: Option<HashMap<String, JsonSchema>>,
        pub required: Option<Vec<String>>,
        pub min_lenght: Option<usize>,
        pub max_lenght: Option<usize>,
        pub pattern: Option<String>,
        pub format: Option<Formats>,
        pub minimum: Option<usize>,
        pub maximum: Option<usize>,
        pub items: Option<Box<JsonSchema>>,
        pub min_items: Option<usize>,
        pub max_items: Option<usize>,
        pub unique_items: Option<bool>,
        pub contains: Option<Box<JsonSchema>>,

    }
}

/// holds the different uses of the format key in string types
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
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
/// schema2struct!{
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
#[derive(Clone, Copy, Debug)]
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
    Struct,
}

/// stores what's after the `:`
///
/// ```rust
/// schema2struct!{
///     type: ... // the `...` is the value
/// }
///     
///    
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonSchemaValues {
    #[serde(
        serialize_with = "serialize_ident",
        deserialize_with = "deserialize_ident"
    )]
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

impl std::fmt::Display for JsonSchemaValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSchemaValues::Ident(ident) => f.write_str(&ident.to_string()),
            JsonSchemaValues::Str(s) => f.write_str(s),
            JsonSchemaValues::Number(num) => f.write_str(&format!("{}", num)),
            JsonSchemaValues::Bool(b) => f.write_str(&format!("{}", b)),
            JsonSchemaValues::Char(c) => f.write_str(&format!("{}", c)),
            JsonSchemaValues::Array(array) => f.write_str(&format!("{:?}", array)),
        }
    }
}

impl JsonSchema {
    /// converts the struct to an empty json
    ///
    /// ```rust
    /// struct Foo {
    ///     baz: String    
    /// }
    ///
    /// ```
    ///
    /// get's turned into
    ///
    /// ```json
    /// {
    ///     "baz": ""
    /// }
    /// ```
    ///
    /// it's used at the generation
    pub fn to_json_sample(&self) -> Value {
        let mut json = Map::new();

        if !matches!(self.ty, JsonSchemaTypes::Object) {
            return Self::get_in_type(self);
        }

        if let Some(struct_name) = self.struct_name.as_ref() {
            json.insert("struct_name".into(), Value::String(struct_name.to_owned()));
        }

        if let Some(properties) = self.properties.as_ref() {
            for (key, property) in properties {
                json.insert(key.to_owned(), Self::to_json_sample(property));
            }
        }

        json.into()
    }

    fn get_in_type(schema: &JsonSchema) -> Value {
        match schema.ty {
            JsonSchemaTypes::String => Value::String(String::new()),
            JsonSchemaTypes::None => Value::Null,
            JsonSchemaTypes::Number => Value::Number(Number::from(0)),
            JsonSchemaTypes::Array => {
                if let Some(items) = &schema.items {
                    Value::Array(vec![Self::get_in_type(items)])
                } else {
                    Value::Array(Vec::new())
                }
            }
            JsonSchemaTypes::Object => Value::Object(Map::new()),
        }
    }
}

// Custom serializer for Ident
fn serialize_ident<S>(ident: &syn::Ident, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&ident.to_string())
}

// Custom deserializer for Ident
fn deserialize_ident<'de, D>(deserializer: D) -> Result<syn::Ident, D::Error>
where
    D: Deserializer<'de>,
{
    struct IdentVisitor;

    impl<'de> Visitor<'de> for IdentVisitor {
        type Value = syn::Ident;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a string representing an identifier")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(syn::Ident::new(value, proc_macro2::Span::call_site()))
        }
    }

    deserializer.deserialize_str(IdentVisitor)
}
