
use std::{path::Path, io::SeekFrom};
use tokio::{fs::File, io::{AsyncReadExt, AsyncSeekExt}};
use anyhow::{Result, bail, Context};

use crate::tlv2::{Type, tag_value::TagRef, Header};

use super::VecBuf;

pub struct TlvFileListReader { 
    file_list: Vec<String>,
    magic: Option<String>,
    state: Option<TlvFileReader>,
}

impl TlvFileListReader { 
    pub fn new<I: Into<String>>(magic: Option<I>, mut file_list: Vec<String>,) -> Self { 
        let magic = magic.map(|x|x.into());
        file_list.reverse();
        Self { magic, file_list, state: None }
    }

    pub async fn read_tag<'a>(&mut self, buf: &'a mut VecBuf) -> Result<Option<TagRef<'a>>> {
        let r = self.read_next(buf).await?;
        Ok(r.map(move |(rtype, _len)| TagRef::new(rtype, buf.as_slice())))
    }

    pub async fn read_next(&mut self, buf: &mut VecBuf) -> Result<Option<(Type, usize)>> {
        if self.state.is_none() { 
            self.next_file().await?;
        }

        loop {
            match &mut self.state {
                Some(state) => { 
                    let r = state.read_next(buf).await;
                    if let Ok(r) = r {
                        return Ok(Some(r));
                    }
                    self.next_file().await?;
                },
    
                None => { return Ok(None) },
            }
        }
    }

    async fn next_file(&mut self) -> Result<()> { 
        self.state = match self.file_list.pop() {
            Some(path) => {
                let reader = TlvFileReader::open_with_magic(&path, self.magic.as_deref()).await?;
                tracing::debug!("opened tlv file [{}]", path);
                Some(reader)
            },
            None => None,
        };
        
        Ok(())
    }
}


#[derive(Debug)]
pub struct TlvFileReader {
    file: File,
    header: [u8; Header::SIZE],
}

impl TlvFileReader {
    pub async fn open_with_magic(path: impl AsRef<Path>, magic: Option<&str>) -> Result<Self> {
        let mut buf = VecBuf::default();
        let (mut self0, magic0, _desc0) = Self::open_with_buf(path, &mut buf).await?;

        if let Some(expect) = magic {
            if magic0 != expect {
                bail!("expect magic [{}] but [{:?}]", expect, magic0);
            }
        }
        
        self0.file.seek(SeekFrom::Start(0)).await?;

        Ok(self0)
    }

    pub async fn open_with_buf<'a>(path: impl AsRef<Path>, buf: &'a mut VecBuf) -> Result<(Self, &'a str, &'a str)> {
        let file = File::open(path.as_ref()).await
        .with_context(||format!("fail to open tlv file [{:?}]", path.as_ref()))?;
        // println!("opened tlv file [{:?}]", path.as_ref());

        let mut self0 = Self { 
            file, 
            header: [0; Header::SIZE],
        };
        
        // let mut buf = Vec::new();
        let tag = self0.read_tag(buf).await?;
        // let (magic0, desc0) = tag.as_str2()?;

        if tag.rtype() != Type::ATTACH_BEGIN {
            bail!("expect type ATTACH_BEGIN but [{:?}]", tag.rtype());
        }
        
        let (_ts, magic0, desc0) = tag.value().as_i64_str2()?;
        Ok((self0, magic0, desc0))

        // if let Some(expect) = magic {
        //     if magic0 != expect {
        //         bail!("expect magic [{}] but [{:?}]", expect, magic0);
        //     }
        // }

        // let magic = if magic.is_none() {
        //     Some(magic0.to_owned())
        // } else {
        //     None
        // };

        // let desc = if desc0.is_empty() {
        //     None
        // } else {
        //     Some(desc0.to_owned())
        // };

        // Ok((self0, magic, desc))
    }

    pub async fn read_next(&mut self, buf: &mut VecBuf) -> Result<(Type, usize)> {

        loop {
            let (rtype, len) = read_raw_type_len(&mut self.file, &mut self.header[..]).await?;

            buf.clear();
    
            read_raw_additional(&mut self.file, buf, len).await?;
            if rtype == Type::ATTACH_END {
                continue;
            }
            
            return Ok((rtype, len))
        }
    }

    pub async fn read_tag<'a>(&mut self, buf: &'a mut VecBuf) -> Result<TagRef<'a>> {
        let (rtype, _len) = self.read_next(buf).await?;
        Ok(TagRef::new(rtype, buf.as_slice()))
    }

}


