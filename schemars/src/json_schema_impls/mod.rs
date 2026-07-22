macro_rules! no_ref_schema {
    () => {
        fn is_referenceable() -> bool {
            false
        }
    };
}

macro_rules! forward_impl {
    (($($impl:tt)+) => $target:ty) => {
        impl $($impl)+ {
            fn is_referenceable() -> bool {
                <$target>::is_referenceable()
            }

            fn schema_name() -> String {
                <$target>::schema_name()
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                <$target>::schema_id()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                <$target>::json_schema(generator)
            }

            fn _schemars_private_non_optional_json_schema(generator: &mut SchemaGenerator) -> Schema {
                <$target>::_schemars_private_non_optional_json_schema(generator)
            }

            fn _schemars_private_is_option() -> bool {
                <$target>::_schemars_private_is_option()
            }
        }
    };
    ($ty:ty => $target:ty) => {
        forward_impl!((JsonSchema for $ty) => $target);
    };
}

mod array;
#[cfg(target_has_atomic = "8")]
mod atomic;
mod core;
mod ffi;
#[cfg(feature = "indexmap2")]
mod indexmap2;
mod maps;
mod nonzero_signed;
mod nonzero_unsigned;
mod primitives;
#[cfg(feature = "regex")]
mod regex;
mod sequences;
mod serdejson;
mod time;
mod tuple;
mod wrapper;
