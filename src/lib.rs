/// # schema2struct: JSON Schema to Rust Struct Generator
///
/// A powerful procedural macro for generating Rust structs from JSON Schema definitions
/// with compile-time type safety
///
/// ## Features
/// - Automatic struct generation from JSON Schema
/// - Serde integration for easy serialization and deserialization
/// - Compile-time type checking to catch errors early
/// - Flexible schema parsing with support for nested structures
/// - Validation of schema constraints such as required fields, type restrictions, and more
///
/// ## Supported Schema Validations
/// - Type constraints
/// - Length restrictions
/// - Numeric ranges
/// - Required fields
/// - Array constraints
///
/// ## Avaliable keywords
///    - type => [ object,  string, array, number]
///    - title
///    - required
///    - description
///    - items
///    - properties
///    - default
///    - examples
///    - enum
///    - const
///    - min_length
///    - max_length
///    - pattern
///    - format => [date, time, datetime, email, hostname, ipv4, ipv6, uri ]
///    - minimum
///    - maximum
///    - max_items
///    - min_items
///    - unique_items
///    - contains
///    - struct
///
mod checkers;
mod generator;
mod models;
mod parsers;
mod try_from_impls;

use generator::{generate_structs, JsonMacroInput};
use models::JsonSchema;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};

/// converts json schema into a useable struct as a response from the schema
///
/// # Example
/// ```rust
/// schema2struct! {
///     struct: User,
///     type: object,
///     properties: {
///         "name": { type: string },
///         "age": { type: number, minimum: 0 }
///     },
///     required: ["name", "age"]
/// }
///
/// fn bind_it() {
///     let json = &*USER_JSON_VALUE;
///     let response = // request to an api with the generated json value
///
///     let hard_bind_response: User = serde_json::from_str(response.text).unwrap();
///
///     // now you can access the fileds using dot notation
///     println!("{}", hard_bind_response.name);
/// }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn schema2struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = syn::parse_macro_input!(input as JsonSchema);

    if let Some(struct_name) = &schema.struct_name {
        let title = format_ident!("{}", struct_name);

        let json = schema.to_json_sample();

        let json_struct = &JsonMacroInput {
            struct_name: title.clone(),
            content: json,
        };

        let mut output = proc_macro2::TokenStream::new();

        let (main_struct, other_nested_struct) = generate_structs(json_struct, &title);

        output.extend(get_serde_const(&schema, &title));

        output.extend(quote! {
            #main_struct
            #(#other_nested_struct)*
        });

        return output.into();
    }

    proc_macro::TokenStream::new()
}

// gets the whole schema as json and save it to a const value
fn get_serde_const(schema: &JsonSchema, title: &syn::Ident) -> proc_macro2::TokenStream {
    let serde_value_str = serde_json::to_string(schema).unwrap_or_default();

    // Generate a constant name based on struct name
    let const_json_ident = format_ident!("{}_{}", title.to_string().to_uppercase(), "JSON_VALUE");

    quote! {
        pub static #const_json_ident: ::std::sync::LazyLock<::serde_json::Value> =
            ::std::sync::LazyLock::new(||
                ::serde_json::from_str(#serde_value_str)
                    .expect("Couldn't convert the text into valid json")
            );
    }
}
