use std::collections::BTreeMap;


pub struct RtpInorderBuf<T> {
    buf: BTreeMap<i64, T>,
    capacity: usize,
    seq_ext: U16Extender,
    seq_next: Option<i64>,
}

impl<T>  RtpInorderBuf<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: BTreeMap::new(),
            capacity,
            seq_ext: U16Extender::new(),
            seq_next: None,
        }
    }

    pub fn pushpop(&mut self, seq: u16,  data: T) -> Option<(RtpOrder, T)> {
        let seq64 = self.seq_ext.convert(seq);

        match self.seq_next {
            Some(seq_next) => {
                if seq64 < seq_next {
                    // Duplicate 
                    return None
                }

                if seq64 == seq_next {
                    self.seq_next = Some(seq_next + 1);
                    return Some((RtpOrder::Normal(seq64), data))
                }

                self.buf.insert(seq64, data);
            },
            None => {
                self.buf.insert(seq64, data);
            },
        }
        
        if self.buf.len() >= self.capacity {
            self.pop()
        } else {
            None
        }
    }

    pub fn inorder_pop(&mut self) -> Option<(i64, T)> {
        if let Some(first) = self.buf.first_key_value() {
            let seq = *first.0;
            let order = self.check_order(seq);
            if let Some(RtpOrder::Normal(v)) = order {
                self.seq_next = Some(v + 1);
                return self.buf.pop_first();
            }
        }
        None
    }

    pub fn pop(&mut self) -> Option<(RtpOrder, T)> {
        match self.buf.pop_first() {
            Some(first) => {
                let seq = first.0;
                let order = self.check_order(seq);
                self.seq_next = Some(seq + 1);
                match order {
                    Some(order) => Some((order, first.1)),
                    None => Some((RtpOrder::Normal(seq), first.1)),
                }
            },
            None => None,
        }
    }

    fn check_order(&self, seq: i64) -> Option<RtpOrder> {
        match self.seq_next {
            Some(seq_next) => {
                if seq == seq_next {
                    Some(RtpOrder::Normal(seq))
                } else {
                    let lost = (seq - seq_next) as u16;
                    Some(RtpOrder::Lost(lost, seq))
                }
            },
            None => None,
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RtpOrder {
    Normal(i64),
    Lost(u16, i64),
    Old,
}

#[test]
fn test_rtp_order_buf() {
    let mut buf = RtpInorderBuf::new(3);
    assert_eq!(buf.pushpop(1, ()), None );
    assert_eq!(buf.pushpop(2, ()), None );
    assert_eq!(buf.pushpop(3, ()), Some((RtpOrder::Normal(1), ())) );
    assert_eq!(buf.pushpop(4, ()), Some((RtpOrder::Normal(2), ())) );
    assert_eq!(buf.pop(), Some((RtpOrder::Normal(3), ())) );
    assert_eq!(buf.pop(), Some((RtpOrder::Normal(4), ())) );
    assert_eq!(buf.pop(), None);

    let mut buf = RtpInorderBuf::new(5);
    assert_eq!(buf.pushpop(1, ()), None );
    assert_eq!(buf.pushpop(3, ()), None );
    assert_eq!(buf.pushpop(4, ()), None );
    assert_eq!(buf.pushpop(5, ()), None );
    assert_eq!(buf.pushpop(6, ()), Some((RtpOrder::Normal(1), ())) );
    assert_eq!(buf.pushpop(7, ()), Some((RtpOrder::Lost(1, 3), ())) );
    assert_eq!(buf.pushpop(8, ()), Some((RtpOrder::Normal(4), ())) );
    assert_eq!(buf.pushpop(9, ()), Some((RtpOrder::Normal(5), ())) );
    assert_eq!(buf.pushpop(10, ()), Some((RtpOrder::Normal(6), ())) );
    assert_eq!(buf.pushpop(2, ()), None );
    assert_eq!(buf.pushpop(15, ()), Some((RtpOrder::Normal(7), ())) );
    assert_eq!(buf.pushpop(16, ()), Some((RtpOrder::Normal(8), ())) );
    assert_eq!(buf.pushpop(17, ()), Some((RtpOrder::Normal(9), ())) );
    assert_eq!(buf.pushpop(18, ()), Some((RtpOrder::Normal(10), ())) );
    assert_eq!(buf.pushpop(19, ()), Some((RtpOrder::Lost(4, 15), ())) );
    assert_eq!(buf.pushpop(22, ()), Some((RtpOrder::Normal(16), ())) );
    assert_eq!(buf.inorder_pop(), Some((17, ())) );
    assert_eq!(buf.inorder_pop(), Some((18, ())) );
    assert_eq!(buf.inorder_pop(), Some((19, ())) );
    assert_eq!(buf.inorder_pop(), None );
    assert_eq!(buf.pop(), Some((RtpOrder::Lost(2, 22), ())) );
    assert_eq!(buf.pop(), None );
}

#[derive(Debug, Clone, Copy)]
pub struct U16Extender {
    last_input: Option<u16>,
    ext: i64,
}

impl U16Extender {
    pub fn new() -> Self {
        Self {
            last_input: None,
            ext:0,
        }
    }

    pub fn convert(&mut self, val: u16) -> i64 {
        match self.last_input {
            Some(last_input) => {
                self.ext = u16_extend(last_input, val, self.ext);
                self.last_input = Some(val);
            },
            None => {
                self.last_input = Some(val);
                self.ext = val as i64;
            },
        }
        self.ext
    }
}

fn u16_extend(last: u16, curr: u16, ext: i64) -> i64 {
    let d1 = curr.wrapping_sub(last);
    let d2 = last.wrapping_sub(curr);
    if d1 <= d2 { 
        // d1==d2 when d1 is 32768
        ext + (d1 as i64)
    } else {
        ext - (d2 as i64)
    }
}



#[test]
fn test_extend() {
    assert_eq!(u16_extend(0, 1, 10), 11_i64);
    assert_eq!(u16_extend(0, 5, 10), 15_i64);
    assert_eq!(u16_extend(0, 32768, 10), 10_i64+32768);
    assert_eq!(u16_extend(0, 32769, 10), 10_i64.wrapping_sub(32767) );
    assert_eq!(u16_extend(0, 65535, 10), 10_i64.wrapping_sub(1) );
    assert_eq!(u16_extend(65535, 0, 10), 10_i64 + 1 );
}

