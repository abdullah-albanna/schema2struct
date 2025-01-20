extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, Type};

#[proc_macro]
pub fn create_json_schema_struct(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Struct name
    let struct_name = &input.ident;

    // Generate span fields for each struct field
    let mut expanded_fields = vec![];

    if let syn::Data::Struct(s) = &input.data {
        for field in &s.fields {
            let field_name = field.ident.as_ref().expect("Field must have a name");
            let field_type = &field.ty;

            // Check for a rename attribute
            let rename = field.attrs.iter().find_map(|attr| {
                if attr.path().is_ident("serde") {
                    match &attr.meta {
                        Meta::List(list) => {
                            list.tokens
                                .clone()
                                .into_iter()
                                .find_map(|token| match token {
                                    proc_macro2::TokenTree::Literal(lit) => {
                                        let s = lit.to_string().trim_matches('"').to_string();
                                        Some(s)
                                    }
                                    _ => None,
                                })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            });

            // Create span field name
            let field_name_span =
                syn::Ident::new(&format!("{}_span", field_name), field_name.span());

            // Check if the type is an Option
            let is_option = match field_type {
                Type::Path(type_path) => type_path
                    .path
                    .segments
                    .first()
                    .map_or(false, |segment| segment.ident == "Option"),
                _ => false,
            };

            let rename_attr = if let Some(r) = rename {
                quote! { #[serde(rename = #r)] }
            } else {
                quote! {}
            };

            let field_attrs = if is_option {
                quote! {
                    #rename_attr
                    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
                }
            } else {
                rename_attr
            };

            expanded_fields.push(quote! {
                #field_attrs
                pub #field_name: #field_type,

                #[serde(skip)]
                pub #field_name_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
            });
        }
    }

    // Generate the final struct code with span fields
    //
    // with special fields
    let expanded = quote! {
        #[derive(Clone, Debug, Default, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #struct_name {
            #(#expanded_fields)*

            #[serde(skip)]
            pub depth: usize,

            #[serde(skip)]
            pub current_key_span: Option<proc_macro2::Span>,

            #[serde(skip)]
            pub struct_name: Option<String>,

            #[serde(skip)]
            pub struct_name_span: Option<(proc_macro2::Span, proc_macro2::Span)>
        }
    };

    // Convert the quote into TokenStream and return
    TokenStream::from(expanded)
}
