use anyhow::Result;
use derive_more::{Add, Display, From, Into};

#[derive(PartialEq, From, Add, Display, Debug)]
struct MyInt(i32);

#[derive(PartialEq, From, Into, Display)]
#[display("{{ x: {x}, y: {y} }}")]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, From, Add, Display)]
enum MyEnum {
    #[display("int: {}", _0)]
    Int(i32),
    Uint(u32),
    #[display("nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int: MyInt = 10.into();
    println!("{:?}", my_int);
    let v = my_int + MyInt(2);
    println!("{:?}", v);
    let p = Point2D { x: 1, y: 2 };
    println!("{}", p);
    Ok(())
}
