use crate::JsonSchema;
use crate::r#gen::SchemaGenerator;
use crate::schema::*;
use std::borrow::Cow;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                super::concat_name("Map_of_", V::schema_name())
            }

            fn schema_id() -> Cow<'static, str> {
                super::wrap_id("Map<", V::schema_id(), ">")
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                map_schema(generator.subschema_for::<V>())
            }
        }
    };
}

/// Non-generic body shared by every map `json_schema`, so it is compiled once
/// rather than once per value type in every downstream crate.
fn map_schema(subschema: Schema) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            additional_properties: Some(Box::new(subschema)),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

map_impl!(<K, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K, V, H> JsonSchema for std::collections::HashMap<K, V, H>);
