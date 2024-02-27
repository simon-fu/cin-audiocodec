use std::{fmt, convert::TryFrom, ops::Range};

use anyhow::{Result, bail, Context};
use bytes::Buf;


use super::tag_buf::MAX_SEG_LEN;
use super::varint::decode::decode_varint32;

use super::{Type, Header};


pub struct TagRef<'a> {
    rtype: Type,
    data: &'a [u8],
}

impl<'a> fmt::Debug for TagRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tag")
        .field("rtype", &self.rtype)
        .field("payload", &self.data.len())
        .finish()
    }
}

impl<'a> TryFrom<&'a [u8]> for TagRef<'a> {
    type Error = ();

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> { 
        let r = Self::parse_slice(buf)?;
        Ok(r.0)
    }
}

impl<'a> TagRef<'a> { 
    pub fn new(rtype: Type, data: &'a [u8]) -> Self {
        Self { rtype, data, }
    }

    pub fn parse_slice(buf: &'a [u8]) -> Result<(Self, &'a [u8]), ()> { 
        let r = Header::try_parse(buf);
        match r {
            Some((rtype, len)) => {
                if len > buf.len() {
                    return Err(())
                }
                let value = &buf[Header::SIZE..Header::SIZE+len];
                let remaining = &buf[Header::SIZE+len..];
                Ok((Self::new(rtype, value), remaining))
            },
            None => Err(()),
        } 
    }

    pub fn full_len(&self) -> usize {
        Header::SIZE + self.data.len()
    }

    pub fn is_last(&self) -> bool {
        self.data.len() < MAX_SEG_LEN
    }

    pub fn rtype(&self) -> Type {
        self.rtype
    }

    pub fn value(&self) -> ValueRef<'a> {
        ValueRef::new(self.data)
    }
}

#[derive(Copy)]
pub struct ValueRef<'a> {
    data: &'a [u8],
}

impl<'a> Clone for ValueRef<'a> {
    fn clone(&self) -> Self {
        Self { data: self.data }
    }
}

impl<'a> ValueRef<'a> { 
    pub fn new(data: &'a [u8]) -> Self {
        Self {data}
    }

