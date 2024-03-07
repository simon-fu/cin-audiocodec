


use bytes::{BufMut, Buf};
use chrono::Local;

use super::varint::zigzag::encode_zig_zag_64;
use super::varint::{MAX_VARINT32_ENCODED_LEN, MAX_VARINT_ENCODED_LEN};
use super::varint::encode::{encode_varint32_size, encode_varint64_size};

use super::seg_buf::{AllocSeg, SegBuf, Cursor, SegList};
use super::Type;


#[derive(Debug)]
pub struct TagBuf<A: AllocSeg = ()> { 
    buf: SegBuf<A>,
}

impl Default for TagBuf {
    fn default() -> Self {
        Self { buf: SegBuf::default() }
    }
}


impl<A: AllocSeg> GetTagBuf for &mut TagBuf<A> {
    type Alloc = A;

    fn get_tag_buf(&mut self) -> &mut TagBuf<Self::Alloc> {
        self
    }
}

impl TagBuf { 
    pub fn new() -> Self {
        Self::default()
    }
}

impl<A: AllocSeg> TagBuf<A> { 

    pub fn with_alloc(alloc: A) -> Self { 
        Self { buf: SegBuf::with_alloc(alloc) }
    }

    pub fn tag_builder<'a>(&'a mut self) -> TagBuilder<&'a mut Self> { 
        TagBuilder::new(self)
    }

    pub fn begin_tag<'a, T: Into<Type>>(&'a mut self, rtype: T) -> SingleTag<&'a mut Self> { 
        SingleTag::begin_with(rtype, self)
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn split(&mut self) -> SegList {
        self.buf.split()
    }

    pub fn into_seg_list(self) -> SegList {
        self.buf.into_list()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.buf.to_vec()
    }
}



pub trait GetTagBuf {
    type Alloc: AllocSeg;
    fn get_tag_buf(&mut self) -> &mut TagBuf<Self::Alloc>;
}

pub trait ValueAppender: AsBufMut {

    fn append_value<A: AppendToTag>(self, v: &A)
    where Self: Sized
    {
        v.append_to_tag(self)
    }

    fn append_now_milli(self) -> Self ;

    fn append_fixed<V>(self, v: V) -> Self 
    where
        V: IntoAsBuf + FixedSize;

    // fn append_len_value<V: AsBuf>(self, v: &V) -> Self ;
    fn append_len_value<V: IntoAsBuf>(self, v: V) -> Self ;

    fn append_var_u64(self, v: u64) -> Self ;

    fn append_var_i64(self, v: i64) -> Self ;

    fn append_last<V: IntoAsBuf>(self, v: V)  ;
    
    fn finish(self) ;
} 

pub trait AppendToTag {
    fn append_to_tag<T: ValueAppender>(&self, tag: T) ;
}


pub struct TagBuilder<C: GetTagBuf> {
    ctx: C,
}

impl<C: GetTagBuf> TagBuilder<C> { 
    pub fn new(ctx: C) -> Self { 
        Self { ctx } 
    }

    pub fn begin_tag(self, rtype: Type) -> MultiTag<C> {  
        MultiTag(SingleTag::begin_with(rtype, self.ctx))
    }
}

pub struct MultiTag<C: GetTagBuf>(SingleTag<C>);

impl<C: GetTagBuf> MultiTag<C> {
    pub fn begin_tag(mut self, rtype: Type) -> MultiTag<C> {  
        self.0.begin_tag_(rtype);
        MultiTag(self.0)
    }

    pub fn append_last<V: IntoAsBuf>(self, v: V) -> More<C> {
        More(self.0.append_last_(v))
    }
}

impl<C: GetTagBuf> ValueAppender for MultiTag<C> {
    fn append_now_milli(self) -> Self {
        MultiTag(self.0.append_now_milli())
    }

    fn append_fixed<V>(self, v: V) -> Self 
    where
        V: IntoAsBuf + FixedSize,
    {
        MultiTag(self.0.append_fixed(v))
    }

    fn append_len_value<V: IntoAsBuf>(self, v: V) -> Self { 
        MultiTag(self.0.append_len_value(v))
    }

    fn append_var_u64(self, v: u64) -> Self  {
        MultiTag(self.0.append_var_u64(v))
    }

    fn append_var_i64(self, v: i64) -> Self {
        MultiTag(self.0.append_var_i64(v))
    }

