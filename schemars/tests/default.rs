mod util;
use schemars::JsonSchema;
use util::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

fn ten_and_true() -> MyStruct2 {
    MyStruct2 { my_int: 10, my_bool: true }
}

fn six() -> i32 {
    6
}

fn custom_serialize<S>(value: &MyStruct2, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ser.collect_str(&format_args!("i:{} b:{}", value.my_int, value.my_bool))
}

#[allow(dead_code)]
#[derive(Default, JsonSchema)]
#[serde(default)]
struct MyStruct {
    my_int: i32,
    my_bool: bool,
    my_optional_string: Option<String>,
    #[serde(serialize_with = "custom_serialize")]
    my_struct2: MyStruct2,
    #[serde(serialize_with = "custom_serialize", skip_serializing_if = "is_default")]
    my_struct2_default_skipped: MyStruct2,
    not_serialize: NotSerialize,
}

#[allow(dead_code)]
#[derive(Default, JsonSchema, PartialEq)]
#[serde(default = "ten_and_true")]
struct MyStruct2 {
    #[serde(default = "six")]
    my_int: i32,
    my_bool: bool,
}

#[derive(Default, JsonSchema)]
struct NotSerialize;

#[test]
fn schema_default_values() -> TestResult {
    test_default_generated_schema::<MyStruct>("default")
}

fn ten_and_false() -> MyStruct2 {
    MyStruct2 { my_int: 10, my_bool: false }
}

/// Field-level defaults (not taken from the container default) combined with
/// `skip_serializing_if` and `serialize_with`.
#[allow(dead_code)]
#[derive(JsonSchema)]
struct FieldDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skipped_none: Option<String>,
    #[serde(default = "six", skip_serializing_if = "is_default")]
    kept_six: i32,
    #[serde(default = "ten_and_false", serialize_with = "custom_serialize")]
    custom: MyStruct2,
    #[serde(
        default = "ten_and_false",
        serialize_with = "custom_serialize",
        skip_serializing_if = "is_default"
    )]
    custom_skip_if: MyStruct2,
}

#[test]
fn schema_field_default_values() {
    let schema = serde_json::to_value(schemars::schema_for!(FieldDefaults)).unwrap();
    let props = &schema["properties"];
    assert_eq!(props["skipped_none"].get("default"), None);
    assert_eq!(props["kept_six"]["default"], 6);
    assert_eq!(props["custom"]["default"], "i:10 b:false");
    assert_eq!(props["custom_skip_if"]["default"], "i:10 b:false");
}
