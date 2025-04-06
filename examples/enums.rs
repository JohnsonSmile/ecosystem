use anyhow::Result;
use strum::{
    Display, EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator,
    IntoStaticStr, VariantNames,
};

#[allow(unused)]
#[derive(
    Debug,
    EnumString,
    EnumCount,
    EnumDiscriminants,
    EnumIter,
    EnumIs,
    IntoStaticStr,
    // VariantArray,
    VariantNames,
)]
enum MyEnum {
    A,
    B(String),
    C,
    D,
}

#[allow(unused)]
#[derive(Display)]
enum Color {
    #[strum(serialize = "red block")] // to_string优先级高于serialize
    Red,
    Green,
    Blue,
    Yellow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: i32,
    },
}
fn main() -> Result<()> {
    MyEnum::VARIANTS.iter().for_each(|v| println!("{:?}", v));
    MyEnum::iter().for_each(|v| println!("{:?}", v));

    let e = MyEnum::B("Hello".to_string());
    println!("{:?}", e.is_b());

    let red = Color::Red;
    let green = Color::Green;
    let yellow = Color::Yellow;
    let purple = Color::Purple { sat: 30 };
    println!(
        "red: {}, green: {}, yellow: {}, purple: {}",
        red, green, yellow, purple
    );

    println!("count: {}", MyEnum::COUNT);

    Ok(())
}
