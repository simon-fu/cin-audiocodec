use std::{fs::File, io::Write, path::Path};
use anyhow::{Context, Result};

use bytes::Buf;


use crate::{tlv2::{tag_buf::TagBuf, Type}, tlv_custom::{FileInfoRef, Muxer, TlvType, TLV_MAGIC}};

use super::ChInfo;


pub struct TlvCustomFileWriter {
    ofile: File,
    muxer: Muxer,
    buf: TagBuf,
}

impl TlvCustomFileWriter {
    pub fn open(output: &Path) -> Result<Self> {
        let ofile = File::create(&output)
            .with_context(||format!("failed open [{output:?}]"))?;

        // const DEFAULT_BUF_SIZE: usize = 8*1024;
        // pub type Allocator = SegAllocator::<DEFAULT_BUF_SIZE>;
        // let mut buf = TagBuf::with_alloc(Allocator{});

        Ok(Self {
            ofile,
            muxer: Muxer::new(),
            buf: TagBuf::new(),
        })
    }

    pub fn write_header(&mut self) -> Result<()> {
        self.muxer.mux_file_header(&mut self.buf, &FileInfoRef {
            magic: TLV_MAGIC,
            desc: None,
        });
    
        self.write_to_file()?;

        Ok(())
    }

    pub fn write_adding_ch(&mut self, info: &ChInfo) -> Result<()> {
        let content = serde_json::to_string(info)?;
        self.muxer.mux_string(&mut self.buf, TlvType::AddCh.into(), &content);
        self.write_to_file()?;
        Ok(())
    }

    pub fn write_ch_data(&mut self, ch_id: u64, data: &[u8]) -> Result<()> {
        self.muxer.mux_ch_data(&mut self.buf, ch_id, data);
        self.write_to_file()?;
        Ok(())
    }

    pub fn write_ch_data_with_ts(&mut self, ch_id: u64, data: &[u8], ts: i64) -> Result<()> {
        self.muxer.mux_ch_data_with_ts(&mut self.buf, ch_id, data, ts);
        self.write_to_file()?;
        Ok(())
    }

    pub fn write_file_end(&mut self) -> Result<()> {
        self.muxer.mux_string(&mut self.buf, Type::FILE_END, "tlv file end");
        self.write_to_file()?;
        Ok(())
    }

    fn write_to_file(&mut self) -> Result<()> {
        let list = self.buf.split();
        let mut rbuf = list.as_buf();
        while rbuf.remaining() > 0 {
            let chunk = rbuf.chunk();
            self.ofile.write_all(chunk)
                .with_context(||"write file failed")?;
            rbuf.advance(chunk.len());
        }

        Ok(())
    }
}
