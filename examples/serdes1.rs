use anyhow::Result;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    #[serde(rename = "private_age")]
    age: u8,
    dob: DateTime<Utc>,
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")] // 全部使用snake_case
enum WorkState {
    Working(String),
    OnLeave(String),
    Terminated,
}

fn main() -> Result<()> {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        dob: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
        state: WorkState::Working("Rust Engineer".to_string()),
        data: vec![1, 2, 3, 4, 5],
    };

    let json = serde_json::to_string(&user)?;
    println!("{}", json);
    Ok(())
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = BASE64_URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}
