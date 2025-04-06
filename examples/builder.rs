use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;

#[allow(dead_code)]
#[derive(Debug, Builder)]
#[builder(build_fn(name = "_build"))]
struct User {
    #[builder(setter(into), default = String::from(""))]
    name: String,
    #[builder(default = 18, setter(skip))] // 可以通过生日来计算
    age: u32,
    #[builder(setter(into, strip_option), default = "None")]
    email: Option<String>,
    #[builder(default = vec![], setter(each(name="skill", into)))]
    skills: Vec<String>,
    #[builder(setter(custom))]
    dob: Option<DateTime<Utc>>,
}

impl User {
    pub fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn build(&mut self) -> Result<User> {
        let mut user = self._build()?;
        user.age = (Utc::now().year() - user.dob.unwrap_or(Utc::now()).year()) as u32;
        Ok(user)
    }
    pub fn bob(&mut self, val: &str) -> &mut Self {
        self.dob = Some(
            DateTime::parse_from_rfc3339(val)
                .map(|dt| dt.with_timezone(&Utc))
                .ok(),
        );
        self
    }
}

fn main() -> Result<()> {
    // let user = UserBuilder::default()
    //     .name("zhangsan")
    //     .email("johnsonsmile@163.com")
    //     .build()?;
    // println!("{:#?}", user);
    // let user = UserBuilder::default().build()?;
    // println!("{:#?}", user);
    // let user = UserBuilder::default().build()?;
    // println!("{:#?}", user);
    let user = User::build()
        .skill("piano")
        .skill("chess")
        .bob("2005-01-01T00:00:00Z")
        .build()?;
    println!("{:#?}", user);
    Ok(())
}
