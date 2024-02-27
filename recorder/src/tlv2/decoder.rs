use anyhow::Context;
use bytes::{Bytes, BytesMut, Buf};
use tokio_util::codec::Decoder;

use super::{Header, Type};

#[derive(Debug, Default)]
pub struct TlvDecoder;


impl Decoder for TlvDecoder {
    type Item = (Type, Bytes);
    type Error = anyhow::Error;

    #[inline]
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < Header::SIZE {
            return Ok(None)
        }

        let (rtype, payload_len) = Header::try_parse(&src[..]).with_context(||"invalid tlv header")?;
        if src.len() < Header::SIZE + payload_len {
            return Ok(None)
        }

        src.advance(Header::SIZE);
        let data = src.split_to(payload_len).freeze();

        Ok(Some((rtype, data)))
    }
}
