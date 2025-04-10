use anyhow::Result;
use axum::extract::State;
use axum::routing::{get, patch};
use axum::Json;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

// #[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u8,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("User", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("age", &self.age)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("User", &["name", "age"], UserVisitor)
    }
}

struct UserVisitor;
impl<'de> Visitor<'de> for UserVisitor {
    type Value = User;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct User")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let name = seq
            .next_element::<&str>()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let age = seq
            .next_element::<u8>()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        Ok(Self::Value {
            name: name.to_string(),
            age,
        })
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut age = None;
        let mut map = map;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "name" => {
                    if name.is_some() {
                        return Err(serde::de::Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?);
                }
                "age" => {
                    if age.is_some() {
                        return Err(serde::de::Error::duplicate_field("age"));
                    }
                    age = Some(map.next_value()?);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }
        let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
        let age = age.ok_or_else(|| serde::de::Error::missing_field("age"))?;
        Ok(Self::Value {
            name: name.to_string(),
            age,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserUpdate {
    name: Option<String>,
    age: Option<u8>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = fmt::Layer::new().pretty().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(console).init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    let user = User {
        name: "zhangsan".to_string(),
        age: 18,
    };
    let router = axum::Router::new()
        .route("/", get(handle_user))
        .route("/", patch(update_user))
        .with_state(Arc::new(Mutex::new(user)));
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

async fn handle_user(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    // let user = User {
    //     name: "zhangsan".to_string(),
    //     age: 18,
    // };
    (*user.lock().unwrap()).clone().into()
}

async fn update_user(
    State(user): State<Arc<Mutex<User>>>,
    Json(new_user): Json<UserUpdate>,
) -> Json<User> {
    info!("user: {:?}", new_user);
    let mut user = user.lock().unwrap();
    if let Some(name) = new_user.name {
        user.name = name;
    }
    if let Some(age) = new_user.age {
        user.age = age;
    }
    user.clone().into()
}
