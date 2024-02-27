


use bytes::{BufMut, buf::UninitSlice, BytesMut, Buf};
use tinyvec::TinyVec;


#[derive(Debug)]
pub struct SegBuf<A: AllocSeg = ()> {
    segs: BufQueue,
    len: usize,
    alloc: A,
}

impl Default for SegBuf {
    fn default() -> Self { 
        Self::with_alloc(())
    }
}

impl<A: AllocSeg> SegBuf<A> {
    pub fn with_alloc(alloc: A) -> Self {
        let mut self0 = Self { segs: Default::default(), len: 0, alloc };
        self0.alloc_seg();
        self0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn num_segs(&self) -> usize {
        self.segs.len()
    }

    pub fn cursor(&self) -> Cursor {
        Cursor { 
            index: self.num_segs() - 1, 
            pos: self.last().len(), 
            offset: self.len 
        }
    }


    pub fn clear(&mut self) {
        self.segs.clear();
        self.alloc_seg();
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_buf().into_vec()
    }
    
    /// NOT test yet
    pub fn as_buf(&self) -> RBuf { 
        RBuf { 
            segs: &self.segs[..],
            index: 0, 
            pos: 0, 
            remaining: self.len, 
        }
    }

    /// NOT test yet
    pub fn at_cursor(&mut self, cursor: &Cursor) -> RBuf<'_> {
        RBuf { 
            index: cursor.index, 
            pos: cursor.pos, 
            remaining: self.len - cursor.offset, 
            segs: &self.segs[..],
        }
    }

    pub fn at_cursor_mut(&mut self, cursor: &Cursor) -> WBuf<'_> {
        WBuf { 
            index: cursor.index, 
            pos: cursor.pos, 
            remaining: self.len - cursor.offset, 
            segs: &mut self.segs[..],
        }
    }

    pub fn len_from_cursor(&self, cursor: &Cursor) -> usize {
        self.len - cursor.offset
    }

    pub fn put_limit<T: Buf>(&mut self, src: &mut T, mut limit: usize) {
        // assert!(self.remaining_mut() >= src.remaining());
        assert!(self.remaining_mut() >= src.remaining().min(limit));

        while src.has_remaining() && limit > 0 {
            let l;

            unsafe {
                let s = src.chunk();
                let d = self.chunk_mut();

                let min = std::cmp::min(s.len(), d.len());
                l = std::cmp::min(min, limit);

                std::ptr::copy_nonoverlapping(s.as_ptr(), d.as_mut_ptr() as *mut u8, l);
            }

            src.advance(l);
            unsafe {
                self.advance_mut(l);
            }
            
            limit -= l;
        }
    }

    pub fn into_list(self) -> SegList { 
        SegList{list: self.segs}
    }

    pub fn split(&mut self) -> SegList { 

        if let Some(mut last) = self.segs.pop() { 

            let last_r = split_last(&mut last, &mut self.alloc);

            if self.segs.is_empty() {
                
                self.segs.push(last);

                if let Some((item, is_full)) = last_r {
                    if is_full {
                        return SegList::many(vec![item, empty_buf()])
                    } else {
                         return SegList::one(item)
                    }
                }
            } else {
                let list = std::mem::replace(&mut self.segs, Default::default());

                let mut list = match list {
                    TinyVec::Inline(a) => a.into_inner().into(),
                    TinyVec::Heap(v) => v,
                };

                self.segs.push(last);
            
                if let Some((item, is_full)) = last_r {
                    list.push(item);
                    if is_full {
                        list.push(empty_buf());
                    }
                } else {
                    list.push(empty_buf());
                }

                return SegList::many(list)
            }
        } 

        return SegList::one(empty_buf())



        // if self.segs.len() == 1 {
        //     if let Some((item, is_full)) = self.freeze_last() {
        //         if is_full {
        //             return SegList::many(vec![item, empty_buf()])
        //         } else {
        //             return SegList::one(item)
        //         }
        //     }
            
        // } else if self.segs.len() > 1 { 
            
            
        //     let mut list = Vec::with_capacity(self.segs.len());
        //     for item in self.segs.drain(0..self.segs.len()-1) {
        //         list.push(item);
        //     }

        //     if let Some((item, is_full)) = self.freeze_last() {
        //         list.push(item);
        //         if is_full {
        //             list.push(empty_buf());
        //         }
        //     } else {
        //         list.push(empty_buf());
        //     }

        //     return SegList::many(list)
        // }

        // return SegList::one(empty_buf())
    }

    // fn freeze_last(&mut self) -> Option<(BufItem, bool)> { 

    //     let item = self.last_mut();
    //     if item.len() > 0 {
    //         if item.len() < item.capacity() {
    //             let other = item.split();
    //             Some((other, false))
                
    //         } else { 
    //             let new_item = self.alloc.alloc_cap_seg();
    //             let last = std::mem::replace(self.last_mut(), new_item);
    //             Some((last, true))
    //         }
    //     } else {
    //         None
    //     }

    // }


    fn alloc_seg(&mut self) { 
        let seg = self.alloc.alloc_cap_seg();
        self.segs.push(seg);
    }

    fn last_mut(&mut self) -> &mut BufItem {
        let index = self.segs.len() - 1;
        &mut self.segs[index]
    }

    fn last(&self) -> &BufItem {
        let index = self.segs.len() - 1;
        &self.segs[index]
    }
}

