use std::{fmt, ops::Range};

use bytes::{Buf, Bytes, BytesMut};



// reading buf
pub trait RBuf<T> {
    type Item;
    fn rlen(&self) -> usize ;
    fn rdata(&self) -> &[T];
    fn radvance(&mut self, cnt: usize);
    fn rsplit_to(&mut self, at: usize) -> Self::Item;
}

impl<'a, T> RBuf<T> for &'a [T] {
    type Item = &'a [T];

    fn rlen(&self) -> usize  {
        self.len()
    }

    fn rdata(&self) -> &[T] {
        &self[..]
    }

    fn radvance(&mut self, cnt: usize) {
        *self = &self[cnt..]
        // self.advance(cnt);
    }

    fn rsplit_to(&mut self, at: usize) -> Self::Item {
        let part1 = &self[..at];
        *self = &self[at..];
        part1
    }
}


impl RBuf<u8> for BytesMut {
    type Item = Bytes;

    fn rlen(&self) -> usize  {
        self.len()
    }

    fn rdata(&self) -> &[u8] {
        &self[..]
    }

    fn radvance(&mut self, cnt: usize) {
        self.advance(cnt);
    }

    fn rsplit_to(&mut self, at: usize) -> Self::Item {
        self.split_to(at).freeze()
    }
}

#[derive(Default, Clone)]
pub struct RwBufVec<T> {
    buf: Vec<T>,
    begin: usize,
    end: usize,
}

impl<T> fmt::Debug for RwBufVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VecBuf")
        .field("cap", &self.buf.len())
        .field("begin", &self.begin)
        .field("end", &self.end)
        .finish()
    }
}

impl<T> fmt::Display for RwBufVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            "cap {}, begin {}, end {}",
            self.buf.len(),
            self.begin,
            self.end,
        )
    }
}

impl<T: Copy + Default> RwBufVec<T> {
    pub fn new(init_len: usize) -> Self {
        Self {
            buf: vec![T::default(); init_len],
            ..Default::default()
        }
    }

    pub fn trim(&mut self) {
        if self.begin > 0 {
            self.buf.copy_within(self.begin..self.end, 0); 
            self.end = self.end - self.begin;
            self.begin = 0;
        }
    }

    pub fn at(&self, range: &RRange) -> &[T]{
        &self.buf[range.0.start..range.0.end]
    }

    pub fn wsize(&self) -> usize {
        self.buf.len() - self.end
    }

    pub fn wbuf(&mut self) -> &mut [T] {
        let len = self.buf.len();
        &mut self.buf[self.end..len]
    }

    pub fn wadvance(&mut self, cnt: usize) {
        self.end += cnt;
    }

    pub fn reserve(&mut self, extra: usize) {
        self.buf.resize(self.buf.len() + extra, T::default());
    }

    pub fn trim_and_check_reserve(&mut self, extra: usize) -> &mut [T]{
        self.trim();
        if self.wsize() == 0 {
            self.reserve(extra);
        }
        self.wbuf()
    }

    pub fn push_rotate(&mut self, input: &[T]) {

        if input.len() > self.buf.len() {
            let off = input.len() - self.buf.len();
            let idata = &input[off..];
            (&mut self.buf[..]).copy_from_slice(idata);
            self.begin = 0;
            self.end = 0;
            self.wadvance(idata.len());
            return ;
        } 

        if self.wsize() >= input.len() {
            (&mut self.wbuf()[..input.len()]).copy_from_slice(input);
            self.wadvance(input.len());
            return;
        }

        let spare_len = self.begin + self.wsize();
        if spare_len >= input.len() {
            self.trim();
            (&mut self.wbuf()[..input.len()]).copy_from_slice(input);
            self.wadvance(input.len());
            return;
        }

        self.radvance(input.len() - spare_len);
        self.trim();
        (&mut self.wbuf()[..input.len()]).copy_from_slice(input);
        self.wadvance(input.len());

    }

}

impl<T> RBuf<T> for RwBufVec<T> {
    type Item = RRange;

    fn rlen(&self) -> usize  {
        self.end - self.begin
    }

