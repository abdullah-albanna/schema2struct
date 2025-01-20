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
macro_rules! json_schema_struct {
    // Macro entry point with optional struct-level attributes
    (
        $(#[$meta:meta])*
        $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident $(: $field_type:ty)?
            ),* $(,)?
        }
    ) => {
        // Collect struct-level attributes
        $(#[$meta])*
        #[derive(
            Clone,
            Debug,
            Default,
            Serialize,
            Deserialize
        )]
        #[serde(rename_all = "camelCase")]
        pub struct $struct_name {
            // Generate original fields and their span fields
            $(
                $crate::__json_schema_struct_field!(
                    $(#[$field_meta])*
                    $field_name $(: $field_type)?
                )
            )*

            // Metadata fields with configurable attributes
            #[serde(skip)]
            pub depth: usize,

            #[serde(skip)]
            pub current_key_span: Option<proc_macro2::Span>,

            #[serde(skip)]
            pub struct_name: Option<String>,

            #[serde(skip)]
            pub struct_name_span: Option<(proc_macro2::Span, proc_macro2::Span)>,

        }

    };
}

// Internal macro to handle different field types with advanced configuration
#[macro_export]
macro_rules! __json_schema_struct_field {
    // Field with multiple attributes
    (
        $(#[serde($serde_attr:meta)])*
        $(#[schema($schema_attr:ident)])*
        $name:ident: $type:ty
    ) => {
        // Handle Serde attributes
        $(#[serde($serde_attr)])*

        // Skip serialization for schema-skipped fields
        $(
            #[cfg(if $schema_attr != skip)]
            #[serde(skip_serializing_if = "Option::is_none")]
        )?

        pub $name: $type,

        paste::paste! {
            #[serde(skip)]
            [<$name _span>]: Option<(proc_macro2::Span, proc_macro2::Span)>;
        }

    };

    // Optional field with multiple attributes
    (
        $(#[serde($serde_attr:meta)])*
        $(#[schema($schema_attr:ident)])*
        $name:ident: Option<$type:ty>
    ) => {
        // Handle Serde attributes
        $(#[serde($serde_attr)])*

        // Skip serialization for schema-skipped fields
        $(
            #[cfg(if $schema_attr != skip)]
            #[serde(skip_serializing_if = "Option::is_none")]
        )?

        pub $name: Option<$type>,

        paste::paste! {
            #[serde(skip)]
            [<$name _span>]: Option<(proc_macro2::Span, proc_macro2::Span)>;
        }

    };

    // Default case: simple field without attributes
    ($name:ident: $type:ty) => {
        pub $name: $type,

        paste::paste! {
            #[serde(skip)]
            [<$name _span>]: Option<(proc_macro2::Span, proc_macro2::Span)>;
        }

    };

    // Default case: optional field without attributes
    ($name:ident: Option<$type:ty>) => {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub $name: Option<$type>,


        paste::paste! {
            #[serde(skip)]
            [<$name _span>]: Option<(proc_macro2::Span, proc_macro2::Span)>;
        }

    };
}
