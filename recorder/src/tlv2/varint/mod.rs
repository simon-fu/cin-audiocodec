// refer from https://github.com/stepancheg/rust-protobuf/blob/v3.2.0/protobuf/src/varint/mod.rs

pub mod decode;
pub mod encode;
pub mod generic;
pub mod zigzag;
/// Encoded varint message is not longer than 10 bytes.
pub const MAX_VARINT_ENCODED_LEN: usize = 10;
pub const MAX_VARINT32_ENCODED_LEN: usize = 5;

#[test]
fn test() {
    use std::mem::MaybeUninit;

    let mut buf = [0_u8; 10];
    let n = encode::encode_varint32_size(100, &mut buf[..]);
    println!("encoded len {n}");
    let mut buf = [MaybeUninit::uninit(); 10];
    let n = encode::encode_varint64(1709094963000, &mut buf[..]);
    println!("encoded len {n}");

    let value = 1709094963000;
    let znum = zigzag::encode_zig_zag_64(value);
    let n = encode::encode_varint64(znum, &mut buf[..]);
    println!("encoded {value}: len {n}, znum {znum}");

    let value = 0x000FFFFF;
    let znum = zigzag::encode_zig_zag_64(value);
    let n = encode::encode_varint64(znum, &mut buf[..]);
    println!("encoded {value}: len {n}, znum {znum}");
}


