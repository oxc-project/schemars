use schemars::{JsonSchema_repr, schema_for};

#[derive(JsonSchema_repr)]
#[repr(u8)]
enum SmallPrime {
    Two = 2,
    Three = 3,
    Five = 5,
    Seven = 7,
}

fn main() {
    let schema = schema_for!(SmallPrime);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