    fn append_last<V: IntoAsBuf>(self, v: V) {
        More(self.0.append_last_(v));
    }

    fn finish(self) {
        self.0.finish()
    }
}

pub struct More<C: GetTagBuf>(SingleTag<C>);

impl<C: GetTagBuf> More<C> { 
    pub fn begin_tag(mut self, rtype: Type) -> MultiTag<C> {  
        self.0.begin_tag_(rtype);
        MultiTag(self.0)
    }

    pub fn finish(self) {
        self.0.finish()
    }
}

impl<C: GetTagBuf> AsBufMut for MultiTag<C> {
    type Buf<'a> = &'a mut SegBuf<C::Alloc> where Self: 'a;

    fn as_buf_mut(&mut self) -> Self::Buf<'_> {
        self.0.as_buf_mut()
    }
}


// pub struct TagBuilder<C: GetTagBuf> {
//     rtype: Type,
//     ctx: C,
//     len_cursor: Cursor,
// }

// impl<C: GetTagBuf> Drop for TagBuilder<C> {
//     fn drop(&mut self) { 
//         let buf = &mut self.ctx.get_tag_buf().buf;
//         write_back_len(buf, &self.len_cursor);
//     }
// }

// impl<C: GetTagBuf> TagBuilder<C> {
//     pub fn begin_with<T: Into<Type>>(rtype: T, mut ctx: C) -> Self { 
//         let rtype = rtype.into();
//         let buf = &mut ctx.get_tag_buf().buf;
//         return Self {
//             len_cursor: begin_tag(buf, rtype),
//             rtype,
//             ctx
//         } 
//     }

//     // pub fn begin_tag(&mut self, rtype: Type) -> TagMut<'_, C> {  
//     //     self.begin_tag_(rtype);
        
//     //     return TagMut(self) 
//     // }

//     pub fn begin_tag(mut self, rtype: Type) -> Self {  
//         self.begin_tag_(rtype);
        
//         return self 
//     }

//     pub fn append_now_milli(self) -> Self {
//         self.append_fixed(&Local::now().timestamp_millis())
//     }

//     pub fn append_fixed<V>(mut self, v: &V) -> Self 
//     where
//         V: ToBuf + FixedSize,
//     {
//         let buf = v.to_buf();
//         self.write(buf.as_buf());
//         self
//     }

//     pub fn append_len_value<V: AsBuf>(mut self, v: &V) -> Self { 
//         let v = v.as_buf();
//         let len = v.remaining() as u32;
        
//         let mut bytes1 = [0; MAX_VARINT32_ENCODED_LEN];
//         let len1 = encode_varint32_size(len, &mut bytes1);
//         self.write(Buf2(&bytes1[..len1], v));

//         self
//     }

//     pub fn append_last<V: AsBuf>(mut self, v: &V) {
//         let v = v.as_buf();
//         self.write(v);
//         self.finish()
//     }

//     pub fn finish(self) { }

// }



pub struct SingleTag<C: GetTagBuf> {
    rtype: Type,
    ctx: C,
    len_cursor: Cursor,
}

impl<C: GetTagBuf> Drop for SingleTag<C> {
    fn drop(&mut self) { 
        let buf = &mut self.ctx.get_tag_buf().buf;
        write_back_len(buf, &self.len_cursor);
    }
}

impl<C: GetTagBuf> SingleTag<C> {
    pub fn begin_with<T: Into<Type>>(rtype: T, mut ctx: C) -> Self { 
        let rtype = rtype.into();
        let buf = &mut ctx.get_tag_buf().buf;
        return Self {
            len_cursor: begin_tag(buf, rtype),
            rtype,
            ctx
        } 
    }
}

impl<C: GetTagBuf> ValueAppender for SingleTag<C> {
    fn append_now_milli(self) -> Self {
        self.append_fixed(&Local::now().timestamp_millis())
    }

    fn append_fixed<V>(mut self, v: V) -> Self 
    where
        V: IntoAsBuf + FixedSize,
    {
        let buf = v.into_as_buf();
        self.write(buf.as_buf());
        self
    }

