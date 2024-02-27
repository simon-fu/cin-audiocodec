// refer from https://github.com/stepancheg/rust-protobuf/blob/v3.2.0/protobuf/src/varint/mod.rs

pub mod decode;
pub mod encode;
pub mod generic;

/// Encoded varint message is not longer than 10 bytes.
pub const MAX_VARINT_ENCODED_LEN: usize = 10;
pub const MAX_VARINT32_ENCODED_LEN: usize = 5;

