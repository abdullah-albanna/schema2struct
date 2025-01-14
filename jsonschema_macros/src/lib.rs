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
