use crate::JsonSchema;
use crate::r#gen::SchemaGenerator;
use crate::schema::*;
use std::borrow::Cow;

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                super::concat_name("Array_of_", T::schema_name())
            }

            fn schema_id() -> Cow<'static, str> {
                super::wrap_id("[", T::schema_id(), "]")
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                array_schema(generator.subschema_for::<T>(), false)
            }
        }
    };
}

macro_rules! set_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                super::concat_name("Set_of_", T::schema_name())
            }

            fn schema_id() -> Cow<'static, str> {
                super::wrap_id("Set<", T::schema_id(), ">")
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                array_schema(generator.subschema_for::<T>(), true)
            }
        }
    };
}

/// Non-generic body shared by every sequence and set `json_schema`, so it is compiled once
/// rather than once per element type in every downstream crate.
fn array_schema(items: Schema, unique_items: bool) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            unique_items: if unique_items { Some(true) } else { None },
            items: Some(items.into()),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

seq_impl!(<T> JsonSchema for std::collections::BinaryHeap<T>);
seq_impl!(<T> JsonSchema for std::collections::LinkedList<T>);
seq_impl!(<T> JsonSchema for [T]);
seq_impl!(<T> JsonSchema for Vec<T>);
seq_impl!(<T> JsonSchema for std::collections::VecDeque<T>);

set_impl!(<T> JsonSchema for std::collections::BTreeSet<T>);
set_impl!(<T, H> JsonSchema for std::collections::HashSet<T, H>);