    pub fn as_magic(&self) -> Result<(i64, &'a str, &'a str)> {
        self.as_i64_str2()
    }

    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }

    pub fn cut_milli(&mut self) -> Result<i64> {
        self.cut_i64()
    }

    pub fn cut_i64(&mut self) -> Result<i64> {
        let mut cut = self.cut_slice_origin(8, "cut_milli")?;
        Ok(cut.get_i64())
    }

    pub fn cut_u64(&mut self) -> Result<u64> {
        let mut cut = self.cut_slice_origin(8, "cut_milli")?;
        Ok(cut.get_u64())
    }

    pub fn cut_bool(&mut self) -> Result<bool> {
        let cut = self.cut_slice_origin(1, "cut_bool")?;
        Ok(cut[0] != 0)
    }

    pub fn cut_u8(&mut self) -> Result<u8> {
        let cut = self.cut_slice_origin(1, "cut_u8")?;
        Ok(cut[0])
    }

    pub fn cut_u16(&mut self) -> Result<u16> {
        let mut cut = self.cut_slice_origin(2, "cut_u16")?;
        Ok(cut.get_u16())
    }

    pub fn cut_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        self.cut_slice_origin(n, "cut_slice")
    }

    pub fn cut_str(&mut self) -> Result<&'a str> {
        let d1 = self.cut_unit_origin("cut_str")?;
        let s1 = std::str::from_utf8(d1).with_context(||"cut_str but invalid")?;
        Ok(s1)
    }

    pub fn cut_unit(&mut self) -> Result<&'a [u8]> {
        self.cut_unit_origin("cut_unit")
    }

    fn cut_unit_origin(&mut self, origin: &str) -> Result<&'a [u8]> {
        let (d1, d2) = cut_unit(self.data).with_context(||format!("cur unit origin [{}]", origin))?;
        self.data = d2;
        Ok(d1)
    }

    fn cut_slice_origin(&mut self, n: usize, origin: &str) -> Result<&'a [u8]> {
        if self.data.len() < n {
            bail!("cut slice origin [{}], expect len [{}] but [{}]", origin, n, self.data.len())
        }
        let cut = &self.data[..n];
        self.data = &self.data[n..];
        Ok(cut)
    }

    pub fn as_i64(&self) -> Result<i64> { 
        let data = &self.data[..];
        let (v1, data) = cut_i64(data)?;
        if !data.is_empty() {
            bail!("as_i64: has remaining")
        }
        Ok(v1)
    }

    pub fn as_str(&self) -> Result<&'a str> { 
        let v1 = std::str::from_utf8(self.data)?;
        Ok(v1)
    }

    pub fn as_i64_str(&self) -> Result<(i64, &str)> { 
        let data = &self.data[..];
        let (v1, data) = cut_i64(data)?;
        let v2 = std::str::from_utf8(data)?;
        Ok((v1, v2))
    }

    pub fn as_i64_str2(&self) -> Result<(i64, &'a str, &'a str)> { 
        let data = &self.data[..];
        let (v1, data) = cut_i64(data)?;
        let (v2, data) = cut_ustr(data)?;
        let v3 = std::str::from_utf8(data)?;
        Ok((v1, v2, v3))
    }

    /// two unit str
    pub fn as_ustr2(&self) -> Result<(&'a str, &'a str)> { 
        let data = &self.data[..];
        let (v1, data) = cut_ustr(data)?;
        let (v2, data) = cut_ustr(data)?;
        if !data.is_empty() {
            bail!("as_str2: has remaining")
        }
        Ok((v1, v2))
    }

    pub fn as_units2(&self) -> Result<(&'a [u8], &'a [u8])> { 
        let data = &self.data[..];
        let (v1, data) = cut_unit(data)?;
        let (v2, data) = cut_unit(data)?;
        if !data.is_empty() {
            bail!("as_units2: has remaining")
        }
        Ok((v1, v2))
    }

    pub fn as_units(&self) -> Result<UnitsRef<'a>> { 
        let units = UnitsRef::parse(&self.data[..])?;
        Ok(units)
    }

    // pub fn split_slice_units(&self, len0: usize) -> Result<(&[u8], UnitsRef<'a>)> { 
    //     let first = &self.data[..len0];
    //     let units = UnitsRef::parse(&self.data[len0..])?;
    //     Ok((first, units))
    // }

    // pub fn split_i64_units(&self) -> Result<(i64, UnitsRef<'a>)> {
    //     let (first, units) = self.split_slice_units(8)?;
    //     let first = (&first[..]).get_i64();
    //     Ok((first, units))
    // }

    // pub fn split_i64(self) -> Result<(i64, Self)> {
    //     let data = &self.data[..];
    //     let (v1, data) = cut_i64(data)?;
    //     Ok((v1, Self { data }))
    // }

}


fn cut_i64(data: &[u8]) -> Result<(i64, &[u8])> {
    if data.len() < 8 {
        bail!("cut_i64: error len {}", data.len())
    }
    let v1 = (&data[..]).get_i64();
    Ok((v1, &data[8..]))
}

fn cut_ustr(data: &[u8]) -> Result<(&str, &[u8])> { 
    let (unit, data) = cut_unit(data)?;
    let s = std::str::from_utf8(unit)?;
    Ok((s, data))
}

fn cut_unit(data: &[u8]) -> Result<(&[u8], &[u8])> { 
    let (start, end) = parse_unit(data).with_context(||"cut_unit: error")?;
    Ok((&data[start..end], &data[end..]))
}

#[derive(Clone)]
pub struct UnitsRef<'a> { 
    data: &'a [u8],
    num: usize,
}

