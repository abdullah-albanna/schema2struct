#[macro_export]
macro_rules! update_schema_fields {
    ($schema:expr, $value:expr, $key_span:expr, $value_span:expr, [
        $($field:ident),* $(,)?
    ]) => {
        $(
            if $schema.$field.is_none() && $value.$field.is_some() {
                $schema.$field = $value.$field;

                paste::paste! {
                    $schema.[<$field _span>] = Some(($key_span, $value_span));
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! create_json_schema_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                pub $field_name:ident : $field_type:ty,
                $($rename:meta)*  // Optional renaming parameter
            )*
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            // Regular fields
            $(
                $(#[$field_meta])*
                $(
                    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
                )?
                $(
                    #[serde(rename = $($rename)*)] // If rename is present, it will apply the rename
                )?
                pub $field_name: $field_type,

                // Span fields for each regular field
                #[serde(skip_serializing_if = "::std::option::Option::is_none")]
                pub $field_name_span: Option<(proc_macro2::Span, proc_macro2::Span)>,
            )*
        }
    };
}
