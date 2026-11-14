use crate::r#gen::SchemaGenerator;
use crate::schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SubschemaValidation};
use crate::{JsonSchema, Map, Set};
use serde::Serialize;
use serde_json::Value;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    generator: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(generator);

    if T::_schemars_private_is_option() && !required {
        if let Schema::Object(SchemaObject { object: Some(ref mut object_validation), .. }) = schema
        {
            object_validation.required.clear();
        }
    }

    schema
}

/// Hack to simulate specialization:
/// `MaybeSerializeWrapper(x).maybe_to_value()` will resolve to either
/// - The inherent method `MaybeSerializeWrapper::maybe_to_value(...)` if x is `Option<impl Serialize>`
/// - The trait method `NoSerialize::maybe_to_value(...)` from the blanket impl otherwise
///
/// The `Option` is unwrapped here rather than in derive-generated code so that each field with a
/// default doesn't instantiate its own closure and `Option::and_then` in the downstream crate.
#[doc(hidden)]
#[macro_export]
macro_rules! _schemars_maybe_to_value {
    ($expression:expr) => {{
        #[allow(unused_imports)]
        use $crate::_private::{MaybeSerializeWrapper, NoSerialize as _};

        MaybeSerializeWrapper($expression).maybe_to_value()
    }};
}

pub struct MaybeSerializeWrapper<T>(pub Option<T>);

pub trait NoSerialize: Sized {
    fn maybe_to_value(self) -> Option<Value> {
        None
    }
}

impl<T> NoSerialize for T {}

impl<T: Serialize> MaybeSerializeWrapper<T> {
    pub fn maybe_to_value(self) -> Option<Value> {
        self.0.and_then(|value| serde_json::value::to_value(value).ok())
    }
}

/// Create a schema for a unit enum
pub fn new_unit_enum(variant: &str) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(vec![variant.into()]),
        ..SchemaObject::default()
    })
}

/// Create a schema for an externally tagged enum
pub fn new_externally_tagged_enum(variant: &str, sub_schema: Schema) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            properties: {
                let mut props = Map::new();
                props.insert(variant.to_owned(), sub_schema);
                props
            },
            required: {
                let mut required = Set::new();
                required.insert(variant.to_owned());
                required
            },
            // Externally tagged variants must prohibit additional
            // properties irrespective of the disposition of
            // `deny_unknown_fields`. If additional properties were allowed
            // one could easily construct an object that validated against
            // multiple variants since here it's the properties rather than
            // the values of a property that distingish between variants.
            additional_properties: Some(Box::new(false.into())),
            ..Default::default()
        })),
        ..SchemaObject::default()
    })
}

/// Create a schema for an internally tagged enum
pub fn new_internally_tagged_enum(
    tag_name: &str,
    variant: &str,
    deny_unknown_fields: bool,
) -> Schema {
    let tag_schema = Schema::Object(SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(vec![variant.into()]),
        ..Default::default()
    });
    Schema::Object(SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            properties: {
                let mut props = Map::new();
                props.insert(tag_name.to_owned(), tag_schema);
                props
            },
            required: {
                let mut required = Set::new();
                required.insert(tag_name.to_owned());
                required
            },
            additional_properties: deny_unknown_fields.then(|| Box::new(false.into())),
            ..Default::default()
        })),
        ..SchemaObject::default()
    })
}

pub fn insert_object_property(
    obj: &mut ObjectValidation,
    key: &str,
    has_default: bool,
    required: bool,
    is_option: bool,
    schema: Schema,
) {
    obj.properties.insert(key.to_owned(), schema);
    if !has_default && (required || !is_option) {
        obj.required.insert(key.to_owned());
    }
}

/// Create a schema for a unit-only enum, e.g. `{"type": "string", "enum": ["a", "b"]}`.
pub fn new_string_enum(variants: &[&str]) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(variants.iter().map(|&v| v.into()).collect()),
        ..SchemaObject::default()
    })
}

/// Create a schema whose variants are combined with `oneOf` (if `unique`) or `anyOf`.
pub fn new_subschemas(unique: bool, schemas: Vec<Schema>) -> Schema {
    let subschemas = if unique {
        SubschemaValidation { one_of: Some(schemas), ..Default::default() }
    } else {
        SubschemaValidation { any_of: Some(schemas), ..Default::default() }
    };
    Schema::Object(SchemaObject { subschemas: Some(Box::new(subschemas)), ..Default::default() })
}

