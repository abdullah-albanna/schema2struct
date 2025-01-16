use inflections::Inflect;
use quote::{format_ident, quote, ToTokens};
use serde_json::{Map, Number, Value};
use syn::Ident;

use crate::models::{JsonSchema, JsonSchemaTypes};

pub struct JsonMacroInput {
    pub struct_name: Ident,
    pub content: Value,
}
/// Generates Rust structs from a JSON-like structure with flexible configuration.
///
/// # Parameters
/// - `json_struct`: The input JSON macro structure
/// - `base_name`: The base name for the primary struct
///
/// # Returns
/// A tuple containing:
/// 1. The main generated struct as a token stream
/// 2. A vector of additional nested structs
pub fn generate_structs(
    json_struct: &JsonMacroInput,
    base_name: &Ident,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    // Collect all generated structs
    let mut all_structs = Vec::new();
    let mut fields = Vec::new();

    // Process each entry in the JSON-like structure
    for (key, value) in json_struct.content.as_object().unwrap() {
        let key = key.to_snake_case();
        // Just in case the identifier is not a valid struct name
        let field_name = format_ident!("{}", key);

        // Infer field type and handle nested structures
        let field_type = match value {
            Value::String(_) => quote!(String),
            Value::Number(_) => quote!(f64),
            Value::Bool(_) => quote!(bool),

            Value::Array(arr) => {
                let (elem_type, _) = infer_array_type(arr);
                quote!(Vec<#elem_type>)
            }

            Value::Object(obj) => {
                // Generate nested struct for object and concat the key with the struct name
                //
                // `Example`
                //
                //```rust
                //
                // struct User {
                //  age: UserAge
                // }
                //
                // struct UserAge;
                //
                //````
                let nested_name = format_ident!("{}{}", base_name, key.to_pascal_case());

                let nested_macro_input = JsonMacroInput {
                    struct_name: json_struct.struct_name.clone(),
                    content: Value::Object(obj.clone()),
                };

                // Recursively generate nested structs
                let (nested_struct, nested_structs) =
                    generate_structs(&nested_macro_input, &nested_name);

                all_structs.extend(nested_structs);
                all_structs.push(nested_struct.clone());

                format_ident!("{}", nested_name).into_token_stream()
            }
            Value::Null => quote!(Option<::serde_json::Value>),
        };

        // Handle Serde alias configuration
        //
        // this is usefull when serializing, and when also specifing the @camel|pascal|snake flags
        //
        // if you have a json that's formatted like so
        //
        // ```json
        // {
        //   "name": "Abdullah",
        //   "jobs_list": ["Cybersecurity"]
        // }
        // ```
        //
        // the keys are written in snake_case,
        // which means if you have a sruct that you want to deserialize to which has an attribte that looks like this
        //
        // ```rust
        // #[derive(Deserialize, Serialize)]
        // #[serde(rename_all = "camelCase")]
        // struct User {
        //   name: String,
        //   jobs_list: Vec<String>
        // }
        // ```
        //
        // this will only deserialize if you give it a camelCase keys, not snake_case
        //
        // this is where the `#[serde(alias = "jobs_list")]` comes in, it allows you to have both,
        // so you can deserialize with camelCase and snake_case
        let field = quote! {
            #[serde(alias = #key)]
            pub #field_name: #field_type
        };

        fields.push(field);
    }

    // Generate the main struct with optional rename strategy
    let main_struct = quote! {
        #[derive(::serde::Deserialize, ::serde::Serialize, ::std::clone::Clone, ::std::fmt::Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct #base_name {
            #(#fields),*
        }
    };

    (main_struct, all_structs)
}

/// Infers the element type for an array of JSON values.
///
/// # Parameters
/// - `arr`: A slice of JSON values
///
/// # Returns
/// A tuple containing:
/// 1. The inferred element type as a token stream
/// 2. Any additional generated structs (currently unused)
fn infer_array_type(arr: &[Value]) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    // Handle empty array
    if arr.is_empty() {
        return (quote!(::serde_json::Value), Vec::new());
    }

    // Infer type based on first element
    match &arr[0] {
        Value::String(_) => (quote!(String), Vec::new()),
        Value::Number(_) => (quote!(f64), Vec::new()),
        Value::Bool(_) => (quote!(bool), Vec::new()),
        _ => (quote!(::serde_json::Value), Vec::new()),
    }
}

fn get_in_type(schema: &JsonSchema) -> Value {
    match schema.ty {
        JsonSchemaTypes::String => Value::String(String::new()),
        JsonSchemaTypes::None => Value::Null,
        JsonSchemaTypes::Number => Value::Number(Number::from(0)),
        JsonSchemaTypes::Array => {
            if let Some(items) = &schema.items {
                Value::Array(vec![get_in_type(items)])
            } else {
                Value::Array(Vec::new())
            }
        }
        JsonSchemaTypes::Object => Value::Object(Map::new()),
    }
}

pub fn convert_raw_schema_to_json_sample(schema: &JsonSchema, title: &String) -> Value {
    let mut json = Map::new();

    if !matches!(schema.ty, JsonSchemaTypes::Object) {
        return get_in_type(schema);
    }

    for (key, property) in schema.properties.as_ref().unwrap() {
        json.insert(
            key.to_owned(),
            convert_raw_schema_to_json_sample(&property, title),
        );
    }

    json.into()
}