    // fn append_len_value<V: AsBuf>(mut self, v: &V) -> Self { 
    fn append_len_value<V: IntoAsBuf>(mut self, v: V) -> Self { 
        let v = v.into_as_buf();
        let v = v.as_buf();
        let len = v.remaining() as u32;
        
        let mut bytes1 = [0; MAX_VARINT32_ENCODED_LEN];
        let len1 = encode_varint32_size(len, &mut bytes1);
        self.write(Buf2(&bytes1[..len1], v));

        self
    }

    fn append_var_u64(mut self, v: u64) -> Self  {
        let mut bytes1 = [0; MAX_VARINT_ENCODED_LEN];
        let len1 = encode_varint64_size(v, &mut bytes1);
        self.write(&bytes1[..len1]);
        self
    }

    fn append_var_i64(self, v: i64) -> Self {
        let v = encode_zig_zag_64(v);
        self.append_var_u64(v)
    }

    fn append_last<V: IntoAsBuf>(self, v: V) {
        self.append_last_(v).finish()
    }

    fn finish(self) { }
}

impl<C: GetTagBuf> AsBufMut for SingleTag<C> {
    type Buf<'a> = &'a mut SegBuf<C::Alloc> where Self: 'a;

    fn as_buf_mut(&mut self) -> Self::Buf<'_> {
        &mut self.ctx.get_tag_buf().buf
    }
}

impl<C: GetTagBuf> SingleTag<C> {

    fn begin_tag_(&mut self, rtype: Type) {  
        
        let buf = &mut self.ctx.get_tag_buf().buf;

        write_back_len(buf, &self.len_cursor);
        
        self.rtype = rtype;
        self.len_cursor = begin_tag(buf, rtype);
    }

    fn append_last_<V: IntoAsBuf>(mut self, v: V) -> Self {
        let v = v.into_as_buf();
        let v = v.as_buf();
        self.write(v);
        self
    }

    fn write<V: Buf>(&mut self, mut v: V) {

        let buf = &mut self.ctx.get_tag_buf().buf;

        let len = current_len(buf, &self.len_cursor);

        if (len + v.remaining()) < MAX_SEG_LEN {
            buf.put(v);

        } else {

            let limit = MAX_SEG_LEN - len;
            buf.put_limit(&mut v, limit);

            write_back_len(buf, &self.len_cursor);
            self.len_cursor = begin_tag(buf, self.rtype);

            // write_back_len(buf,&self.len_cursor);
            // self.len_cursor = buf.cursor();
            // buf.put_slice(&ZERO_LEN_BYTES);

            while v.remaining() >= MAX_SEG_LEN {

                buf.put_limit(&mut v, MAX_SEG_LEN);

                write_back_len(buf, &self.len_cursor);
                self.len_cursor = begin_tag(buf, self.rtype);

                // write_back_len(buf, &self.len_cursor);
                // self.len_cursor = buf.cursor();
                // buf.put_slice(&ZERO_LEN_BYTES);
            }

            if v.remaining() > 0 {
                buf.put(v);
            }
        }
    }


}

fn current_len<A: AllocSeg>(buf: &SegBuf<A>, len_cursor: &Cursor) -> usize { 
    buf.len_from_cursor(len_cursor) - 3
}

fn begin_tag<A: AllocSeg>(buf: &mut SegBuf<A>, rtype: Type) -> Cursor {
    buf.put_u8(rtype.value());

    let len_cursor = buf.cursor();
    buf.put_slice(&ZERO_LEN_BYTES);
    len_cursor
}

fn write_back_len<A: AllocSeg>(buf0: &mut SegBuf<A>, len_cursor: &Cursor) {

    let mut buf = buf0.at_cursor_mut(len_cursor);

    let len = buf.remaining_mut() - 3;
    let bytes = (len as u32).to_be_bytes();
    buf.put_slice(&bytes[1..]);
}

const ZERO_LEN_BYTES: [u8; 3] = [0, 0, 0];

struct Buf2<V1: Buf, V2: Buf>(V1, V2);

impl<V1: Buf, V2: Buf> Buf for Buf2<V1, V2> {
    fn remaining(&self) -> usize {
        self.0.remaining() + self.1.remaining()
    }

    fn chunk(&self) -> &[u8] {
        let chuck = self.0.chunk() ;
        if chuck.len() > 0{
            chuck
        } else {
            self.1.chunk()
        }
    }

