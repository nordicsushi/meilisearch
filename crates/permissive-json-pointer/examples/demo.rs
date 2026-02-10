use permissive_json_pointer::select_values;
use serde_json::*;

fn main() {
    println!("=== Example 1: Simple Selection ===");
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
    println!("{:#?}", results);

    println!("\n=== Example 2: Nested Selection ===");
    let results: Value = select_values(value, vec!["race.name"]).into();
    println!("{:#?}", results);

    println!("\n=== Example 3: Permissive Matching ===");
    let json = json!({
        "pet.dog.name": "jean",
        "pet.dog": {
            "name": "bob"
        },
        "pet": {
            "dog.name": "michel",
            "dog": {
                "name": "milan"
            }
        }
    });
    let value = json.as_object().unwrap();
    let results: Value = select_values(value, vec!["pet.dog.name"]).into();
    println!("{:#?}", results);

    println!("\n=== Example 4: Array Selection ===");
    let json = json!({
        "pets": [
            { "name": "Max", "age": 3 },
            { "name": "Luna", "age": 5 },
            { "type": "bird" }
        ]
    });
    let value = json.as_object().unwrap();
    let results: Value = select_values(value, vec!["pets.name"]).into();
    println!("{:#?}", results);
}
