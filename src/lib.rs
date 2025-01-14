mod checkers;
mod models;
mod parsers;
mod try_from_impls;

use models::JsonSchema;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn jsonschema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = syn::parse_macro_input!(input as JsonSchema);
    println!("{:#?}", schema);

    proc_macro::TokenStream::new()
}