    fn advance(&mut self, mut cnt: usize) { 
        let remaining1 = self.0.remaining();
        if remaining1 > 0 {
            let cnt1 = remaining1.min(cnt);
            self.0.advance(cnt1);
            cnt -= cnt1;
        }

        if cnt > 0 {
            self.1.advance(cnt);
        }
    }
}

pub(super) const MAX_SEG_LEN: usize = (1 << 24) - 1;

pub trait AppendToTagBuf<A: AllocSeg=()> {
    fn append_to_tag_buf(&self, buf: &mut TagBuf<A>);
    
    // fn to_vec(&self) -> Vec<u8> {
    //     let mut buf = TagBuf::default();
    //     self.append_to_tag_buf(&mut buf);
    //     buf.to_vec()
    // }
}

pub trait TagToVec<A = ()> {
    fn to_vec(&self) -> Vec<u8> ;
}

impl<A: AllocSeg + Default, T: AppendToTagBuf<A>> TagToVec<A> for T {
    fn to_vec(&self) -> Vec<u8> {
        let mut buf = TagBuf::with_alloc(A::default());
        self.append_to_tag_buf(&mut buf);
        buf.to_vec()
    }
}

// pub trait AppendToTagBuf {
//     fn append_to_tag_buf<A: AllocSeg>(&self, buf: &mut TagBuf<A>);
    
//     fn to_vec(&self) -> Vec<u8> {
//         let mut buf = TagBuf::default();
//         self.append_to_tag_buf(&mut buf);
//         buf.to_vec()
//     }
// }


pub struct TagObj<T: IntoAsBuf+Copy>(pub Type, pub T);

impl<T: IntoAsBuf+Copy, A: AllocSeg> AppendToTagBuf<A> for TagObj<T> { 
    fn append_to_tag_buf(&self, buf: &mut TagBuf<A>) {
        buf.begin_tag(self.0).append_last(self.1);
    }
}

pub trait AsBuf { 
    type Buf<'a>: Buf where Self: 'a;
    fn as_buf(&self) -> Self::Buf<'_>;
}

pub trait AsBufMut { 
    type Buf<'a>: BufMut where Self: 'a;
    fn as_buf_mut(&mut self) -> Self::Buf<'_>;
}

pub trait AsBufMark: Buf + Clone { }

impl AsBufMark for &[u8] {}

impl<T: AsBufMark> AsBuf for T 
{
    type Buf<'a> = Self where Self: 'a;

    fn as_buf(&self) -> Self::Buf<'_> {
        self.clone()
    }
}

impl<const N: usize> AsBuf for [u8; N] {
    type Buf<'a> = &'a [u8];

    fn as_buf(&self) -> Self::Buf<'_> {
        &self[..]
    }
}

impl AsBuf for &str {
    type Buf<'a> = &'a [u8] where Self: 'a;

    fn as_buf(&self) -> Self::Buf<'_> {
        self.as_bytes()
    }
}

impl AsBuf for &String {
    type Buf<'a> = &'a [u8] where Self: 'a;

    fn as_buf(&self) -> Self::Buf<'_> {
        self.as_bytes()
    }
}


pub trait IntoAsBuf { 
    type Buf: AsBuf ;
    fn into_as_buf(self) -> Self::Buf;
}

impl<T: AsBuf> IntoAsBuf for T {
    type Buf = Self;
    fn into_as_buf(self) -> Self::Buf {
        self
    }
}

impl IntoAsBuf for () {
    type Buf = &'static [u8];
    fn into_as_buf(self) -> Self::Buf {
        &[]
    }
}

impl IntoAsBuf for bool {
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [u8::from(self)]
    }
}

impl IntoAsBuf for &bool {
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [u8::from(*self)]
    }
}

impl IntoAsBuf for u8 {
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [self]
    }
}

impl IntoAsBuf for &u8 {
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [*self]
    }
}

impl IntoAsBuf for i8 { 
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [self as u8]
    }
}

impl IntoAsBuf for &i8 { 
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [*self as u8]
    }
}