/// Push the schema of a unit enum variant that has its own metadata (e.g. a doc comment).
pub fn push_unit_variant(schemas: &mut Vec<Schema>, variant: &str, meta: &metadata::Metadata) {
    schemas.push(metadata::apply(new_unit_enum(variant), meta, None));
}

/// Create the (initially empty) object schema for a struct with named fields.
pub fn new_object_schema(deny_unknown_fields: bool) -> SchemaObject {
    let mut schema_object =
        SchemaObject { instance_type: Some(InstanceType::Object.into()), ..Default::default() };
    if deny_unknown_fields {
        schema_object.object().additional_properties = Some(Box::new(false.into()));
    }
    schema_object
}

/// Static per-property data for [`insert_property`]. Every field is a constant, so derive-generated
/// code passes it as a promoted `&'static` reference instead of building it at runtime.
pub struct PropertyMeta {
    pub name: &'static str,
    pub has_default: bool,
    pub required: bool,
    pub metadata: metadata::Metadata,
}

/// Apply a property's metadata and insert it into `obj`.
///
/// This does in one non-generic call what derive-generated code used to do with a chain of
/// `add_*` calls per field, so the `Schema` value is never moved around in the downstream crate.
pub fn insert_property(
    obj: &mut ObjectValidation,
    schema: Schema,
    is_option: bool,
    meta: &PropertyMeta,
    default: Option<Value>,
) {
    let schema = metadata::apply(schema, &meta.metadata, default);
    insert_object_property(obj, meta.name, meta.has_default, meta.required, is_option, schema);
}

/// [`insert_property`], generating the schema with `schema_fn`
/// (e.g. `SchemaGenerator::subschema_for::<T>`).
pub fn insert_property_with(
    obj: &mut ObjectValidation,
    generator: &mut SchemaGenerator,
    schema_fn: fn(&mut SchemaGenerator) -> Schema,
    is_option: bool,
    meta: &PropertyMeta,
    default: Option<Value>,
) {
    insert_property(obj, schema_fn(generator), is_option, meta, default);
}

pub mod metadata {
    use crate::Schema;
    use serde_json::Value;

    /// Static metadata for [`apply`]; see [`PropertyMeta`](super::PropertyMeta).
    pub struct Metadata {
        pub title: Option<&'static str>,
        pub description: Option<&'static str>,
        pub deprecated: bool,
        pub read_only: bool,
        pub write_only: bool,
    }

    /// Equivalent to the chain of `add_title`, `add_description`, `add_deprecated`,
    /// `add_read_only`, `add_write_only` and `add_default`, in a single call.
    pub fn apply(schema: Schema, meta: &Metadata, default: Option<Value>) -> Schema {
        let has_metadata = meta.title.is_some()
            || meta.description.is_some()
            || meta.deprecated
            || meta.read_only
            || meta.write_only
            || default.is_some();
        if !has_metadata {
            return schema;
        }

        let mut schema_obj = schema.into_object();
        let m = schema_obj.metadata();
        if let Some(title) = meta.title {
            m.title = Some(title.to_owned());
        }
        if let Some(description) = meta.description {
            m.description = Some(description.to_owned());
        }
        m.deprecated |= meta.deprecated;
        m.read_only |= meta.read_only;
        m.write_only |= meta.write_only;
        if default.is_some() {
            m.default = default;
        }
        Schema::Object(schema_obj)
    }

    macro_rules! add_metadata_fn {
        ($method:ident, $name:ident, $ty:ty) => {
            pub fn $method(schema: Schema, $name: impl Into<$ty>) -> Schema {
                let value = $name.into();
                if value == <$ty>::default() {
                    schema
                } else {
                    let mut schema_obj = schema.into_object();
                    schema_obj.metadata().$name = value.into();
                    Schema::Object(schema_obj)
                }
            }
        };
    }

    add_metadata_fn!(add_description, description, String);
    add_metadata_fn!(add_id, id, String);
    add_metadata_fn!(add_title, title, String);
    add_metadata_fn!(add_deprecated, deprecated, bool);
    add_metadata_fn!(add_read_only, read_only, bool);
    add_metadata_fn!(add_write_only, write_only, bool);
    add_metadata_fn!(add_default, default, Option<Value>);

    pub fn add_examples<I: IntoIterator<Item = Value>>(schema: Schema, examples: I) -> Schema {
        let mut schema_obj = schema.into_object();
        schema_obj.metadata().examples.extend(examples);
        Schema::Object(schema_obj)
    }
}
