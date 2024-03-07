

use chrono::Local;

use crate::tlv2::{seg_buf::AllocSeg, tag_buf::{AppendToTagBuf, TagBuf, ValueAppender}, Type};


use super::TlvType;




pub struct Muxer {
    basetime: i64,
    baseid: u64,
}

impl Muxer {
    pub fn new() -> Self {
        Self {
            basetime: 0, // Local::now().timestamp_millis(),
            baseid: 0,
        }
    }

    pub fn mux_file_header<'a, A: AllocSeg>(&self, buf: &mut TagBuf<A>, info: &FileInfoRef<'a>) {
        
        buf.begin_tag(Type::ATTACH_BEGIN)
        .append_now_milli()
        .append_len_value(info.magic)
        .append_last(info.desc.unwrap_or(""));

        buf.begin_tag(Type::ATTACH_END)
        .finish();
    }

    pub fn mux_ch_data<'a, A: AllocSeg>(&self, buf: &mut TagBuf<A>, ch_id: u64, data: &[u8]) {
        let ts = Local::now().timestamp_millis() - self.basetime;
        self.mux_ch_data_with_ts(buf, ch_id, data, ts)
    }

    pub fn mux_ch_data_with_ts<'a, A: AllocSeg>(&self, buf: &mut TagBuf<A>, ch_id: u64, data: &[u8], ts: i64) {
        let delta_ts = ts - self.basetime;
        let delta_id = ch_id - self.baseid;

        buf.begin_tag(TlvType::ChData)
        .append_var_i64(delta_ts)
        .append_var_u64(delta_id)
        .append_last(data);
    }

    pub fn mux_string<'a, A: AllocSeg>(&self, buf: &mut TagBuf<A>, rtype: Type, content: &str) {
        let ts = Local::now().timestamp_millis() - self.basetime;
        let delta_ts = ts - self.basetime;

        buf.begin_tag(rtype)
        .append_var_i64(delta_ts)
        .append_last(content);
    }
    
}



pub struct FileInfoRef<'a> {
    pub magic: &'a str,
    pub desc: Option<&'a str>,
}

impl<'a, A: AllocSeg> AppendToTagBuf<A> for FileInfoRef<'a> {
    fn append_to_tag_buf(&self, buf: &mut TagBuf<A>) {
        buf.begin_tag(Type::ATTACH_BEGIN)
        .append_now_milli()
        .append_len_value(self.magic)
        .append_last(self.desc.unwrap_or(""));

        buf.begin_tag(Type::ATTACH_END)
        .finish();
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
