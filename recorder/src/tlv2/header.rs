use bytes::Buf;

pub type TypeRaw = u8;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Type(TypeRaw);

impl Type {

    pub const MAGIC: Type = Type(1);
    pub const ATTACH_BEGIN: Type = Type(2);
    pub const ATTACH_END: Type = Type(3);
    pub const FILE_END: Type = Type(4);

    pub const BUILD_IN_START: Type = Self::DEBUG;
    pub const DEBUG: Type = Type(4);
    pub const BUILD_IN_END: Type = Type(5);

    pub fn build_in_iter() -> impl Iterator<Item = Type> {
        (Self::BUILD_IN_START.0..Self::BUILD_IN_END.0).map(|x| Self(x))
    }
    
    pub const CUSTOM_VALUE: u8 = 16;
    pub const CUSTOM: Type = Type(Self::CUSTOM_VALUE);
    
    pub fn new(v: TypeRaw) -> Self {
        Self(v)
    }

    pub fn value(&self) -> u8 {
        self.0 
    }

    pub fn is_build_in(&self) -> bool {
        self.value() < Self::CUSTOM_VALUE
    }

    pub fn is_debug_data(&self) -> bool {
        *self == Self::DEBUG
    }
    
}

impl From<Type> for usize {
    #[inline]
    fn from(rtype: Type) -> Self {
        rtype.0 as usize
    }
}


pub struct Header(pub Type, pub u32);

impl Header { 
    pub const SIZE: usize = 4;
    pub fn to_bytes(&self) -> [u8; Self::SIZE] { 
        let mut bytes = self.1.to_be_bytes();
        bytes[0] = self.0.value();
        bytes
    }

    pub fn try_parse(buf: &[u8]) -> Option<(Type, usize)> {
        if buf.len() >= Self::SIZE {
            let (rtype, len) = Self::parse_buf(buf);
            Some((rtype, len))
        } else {
            None
        }
    }
    
    pub fn parse_buf<B: Buf>(mut buf: B) -> (Type, usize) { 
        let tmp = buf.get_u32();
        let rtype = (tmp >> 24 & 0x00FF) as u8;
        let len = (tmp & 0x00FFFFFF) as usize;
        (Type::new(rtype), len)
    }
}