impl<'a> UnitsRef<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self> {

        let mut num = 0;
        let mut iter = RangeIter(data, 0, data.len());
        while let Some(_range) = iter.next() {
            num += 1;
        }

        if iter.1 != data.len() {
            bail!("parse units but has remaining")
        }

        Ok(Self{
            data,
            num,
        })

        // let iter = SliceIter(RangeIter(data, 0, data.len()));
    }

    pub fn num(&self) -> usize {
        self.num
    }

    pub fn iter(&self) -> SliceIter<'a> {
        SliceIter(RangeIter(self.data, 0, self.data.len()))
    }
}

pub struct SliceIter<'a>(RangeIter<'a>);

impl<'a> Iterator for SliceIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> { 
        let r = self.0.next();
        match r {
            Some(range) => Some(&self.0.0[range]),
            None => None,
        }
    }
}

pub struct RangeIter<'a>(&'a [u8], usize, usize);

impl<'a> Iterator for RangeIter<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.1 < self.2 { 
            let r = parse_unit(&self.0[self.1..]);
            if let Some((start, end)) = r {
                self.1 = end;
                return Some(start..end)
            }

            // let r = decode_varint32(&self.0[self.1..]).ok();
            // if let Some(Some((len, pos))) = r {
            //     let start = self.1 + pos;
            //     let end = start + len as usize;
            //     self.1 = end;
            //     return Some(start..end)
            // } 
        }

        None
    }
}

fn parse_unit(data: &[u8]) -> Option<(usize, usize)> {
    let r = decode_varint32(data).ok();
    if let Some(Some((len, pos))) = r {
        let start = 0 + pos;
        let end = start + len as usize;
        return Some((start, end))
    } 
    None
}




#[cfg(test)]
mod test {
    use crate::tlv2::tag_buf::{ValueAppender, AsBuf};

    use super::super::tag_buf::{TagBuf, MAX_SEG_LEN};

    use super::*;

