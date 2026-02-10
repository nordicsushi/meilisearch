use permissive_json_pointer::select_values;
use serde_json::*;

fn main() {
    let json = json!({
        "name": "peanut",
        "age": 8,
        "race": {
            "name": "bernese mountain",
            "avg_age": 12,
            "size": "80cm",
        }
    });
    let value = json.as_object().unwrap();

    let results: Value = select_values(value, vec!["name", "age"]).into();
    println!("{:#?}", results)
}
