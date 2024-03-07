use anyhow::Result;
use std::{marker::PhantomData, num::NonZeroU16};

pub struct RtpDepackerAAC(PhantomData<()>);
impl RtpDepackerAAC {
    pub fn new(
        clock_rate: u32,
        channels: Option<NonZeroU16>,
        fmtp: Option<&str>,
    ) -> Result<super::super::retina::depack::RetinaDepackAAC> {
        super::super::retina::depack::make_retina_depack_aac(clock_rate, channels, fmtp)
    }
}