    #[test]
    fn test_tag_value() {  
        {
            let mut buf = TagBuf::new();
            buf.begin_tag(Type::MAGIC)
            .append_len_value("magic123")
            .append_len_value("desc456")
            .finish();
    
            let data: Vec<u8> = buf.split().to_vec();
            let tag = TagRef::try_from(&data[..]).unwrap();
            assert_eq!(tag.rtype(), Type::MAGIC) ;
            assert_eq!(tag.value().as_ustr2().unwrap(), ("magic123", "desc456")) ;
        }

        {
            let mut buf = TagBuf::new();
            buf.tag_builder()
            .begin_tag(Type::MAGIC)
            .append_len_value("magic123")
            .append_len_value("desc456")

            .begin_tag(Type::CUSTOM)
            .append_fixed(&123_i64)
            
            .begin_tag(Type::CUSTOM)
            .append_fixed(&456_i64)
            .append_last("last321");
    
            let data: Vec<u8> = buf.to_vec();
            let data = &data[..];

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::MAGIC) ;
                assert_eq!(tag.value().as_ustr2().unwrap(), ("magic123", "desc456")) ;
                data
            };

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::CUSTOM) ;
                assert_eq!(tag.value().as_i64().unwrap(), 123_i64) ;
                data
            };

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::CUSTOM) ;
                assert_eq!(tag.value().as_i64_str().unwrap(), (456_i64, "last321")) ;
                data
            };

            assert!(data.is_empty(), "data.len {}", data.len());
        }
    }

    #[test]
    fn test_tag_value_big() {  
        
        {
            let max_len = MAX_SEG_LEN + 100;
            let value = BigValue::new(max_len);

            let mut buf = TagBuf::new();
            buf.begin_tag(Type::MAGIC)
            .append_last(&value);

            let data: Vec<u8> = buf.split().to_vec();
            let mut pos = 0;

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::MAGIC) ;
                assert_eq!(tag.value().as_slice(), BigValue::build_data(pos, pos + MAX_SEG_LEN));
                pos += MAX_SEG_LEN;
                data
            };

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::MAGIC) ;
                assert_eq!(tag.value().as_slice(), BigValue::build_data(pos, pos + 100));
                pos += 100;
                data
            };

            assert!(data.is_empty(), "data.len {}", data.len());
            assert_eq!(pos, max_len);
        }

        {
            let max_len = MAX_SEG_LEN + 100;
            let value = BigValue::new(max_len);

            let mut buf = TagBuf::new();
            buf.begin_tag(Type::MAGIC)
            .append_fixed(&123_i64)
            .append_last(&value);

            let data: Vec<u8> = buf.split().to_vec();
            let mut pos = 0;

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::MAGIC) ;

                // let value = tag.value();

                // let (v1, value) = value.split_i64().unwrap();
                // assert_eq!(v1, 123_i64);

                let mut value = tag.value();

                assert_eq!(value.cut_i64().unwrap(), 123_i64);

                assert_eq!(value.as_slice(), BigValue::build_data(pos, pos + MAX_SEG_LEN-8));
                pos += MAX_SEG_LEN-8;
                data
            };

            let data = {
                let (tag, data) = TagRef::parse_slice(&data[..]).unwrap();
                assert_eq!(tag.rtype(), Type::MAGIC) ;
                assert_eq!(tag.value().as_slice(), BigValue::build_data(pos, max_len));
                pos = max_len;
                data
            };

            assert!(data.is_empty(), "data.len {}", data.len());
            assert_eq!(pos, max_len);
        }

    }

    #[derive(Debug, Clone)]
    struct BigValue {
        len: usize,
    }

    impl BigValue {
        pub fn new(len: usize) -> Self {
            Self { len }
        }

        pub fn build_data(begin: usize, end: usize) -> Vec<u8> {
            let mut vec = Vec::new();
            for n in begin..end {
                vec.push(n as u8);
            }
            vec
        }
    }

    // impl<'a> IntoAsBuf for &'a BigValue {
    //     type Buf = &'a BigValueBuf;
    //     fn into_as_buf(self) -> Self::Buf {
    //         self
    //     }
    // }

    impl AsBuf for BigValue {
        type Buf<'a> = BigValueBuf where Self: 'a;
        fn as_buf(&self) -> Self::Buf<'_> {
            BigValueBuf{ pos: 0, len: self.len }
        }
    }

    impl AsBuf for &BigValue {
        type Buf<'a> = BigValueBuf where Self: 'a;
        fn as_buf(&self) -> Self::Buf<'_> {
            BigValueBuf{ pos: 0, len: self.len }
        }
    }

    // impl AsBufMark for BigValue {}

    #[derive(Debug, Clone)]
    struct BigValueBuf {
        pos: usize,
        len: usize,
    }

    impl Buf for BigValueBuf {
        fn remaining(&self) -> usize {
            self.len - self.pos
        }

        fn chunk(&self) -> &[u8] { 
            const BUF_SIZE: usize = 1024;
            lazy_static::lazy_static! {
                static ref SAMPLES: Vec<u8> = BigValue::build_data(0, BUF_SIZE);
            }
            
            let pos = self.pos % BUF_SIZE;
            let end = (pos+self.remaining()).min(BUF_SIZE);
            &SAMPLES[pos..end]
        }

        fn advance(&mut self, cnt: usize) {
            self.pos += cnt;
        }
    }

    // impl ToBuf for BigValue {
    //     type Buf<'a> = Self where Self: 'a;

    //     fn to_buf(&self) -> Self::Buf<'_> {
    //         self.clone()
    //     }
    // }

    // impl AsBuf for BigValue {
    //     type Buf<'a> = Self where Self: 'a;

    //     fn as_buf(&self) -> Self::Buf<'_> {
    //         self.clone()
    //     }
    // }

}
