use permissive_json_pointer::select_values;
use serde_json::*;

#[allow(unused)]
fn contained_in(selector: &str, key: &str) -> bool {
    selector.starts_with(key)
        && selector[key.len()..].chars().next().map(|c| c == '.').unwrap_or(true)
}
fn main() {
    // {
    //     "dog.name": "jean",
    //     "dog": {
    //         "name": "bob",
    //         "age": 6
    //     }
    // }

    // let if_contained = contained_in("animaux.ch", "animaux.chien");
    // println!("{}", if_contained);

    println!("=== Example 1: Simple Selection ===");
    let json = json!({
        "name": "peanut",
        "age": 8,
        "race": [
            {
                "name": "bernese mountain",
                "avg_age": 12,
                "size": "80cm"
            },
            {
                "name": "cat",
                "avg_age": 5,
                "size": "100cm"
            }
        ]
    });
    let value = json.as_object().unwrap();
    let results: Value = select_values(value, vec!["name", "race.name", "race.avg_age"]).into();
    println!("results: {:#?}", results);
}
