
use ff::Rescale;
use ffmpeg_next as ff;

pub struct FFTimeResaler {
    src: ff::Rational,
    dst: ff::Rational,
}

impl FFTimeResaler {
    pub fn new(src: ff::Rational, dst: ff::Rational) -> Self {
        Self { src, dst, }
    }

    pub fn rescale(&self, ts: i64) -> i64 {
        ts.rescale(self.src, self.dst)
    }

    pub fn rescale_packet(&self, packet: &mut ff::Packet) {
        let pts = packet.pts().map(|x|self.rescale(x));
        let dts = packet.dts().map(|x|self.rescale(x));

        packet.set_pts(pts);
        packet.set_dts(dts);
    }

    pub fn rescale_video(&self, packet: &mut ff::frame::Video) {
        let pts = packet.pts().map(|x|self.rescale(x));
        packet.set_pts(pts);
    }

    pub fn rescale_audio(&self, packet: &mut ff::frame::Audio) {
        let pts = packet.pts().map(|x|self.rescale(x));
        packet.set_pts(pts);
    }
}

pub trait ScaleTime {
    fn scale_time(&mut self, scaler: &FFTimeResaler);
}

impl ScaleTime for ff::Packet {
    fn scale_time(&mut self, scaler: &FFTimeResaler) {
        scaler.rescale_packet(self);
    }
}

impl ScaleTime for ff::frame::Video {
    fn scale_time(&mut self, scaler: &FFTimeResaler) {
        scaler.rescale_video(self);
    }
}

impl ScaleTime for ff::frame::Audio {
    fn scale_time(&mut self, scaler: &FFTimeResaler) {
        scaler.rescale_audio(self);
    }
}

