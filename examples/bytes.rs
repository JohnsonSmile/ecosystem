use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"Hello world\n");
    buf.put(&b"AAA world"[..]);

    println!("{:?}", buf);
    // let mut b = buf.freeze(); // 移动给了b
    // println!("{:?}", b);
    // println!("{:?}", b'\n');
    let r = buf
        .iter()
        .enumerate()
        .find(|&(ref _idx, &c)| c.cmp(&b'd').is_eq());
    if let Some((idx, &c)) = r {
        println!("{}-{:?}", idx, c);
    }

    buf.sort();
    // [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 10, 65, 65, 65, 32, 119, 111, 114, 108, 100]
    // binary search 必须是有序的
    println!("{:?}", buf);
    let pos = buf.binary_search_by(|c| {
        println!("c:{:?} {:?}", c, &b'd');
        c.cmp(&b'd')
    });
    println!("{:?}", pos);

    Ok(())
}
