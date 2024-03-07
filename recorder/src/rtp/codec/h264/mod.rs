
use anyhow::Result;
use std::marker::PhantomData;

pub struct RtpDepackerH264(PhantomData<()>);
impl RtpDepackerH264 {
    pub fn new(fmtp: Option<&str>) -> Result<super::super::retina::depack::RetinaDepackH264> {
        super::super::retina::depack::make_retina_depack_h264(fmtp)
    }
}

mod parameters;
pub use parameters::*;