fn split_last<A: AllocSeg>(last: &mut BufItem, alloc: &mut A) -> Option<(BufItem, bool)> {
    if last.len() > 0 {
        return  if last.len() < last.capacity() {
            let other = last.split();
            Some((other, false))
            
        } else { 
            let new_item = alloc.alloc_cap_seg();
            let last = std::mem::replace(last, new_item);
            Some((last, true))
        }
    }
    None 
}


unsafe impl<A: AllocSeg> BufMut for SegBuf<A> {
    fn remaining_mut(&self) -> usize {
        usize::MAX - self.len
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.len += cnt;
        {
            let last = self.last_mut();
            last.advance_mut(cnt);
            if last.capacity() == last.len() {
                self.alloc_seg();
            }
        }
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice { 
        {
            let last = self.last_mut();
    
            if last.capacity() == last.len() {
                self.alloc_seg();
            }
        }
        
        self.last_mut().chunk_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    index: usize,
    pos: usize,
    offset: usize,
}

pub struct WBuf<'a> {
    segs: &'a mut [BufItem],
    index: usize,
    pos: usize,
    remaining: usize,
}

impl<'a> WBuf<'a> {
    fn current_seg(&self) -> &BufItem {
        &self.segs[self.index]
    }

    fn current_seg_mut(&mut self) -> &mut BufItem {
        &mut self.segs[self.index]
    }

    fn current_remains(&self) -> usize {
        self.current_seg().len() - self.pos
    }

    fn move_cursor_to_next_seg(&mut self) {
        self.index += 1;
        self.pos = 0;
    }
}

unsafe impl<'a> BufMut for WBuf<'a> {
    fn remaining_mut(&self) -> usize {
        self.remaining
    }

    unsafe fn advance_mut(&mut self, mut cnt: usize) {
        self.remaining -= cnt;

        while cnt > 0  {
            assert!(self.index < self.segs.len());

            let len = cnt.min(self.current_remains());

            self.pos += len;
            cnt -= len;

            if self.current_remains() == 0 {
                self.move_cursor_to_next_seg();
            }
        }
        
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice { 

        if self.pos == self.current_seg().len() { 
            self.move_cursor_to_next_seg();
        }

        let pos = self.pos;
        let seg = self.current_seg_mut();
        let len = seg.len();
        
        let data = seg.as_mut();
        let ptr = data.as_mut_ptr();

        unsafe { &mut UninitSlice::from_raw_parts_mut(ptr, len)[pos..] }

    }
}


pub struct RBuf<'a> {
    segs: &'a [BufItem],
    index: usize,
    pos: usize,
    remaining: usize,
}

impl<'a> RBuf<'a> {
    fn current_seg(&self) -> &BufItem {
        &self.segs[self.index]
    }

    fn current_remains(&self) -> usize {
        self.current_seg().len() - self.pos
    }

    fn move_cursor_to_next_seg(&mut self) {
        self.index += 1;
        self.pos = 0;
    }
}

impl<'a> Buf for RBuf<'a> {
    fn remaining(&self) -> usize {
        self.remaining
    }

    fn chunk(&self) -> &[u8] {
        &self.current_seg()[self.pos..]
    }