    fn rdata(&self) -> &[T] {
        &self.buf[self.begin..self.end]
    }

    fn radvance(&mut self, cnt: usize) {
        self.begin += cnt;
    }

    fn rsplit_to(&mut self, at: usize) -> Self::Item {
        let range = self.begin..self.begin+at;
        self.begin += at;
        RRange(range)
    }
}

pub struct RRange(Range<usize>);


// #[derive(Debug, Default, Clone, Copy)]
// pub struct AnnexBParser {
//     zeros1: usize,
//     parsed: usize,
// }

// impl AnnexBParser {
//     pub fn parse_whole<'a>(data: &'a [u8]) -> NalIter<'a> {
//         NalIter {
//             data,
//             parser: Some(AnnexBParser::default()),
//         }
//     }

//     pub fn parse_buf<B: RBuf>(&mut self, buf: &mut B) -> Option<Unit<B::Item>> {
//         if self.zeros1 == 0 {
//             if !self.parse_first(buf) {
//                 return None
//             }
//         }

//         let rdata = buf.rdata();

//         let pos = search_start_code(&rdata[self.parsed..]);
//         match pos.offset {
//             Some(offset) => {
//                 let offset = self.parsed + offset ;
//                 let unit = Unit {
//                     start: self.zeros1+1,
//                     data: buf.rsplit_to(offset),
//                 };
//                 self.zeros1 = pos.zeros;
//                 self.parsed = self.zeros1 + 1;
//                 Some(unit)
//             },
//             None => {
//                 self.parsed = rdata.len() - pos.zeros;
//                 None
//             },
//         } 
//     }

//     fn parse_first<B: RBuf>(&mut self, buf: &mut B) -> bool {
//         let pos = search_start_code(buf.rdata());
//         match pos.offset {
//             Some(offset) => {
//                 buf.radvance(offset);
//                 self.zeros1 = pos.zeros;
//                 self.parsed = self.zeros1 + 1;
//                 true
//             },
//             None => {
//                 buf.radvance(buf.rlen() - pos.zeros);
//                 false
//             },
//         }
//     }

// }

// pub struct Unit<T> {
//     pub start: usize,
//     pub data: T,
// }

// impl<'a> Unit<&'a [u8]> {
//     pub fn payload(&'a self) -> &'a [u8] {
//         &self.data[self.start..]
//     }
// }



// // pub fn nal_iter_from_slice<'a>(data: &'a [u8]) -> NalIter<'a> {
// //     NalIter {
// //         data,
// //         parser: Some(AnnexBParser::default()),
// //     }
// // }

// pub struct NalIter<'a> {
//     data: &'a [u8],
//     parser: Option<AnnexBParser>,
// }

// impl <'a> Iterator for NalIter<'a> {
//     type Item = Unit<&'a[u8]>;

//     fn next(&mut self) -> Option<Self::Item> {

//         let r = self.parser.as_mut().map(|x|x.parse_buf(&mut self.data)).unwrap_or(None);

//         match r {
//             Some(d) => Some(d),
//             None => {
//                 match self.parser.take() {
//                     Some(parser) => {
//                         if parser.zeros1 > 0 {
//                             let unit = Unit {
//                                 start: parser.zeros1 + 1,
//                                 data: &self.data[..],
//                             };
//                             self.data = &[];
//                             Some(unit)
//                         } else {
//                             None
//                         }
//                     },
//                     None => None,
//                 }
//             }
//         }
//     }
// }


// fn search_start_code(data: &[u8]) -> StartCodePos {
//     let mut pos = StartCodePos::default();

//     for (offset, item) in data.iter().enumerate() {
//         if *item == 0 {
//             pos.zeros += 1;
//         } else if *item == 1 {
//             if pos.zeros >= 2 {
//                 pos.offset = Some(offset - pos.zeros);
//                 break;
//             }
//         } else {
//             pos.zeros = 0;
//         }
//     }
//     pos
// }

// #[derive(Debug, Default)]
// struct StartCodePos {
//     offset: Option<usize>,
//     zeros: usize,
// }


