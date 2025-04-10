use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    name: String,
    age: u8,
    skills: Option<Vec<String>>,
}
fn main() -> Result<()> {
    let json = r#"{"name":"john","age":43}"#;
    let person: Person = serde_json::from_str(json)?;
    println!("person: {:?}", person);

    let person = Person {
        name: "john".to_string(),
        age: 43,
        skills: Some(vec!["Skill 1".to_string()]),
    };

    let json = serde_json::to_string(&person)?;
    println!("json: {}", json);

    Ok(())
}