    fn advance(&mut self, mut cnt: usize) {
        self.remaining -= cnt;

        while cnt > 0  {
            assert!(self.index < self.segs.len());

            let len = cnt.min(self.current_remains());

            self.pos += len;
            cnt -= len;

            if self.current_remains() == 0 {
                self.move_cursor_to_next_seg();
            }
        }
    }
}



// pub struct CBuf<'a, A: AllocSeg> {
//     owner: &'a mut SegBuf<A>,
//     index: usize,
//     pos: usize,
//     remaining_mut: usize,
// }

// impl<'a, A: AllocSeg> CBuf<'a, A> {
//     fn current_seg(&self) -> &BufItem {
//         &self.owner.segs[self.index]
//     }

//     fn current_seg_mut(&mut self) -> &mut BufItem {
//         &mut self.owner.segs[self.index]
//     }

//     fn current_remains(&self) -> usize {
//         self.current_seg().len() - self.pos
//     }

//     fn move_cursor_to_next_seg(&mut self) {
//         self.index += 1;
//         self.pos = 0;
//     }
// }

// unsafe impl<'a, A: AllocSeg> BufMut for CBuf<'a, A> {
//     fn remaining_mut(&self) -> usize {
//         self.remaining_mut
//     }

//     unsafe fn advance_mut(&mut self, mut cnt: usize) {
//         self.remaining_mut -= cnt;

//         while cnt > 0  {
//             assert!(self.index < self.owner.segs.len());

//             let len = cnt.min(self.current_remains());

//             self.pos += len;
//             cnt -= len;

//             if self.current_remains() == 0 {
//                 self.move_cursor_to_next_seg();
//             }
//         }
        
//     }

//     fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice { 

//         if self.pos == self.current_seg().len() { 
//             self.move_cursor_to_next_seg();
//         }

//         let pos = self.pos;
//         let seg = self.current_seg_mut();
//         let len = seg.len();
        
//         let data = seg.as_mut();
//         let ptr = data.as_mut_ptr();
//         // println!("chunk_mut: pos {}, len {}, ptr {:?}", pos, len, ptr);
//         unsafe { &mut UninitSlice::from_raw_parts_mut(ptr, len)[pos..] }

//         // let ptr = seg.as_mut_ptr();
//         // unsafe { &mut UninitSlice::from_raw_parts_mut(ptr, len)[pos..] }
//     }
// }


pub trait AllocSeg {
    fn alloc_cap_seg(&mut self) -> BufItem {
        BufItem::with_capacity(SEG_CAP)
    }
    // fn alloc_empty_seg(&mut self) -> BufItem;
}

impl AllocSeg for () {}


pub struct SegAllocator<const N: usize = SEG_CAP>;

impl<const N: usize> Default for SegAllocator<N> {
    fn default() -> Self {
        Self {  }
    }
}

impl<const N: usize> AllocSeg for SegAllocator<N> {
    fn alloc_cap_seg(&mut self) -> BufItem {
        BufItem::with_capacity(N)
    }
}

pub struct CapAllocator(pub usize);

impl AllocSeg for CapAllocator {
    fn alloc_cap_seg(&mut self) -> BufItem {
        BufItem::with_capacity(self.0)
    }
}


const SEG_CAP: usize = 4*1024;

fn empty_buf() -> BufItem {
    BufItem::new()
}

type BufItem = BytesMut;
type BufQueue = TinyVec<[BufItem; 1]>;
// type BufQueue = VecDeque<BufItem>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegList { 
    list: TinyVec<[BufItem; 1]>,
}

impl SegList { 
    
    pub fn len(&self) -> usize {
        let mut remaining = 0;
        for item in self.list.iter() {
            remaining += item.len();
        }
        remaining
    }

    pub fn as_buf(&self) -> RBuf { 
        RBuf { 
            segs: &self.list[..], 
            index: 0, 
            pos: 0, 
            remaining: self.len(), 
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_buf().into_vec()
    }

    pub fn into_parts(mut self) -> Option<(impl Iterator<Item = BufItem>, BufItem)> {
        match self.list.pop() {
            Some(last) => Some((self.list.into_iter(), last)),
            None => None,
        }
    }
}

impl SegList {

    fn one(item: BufItem) -> Self {
        SegList { list: TinyVec::Inline([item].into()) }
    }

    fn many( val : Vec<BufItem>) -> Self {
        SegList { list: TinyVec::Heap(val) }
    }

    // fn list( list : TinyVec<[BufItem; 1]>) -> Self {
    //     SegList { list }
    // }
}

pub trait InotToVec {
    fn into_vec(self) -> Vec<u8>;
}

impl<T: Buf> InotToVec for T {
    fn into_vec(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.remaining());
        let mut rbuf = self;
        while rbuf.has_remaining() {
            let chunk = rbuf.chunk();
            v.extend_from_slice(chunk);
            rbuf.advance(chunk.len());
        }
        v 
    }
}

#[cfg(test)]
impl SegList { 
    fn one_slice(value: &[u8]) -> Self {
        let mut item = BufItem::with_capacity(value.len());
        item.put_slice(value);
        Self::one(item)
    }

    fn last_slice(value: &[u8]) -> Self {
        let mut item = BufItem::with_capacity(value.len());
        item.put_slice(value);
        Self::many(vec![item, empty_buf()])
    }

