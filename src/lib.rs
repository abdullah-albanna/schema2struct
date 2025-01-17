mod checkers;
mod generator;
mod models;
mod parsers;
mod try_from_impls;

use generator::{generate_structs, JsonMacroInput};
use models::JsonSchema;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};

#[proc_macro_error]
#[proc_macro]
pub fn jsonschema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = syn::parse_macro_input!(input as JsonSchema);

    if let Some(struct_name) = &schema.struct_name {
        let title = format_ident!("{}", struct_name);

        let json = schema.to_json_sample();

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

        return output.into();
    }

    proc_macro::TokenStream::new()
}

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
