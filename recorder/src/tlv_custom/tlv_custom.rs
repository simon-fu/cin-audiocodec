
use anyhow::{bail, Result};

use crate::tlv2;


pub const TLV_MAGIC: &str = "CINN-TLV";
const TLV_FIRST: u8 = tlv2::Type::CUSTOM_VALUE + 2;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TlvType {
    CUSTOM = tlv2::Type::CUSTOM_VALUE,
    AddRoom = TLV_FIRST,
    RemoveRoom,
    AddCh,
    RemoveCh,
    ChData,
    Max,
}

impl TlvType {
    pub fn type_iter() -> impl Iterator<Item = tlv2::Type> {
        let start = TLV_FIRST; // Self::UdpRecv as tlv2::TypeRaw;
        let end = Self::Max as tlv2::TypeRaw;
        (start..end).map(|x| tlv2::Type::new(x))
    }

    pub fn rtype(&self) -> tlv2::Type {
        tlv2::Type::new(*self as u8) 
    }
}

impl From<TlvType> for tlv2::Type {
    fn from(value: TlvType) -> Self {
        Self::new(value as u8) 
    }
}

impl TryFrom<tlv2::Type> for TlvType {
    type Error = anyhow::Error;

    fn try_from(v: tlv2::Type) -> Result<Self, Self::Error> {
        match v.value() {
            x if x == Self::AddRoom as tlv2::TypeRaw => Ok(Self::AddRoom),
            x if x == Self::RemoveRoom as tlv2::TypeRaw => Ok(Self::RemoveRoom),
            x if x == Self::AddCh as tlv2::TypeRaw => Ok(Self::AddCh),
            x if x == Self::RemoveCh as tlv2::TypeRaw => Ok(Self::RemoveCh),
            x if x == Self::ChData as tlv2::TypeRaw => Ok(Self::ChData),
            _ => bail!("invalid tts tlv type {:?}", v),
        }
    }
}






// const DEFAULT_BUF_SIZE: usize = 8*1024;
// pub type Allocator = SegAllocator::<DEFAULT_BUF_SIZE>;


// pub struct TlvFileWriter {
//     writer: BufWriter<fs::File>,
// }

// impl TlvFileWriter {
//     pub async fn open_with_header(path: &Path, magic: String, desc: Option<String>) -> Result<Self> {
//         let file = fs::File::create(path).await?;
//         let mut writer =  BufWriter::new(file);
//         Ok(Self {
//             writer,
//         })
//     }

// }
