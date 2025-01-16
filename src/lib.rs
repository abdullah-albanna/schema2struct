mod checkers;
mod generator;
mod models;
mod parsers;
mod try_from_impls;

use generator::{convert_raw_schema_to_json_sample, generate_structs, JsonMacroInput};
use models::JsonSchema;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};

#[proc_macro_error]
#[proc_macro]
pub fn jsonschema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = syn::parse_macro_input!(input as JsonSchema);
    let title = format_ident!("{}", &schema.title.as_ref().unwrap());

    let json = convert_raw_schema_to_json_sample(&schema, &schema.title.clone().unwrap());

    let json_struct = &JsonMacroInput {
        struct_name: title.clone(),
        content: json,
    };

    let mut output = proc_macro2::TokenStream::new();

    let (main, al) = generate_structs(json_struct, &title);

    output.extend(get_serde_const(&schema, &title));

    output.extend(quote! {
        #main
        #(#al)*
    });

    output.into()
}

fn get_serde_const(schema: &JsonSchema, title: &syn::Ident) -> proc_macro2::TokenStream {
    let serde_value_str = serde_json::to_string(schema).unwrap_or_default();

    // Generate a constant name based on struct name
    let const_json_ident = format_ident!("{}_{}", title.to_string().to_uppercase(), "JSON_VALUE");

    quote! {
        static #const_json_ident: ::std::sync::LazyLock<::serde_json::Value> =
            ::std::sync::LazyLock::new(||
                ::serde_json::from_str(#serde_value_str)
                    .expect("Couldn't convert the text into valid json")
            );
    }
}