#[derive(Debug)]
pub struct TlvFileSyncReader {
    file: std::fs::File,
    header: [u8; Header::SIZE],
}

impl TlvFileSyncReader {
    pub fn open_with_magic(path: impl AsRef<Path>, magic: Option<&str>) -> Result<Self> {
        let mut buf = VecBuf::default();
        let (mut self0, magic0, _desc0) = Self::open_with_buf(path, &mut buf)?;

        if let Some(expect) = magic {
            if magic0 != expect {
                bail!("expect magic [{}] but [{:?}]", expect, magic0);
            }
        }
        
        use std::io::Seek;
        self0.file.seek(SeekFrom::Start(0))?;

        Ok(self0)
    }

    pub fn open_with_buf<'a>(path: impl AsRef<Path>, buf: &'a mut VecBuf) -> Result<(Self, &'a str, &'a str)> {
        let file = std::fs::File::open(path.as_ref())
        .with_context(||format!("fail to open tlv file [{:?}]", path.as_ref()))?;
        // println!("opened tlv file [{:?}]", path.as_ref());

        let mut self0 = Self { 
            file, 
            header: [0; Header::SIZE],
        };
        
        // let mut buf = Vec::new();
        let tag = self0.read_tag(buf)?;
        // let (magic0, desc0) = tag.as_str2()?;

        if tag.rtype() != Type::ATTACH_BEGIN {
            bail!("expect type ATTACH_BEGIN but [{:?}]", tag.rtype());
        }
        
        let (_ts, magic0, desc0) = tag.value().as_i64_str2()?;
        Ok((self0, magic0, desc0))

        // if let Some(expect) = magic {
        //     if magic0 != expect {
        //         bail!("expect magic [{}] but [{:?}]", expect, magic0);
        //     }
        // }

        // let magic = if magic.is_none() {
        //     Some(magic0.to_owned())
        // } else {
        //     None
        // };

        // let desc = if desc0.is_empty() {
        //     None
        // } else {
        //     Some(desc0.to_owned())
        // };

        // Ok((self0, magic, desc))
    }

    pub fn read_next(&mut self, buf: &mut VecBuf) -> Result<(Type, usize)> {

        loop {
            let (rtype, len) = Self::read_raw_type_len(&mut self.file, &mut self.header[..])?;

            buf.clear();
    
            Self::read_raw_additional(&mut self.file, buf, len)?;
            if rtype == Type::ATTACH_END {
                continue;
            }
            
            return Ok((rtype, len))
        }
    }

    pub fn read_tag<'a>(&mut self, buf: &'a mut VecBuf) -> Result<TagRef<'a>> {
        let (rtype, _len) = self.read_next(buf)?;
        Ok(TagRef::new(rtype, buf.as_slice()))
    }

    fn read_raw_type_len(file: &mut std::fs::File, buf: &mut [u8]) -> Result<(Type, usize)> {
        use std::io::Read;
        file.read_exact(buf)?;
        Header::try_parse(buf).with_context(||"invalid tlv header")
    }

    fn read_raw_additional(file: &mut std::fs::File, buf: &mut VecBuf, additional: usize) -> Result<()> { 
        use std::io::Read;

        let mut spare = buf.spare_mut(additional);
        
        while !spare.is_full() { 
            let n = file.read(spare.buf())?;
            if n == 0 { 
                bail!("reach EOF")
            }
            spare.take_up(n);
        }
        Ok(())
    }

}


async fn read_raw_type_len(file: &mut File, buf: &mut [u8]) -> Result<(Type, usize)> {

    file.read_exact(buf).await?;

    Header::try_parse(buf).with_context(||"invalid tlv header")
    // let rtype = (&buf[0..2]).get_u16();
    // let len = (&buf[2..6]).get_u32() as usize;
    // Ok((Type::new(rtype), len))
}

async fn read_raw_additional(file: &mut File, buf: &mut VecBuf, additional: usize) -> Result<()> { 

    let mut spare = buf.spare_mut(additional);
    
    while !spare.is_full() { 
        let n = file.read(spare.buf()).await?;
        if n == 0 { 
            bail!("reach EOF")
        }
        spare.take_up(n);
    }
    Ok(())
}