impl IntoAsBuf for u16 { 
    type Buf = [u8; 2];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &u16 { 
    type Buf = [u8; 2];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for i16 {
    type Buf = [u8; 2];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &i16 {
    type Buf = [u8; 2];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for u32 {
    type Buf = [u8; 4];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &u32 {
    type Buf = [u8; 4];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for i32 {
    type Buf = [u8; 4];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &i32 {
    type Buf = [u8; 4];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}


impl IntoAsBuf for u64 {
    type Buf = [u8; 8];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &u64 {
    type Buf = [u8; 8];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for i64 {
    type Buf = [u8; 8];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl IntoAsBuf for &i64 {
    type Buf = [u8; 8];
    fn into_as_buf(self) -> Self::Buf {
        self.to_be_bytes()
    }
}

impl<'a, const N:usize> IntoAsBuf for &'a [u8; N] { 
    type Buf = &'a [u8];
    fn into_as_buf(self) -> Self::Buf {
        &self[..]
    }
}

// impl<'a> IntoAsBuf for &'a [u8] { 
//     type Buf = &'a [u8] where Self: 'a;
//     fn into_as_buf(self) -> Self::Buf {
//         &self[..]
//     }
// }

// impl<'a> IntoAsBuf for &'a str{ 
//     type Buf = &'a [u8] where Self: 'a;
//     fn into_as_buf(self) -> Self::Buf {
//         self.as_bytes()
//     }
// }


impl IntoAsBuf for Type {
    type Buf = [u8; 1];
    fn into_as_buf(self) -> Self::Buf {
        [self.value()]
    }
}




// pub trait ToBuf { 
//     type Buf<'a>: AsBuf where Self: 'a;
//     fn to_buf(&self) -> Self::Buf<'_>;
// }

// impl ToBuf for () {
//     type Buf<'a> = &'static [u8];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         &[]
//     }
// }

// impl ToBuf for u8 {
//     type Buf<'a> = [u8; 1];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         [*self]
//     }
// }

// impl ToBuf for i8 { 
//     type Buf<'a> = [u8; 1];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         [*self as u8]
//     }
// }

// impl ToBuf for u16 { 
//     type Buf<'a> = [u8; 2];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl ToBuf for i16 {
//     type Buf<'a> = [u8; 2];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl ToBuf for u32 {
//     type Buf<'a> = [u8; 4];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl ToBuf for i32 {
//     type Buf<'a> = [u8; 4];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl ToBuf for u64 {
//     type Buf<'a> = [u8; 8];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl ToBuf for i64 {
//     type Buf<'a> = [u8; 8];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.to_be_bytes()
//     }
// }

// impl<const N:usize> ToBuf for [u8; N] { 
//     type Buf<'a> = &'a [u8];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         &self[..]
//     }
// }

// impl ToBuf for &[u8] { 
//     type Buf<'a> = &'a [u8] where Self: 'a;

//     fn to_buf(&self) -> Self::Buf<'_> {
//         &self[..]
//     }
// }

// impl ToBuf for &str { 
//     type Buf<'a> = &'a [u8] where Self: 'a;

//     fn to_buf(&self) -> Self::Buf<'_> {
//         self.as_bytes()
//     }
// }

// impl ToBuf for Type {
//     type Buf<'a> = [u8; 1];

//     fn to_buf(&self) -> Self::Buf<'_> {
//         [self.value()]
//     }
// }




pub trait FixedSize { 
    fn fixed_size(&self) -> usize;
}

impl<T: FixedSize> FixedSize for &T {
    fn fixed_size(&self) -> usize { (*self).fixed_size() }
}

impl FixedSize for () {
    fn fixed_size(&self) -> usize { 0 }
}

impl FixedSize for bool {
    fn fixed_size(&self) -> usize { 1 }
}

impl FixedSize for u8 {
    fn fixed_size(&self) -> usize { 1 }
}

impl FixedSize for i8 { 
    fn fixed_size(&self) -> usize { 1 }
}

impl FixedSize for u16 { 
    fn fixed_size(&self) -> usize { 2 }
}

impl FixedSize for i16 {
    fn fixed_size(&self) -> usize { 2 }
}

impl FixedSize for u32 {
    fn fixed_size(&self) -> usize { 4 }
}

impl FixedSize for i32 {
    fn fixed_size(&self) -> usize { 4 }
}

impl FixedSize for u64 {
    fn fixed_size(&self) -> usize { 8 }
}

impl FixedSize for i64 {
    fn fixed_size(&self) -> usize { 8 }
}

impl<const N:usize> FixedSize for [u8; N] { 
    fn fixed_size(&self) -> usize { N }
}

impl FixedSize for Type {
    fn fixed_size(&self) -> usize { self.value().fixed_size() }
}


