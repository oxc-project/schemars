use std::collections::BTreeMap;

use schemars::JsonSchema;

mod util;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Flat {
    f: f32,
    b: bool,
    s: String,
    #[serde(default)]
    os: String,
    v: Vec<i32>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "Flat")]
struct Deep1 {
    f: f32,
    #[schemars(flatten)]
    deep2: Deep2,
    v: Vec<i32>,
}

#[allow(clippy::option_option, dead_code)]
#[derive(JsonSchema)]
struct Deep2 {
    b: bool,
    #[serde(flatten)]
    deep3: Deep3,
    #[serde(flatten)]
    deep4: Box<Option<Option<Box<Deep4>>>>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Deep3 {
    s: &'static str,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Deep4 {
    #[serde(default)]
    os: &'static str,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct FlattenedMap {
    /// Additional flattened values.
    #[serde(flatten)]
    extra: BTreeMap<String, Vec<String>>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
struct DenyUnknownFields {
    value: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct FlattenedDenyUnknownFields {
    #[serde(flatten)]
    extra: DenyUnknownFields,
}

#[test]
fn test_flat_schema() -> TestResult {
    test_default_generated_schema::<Flat>("flatten")
}

#[test]
fn test_flattened_schema() -> TestResult {
    test_default_generated_schema::<Deep1>("flatten")
}

#[test]
fn test_flattened_map_schema() -> TestResult {
    test_default_generated_schema::<FlattenedMap>("flatten_map")
}

#[test]
fn test_flattened_deny_unknown_fields_schema() -> TestResult {
    test_default_generated_schema::<FlattenedDenyUnknownFields>("flatten_deny_unknown_fields")
}