    fn many_slices(value: &[&[u8]]) -> Self { 
        let mut list = Vec::with_capacity(value.len());
        for slice in value.iter() {
            let mut item = BufItem::with_capacity(slice.len());
            item.put_slice(slice);
            list.push(item);
        }
        
        Self::many(list)
    }
}


// type SegList2 = TinyVec<[Bytes; INLINE_LEN]>;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_seg_buf_basic() { 
        const CAP: usize = 10;
        let mut buf = SegBuf::with_alloc(SegAllocator::<CAP>);
        assert_eq!(buf.split(), SegList::one(empty_buf()));

        buf.put_u64(88);
        assert_eq!(buf.split(), SegList::one_slice(&88_u64.to_be_bytes()[..]) );

        buf.put_u16(99);
        assert_eq!(buf.split(), SegList::last_slice(&99_u16.to_be_bytes()[..]) );
        assert!(buf.last().is_empty());

        buf.put_u64(11);
        buf.put_u64(12);
        assert_eq!(buf.split(), SegList::many_slices(&[
            &[
                &11_u64.to_be_bytes()[..],
                &12_u64.to_be_bytes()[..2],
            ].concat(),
            &12_u64.to_be_bytes()[2..],
        ]) );

        buf.clear();

        buf.put_slice(&[1,2,3,4,5,6,7,]);
        buf.put_slice(&[11,12,13,14,15,16,17,]);
        let segs = buf.split();
        assert_eq!(segs, SegList::many_slices(&[
            &[1,2,3,4,5,6,7,11,12,13],
            &[14,15,16,17],
        ]) );
        let (mut iter, last) = segs.into_parts().unwrap();
        assert_eq!(iter.next().as_deref(), Some(&[1,2,3,4,5,6,7,11,12,13_u8][..]) );
        assert_eq!(iter.next(), None );
        assert_eq!(&last[..], &[14,15,16,17] );
        
        buf.clear();
        assert_eq!(buf.split(), SegList::one(empty_buf()));

        let full: Vec<u8> = (1..=CAP).map(|x|x as u8).collect();

        buf.put_slice(&full);
        assert_eq!(buf.split(), SegList::many_slices(&[
            &full,
            &[],
        ]) );

        buf.put_slice(&full);
        buf.put_slice(&full);
        assert_eq!(buf.split(), SegList::many_slices(&[
            &full,
            &full,
            &[],
        ]) );

        buf.put_slice(&[100, 101, 102]);
        buf.put_slice(&full[..CAP-3]);
        assert_eq!(buf.split(), SegList::many_slices(&[
            &[
                &[100, 101, 102][..],
                &full[..CAP-3],
            ].concat(),
            &[],
        ]) );


    }

    #[test]
    fn test_seg_buf_cursor() {
        let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
        buf.put_slice(&[1,2,3]);

        let cursor = buf.cursor();
        buf.put_slice(&[4,5,6,7,8,9,10,11]);
        {
            let mut cbuf = buf.at_cursor_mut(&cursor);
            cbuf.put_slice(&[34,35,36,37]);
        }

        assert_eq!(buf.split(), SegList::many_slices(&[
            &[1,2,3,34,35,36,37,8,9,10,],
            &[11],
        ]) );

        buf.clear();
        buf.put_slice(&[1,2,3]);
        
        let cursor = buf.cursor();
        buf.put_slice(&[4,5,6,7,8,9,10,11]);

        {
            let mut cbuf = buf.at_cursor_mut(&cursor);
            cbuf.put_slice(&[34,35,36,37,38,39,40,41]);
        }

        assert_eq!(buf.split(), SegList::many_slices(&[
            &[1,2,3,34,35,36,37,38,39,40],
            &[41],
        ]) );

    }

    #[test]
    fn test_seg_buf_rbuf() {
        let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
        buf.put_slice(&[1,2,3]);

        buf.put_slice(&[4,5,6,7,8,9,10,11]);

        {
            let data1 = buf.split();
            let mut rbuf1 = data1.as_buf();

            let data2 = &[1,2,3,4,5,6,7,8,9,10,11_u8];
            let mut rbuf2 = &data2[..]; 

            assert_eq!(rbuf1.remaining(), rbuf2.remaining());
            while rbuf1.remaining() > 0 {
                assert_eq!(rbuf1.get_u8(), rbuf2.get_u8(),);
            }
        }
    }

    #[test]
    fn test_seg_buf_put_limit() { 
        {
            let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
    
            let mut src = &[1,2,3,4,5,6,7,8,9,10,11_u8][..];
    
            buf.put_limit(&mut src, 5);
    
            assert_eq!(buf.split(), SegList::one_slice(&[1,2,3,4,5][..]) );
            assert_eq!(src, &[6,7,8,9,10,11_u8]);
        }

        {
            let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
    
            let mut src = &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24_u8][..];
    
            buf.put_limit(&mut src, 12);
    
            assert_eq!(buf.split(), SegList::many_slices(&[
                &[1,2,3,4,5,6,7,8,9,10][..],
                &[11,12,][..]
            ]));
            assert_eq!(src, &[13,14,15,16,17,18,19,20,21,22,23,24_u8]);
        }

        {
            let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
    
            let mut src = &[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24_u8][..];
    
            buf.put_limit(&mut src, 22);
    
            assert_eq!(buf.split(), SegList::many_slices(&[
                &[1,2,3,4,5,6,7,8,9,10][..],
                &[11,12,13,14,15,16,17,18,19,20][..],
                &[21,22][..],
            ]));
            assert_eq!(src, &[23,24_u8]);
        }

        {
            let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
            buf.put_slice(&[1,2,3]);
    
            let mut src = &[4,5,6,7,8,9,10,11_u8][..];
    
            buf.put_limit(&mut src, 5);
    
            assert_eq!(src, &[9,10,11_u8]);
            assert_eq!(buf.split(), SegList::one_slice(&[1,2,3,4,5,6,7,8][..]) );
        }

        {
            let mut buf = SegBuf::with_alloc(SegAllocator::<10>);
            buf.put_slice(&[1,2,3]);
    
            let mut src = &[4,5,6,7,8,9_u8][..];
    
            buf.put_limit(&mut src, 15);
    
            assert_eq!(src, &[0_u8][..0]);
            assert_eq!(buf.split(), SegList::one_slice(&[1,2,3,4,5,6,7,8,9][..]) );
        }

    }
}



