
// use anyhow::Result;
// use bytes::Bytes;

// pub trait RtpCodecDepacker {
//     fn push_rtp_slice(&mut self, rtp: &[u8]) -> Result<()> ;
//     fn pull_frame(&mut self) -> Result<Option<Bytes>> ;
// }


pub fn check_is_rtcp(data: &[u8]) -> bool {
    // refer from rtc-xswitch
    // Check the RTP payload type.  If 63 < payload type < 96, it's RTCP.
    // For additional details, see http://tools.ietf.org/html/rfc5761.
    // https://blog.csdn.net/ciengwu/article/details/78024121#:~:text=rtp%E4%B8%8Ertcp%E5%8D%8F%E8%AE%AE%E5%A4%B4,%E5%B0%B1%E5%8F%98%E4%B8%BA%E4%BA%8672~78%E3%80%82

    if data.len() < 2 {
        return false;
    }
    let pt = data[1] & 0x7F;
    return (63 < pt) && (pt < 96);
}