// #[cfg(test)]
// mod test_buf_mut {
//     use bytes::{BufMut, Buf, BytesMut, Bytes};
//     use tinyvec::TinyVec;

//     #[test]
//     fn test_buf_mut() {
//         // let buf = bytes::BytesMut::new().freeze();
//         let cap = 1024;
//         let mut buf = BytesMut::with_capacity(cap);
//         println!("after new: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());

//         buf.put_u64(1);
//         println!("after put_u64 1: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());
        
//         let v1 = (&buf[..8]).get_u64();
//         (&mut buf[..8]).put_u64(2);
//         let v2 = (&buf[..8]).get_u64();
//         println!("after get/put u64 2: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());
//         println!("v1 {}, v2 {}", v1, v2);

//         let other = buf.split();
//         println!("after split: len {}, cap {}", buf.len(), buf.capacity());
//         println!("other: len {}, cap {}", other.len(), other.capacity());

//         let other = other.freeze();
//         println!("other freeze: len {}", other.len());
        
//         println!("after freeze: len {}, cap {}", buf.len(), buf.capacity());

//         buf.put_bytes(0, buf.capacity()/3);
//         println!("after put_bytes 1/3: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());

//         buf.split().freeze();
//         println!("after freeze2: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());

//         drop(other);
//         println!("after drop other: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());

//         buf.put_bytes(0, buf.capacity());
//         println!("after put_bytes full: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());
        
//         buf.split().freeze();
//         println!("after freeze3: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());
//         let last_ptr = buf.as_ptr();

//         buf.put_u64(2);
//         println!("after put_u64 2: len {}, cap {}, ptr {:?}", buf.len(), buf.capacity(), buf.as_ptr());
//         let first_ptr = buf.as_ptr();
//         let r = unsafe{ first_ptr.offset(cap as isize) };
//         assert_eq!(r, last_ptr);
        
//         println!("Bytes size {}", std::mem::size_of::<Bytes>());
//         println!("BytesMut size {}", std::mem::size_of::<BytesMut>());
//         println!("Vec<u8> size {}", std::mem::size_of::<Vec<u8>>());
//         println!("Vec<64> size {}", std::mem::size_of::<Vec<u64>>());
//         println!("TinyVec<[BytesMut;1]> size {}", std::mem::size_of::<TinyVec<[BytesMut; 1]>>());
//         println!("TinyVec<[BytesMut;2]> size {}", std::mem::size_of::<TinyVec<[BytesMut; 2]>>());
        
//         #[derive(Debug)]
//         enum BytesMutList { 
//             _One(BytesMut),
//             _Last(BytesMut),
//             _Many(Vec<BytesMut>),
//         }

//         println!("BytesMutList size {}", std::mem::size_of::<BytesMutList>());

//     }
// }



