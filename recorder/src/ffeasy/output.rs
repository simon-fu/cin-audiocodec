
use std::path::Path;

use ff::Rescale;
use ffmpeg_next as ff;

use super::parameters::FFParameters;

pub struct FFOutput {
    output: ff::format::context::output::Output,
}

impl FFOutput {
    pub fn open<P: AsRef<Path>>(path: &P) -> Result<Self, ff::Error> {
        let output = ff::format::output(path)?;
        Ok(Self {
            output,
        })
    }

    pub fn has_global_header(&self) -> bool {
        self.output
        .format()
        .flags()
        .contains(ff::format::Flags::GLOBAL_HEADER)
    }

    pub fn add_track<'a>(
        &'a mut self, 
        codec: Option<ff::codec::codec::Codec>,
        param: ff::codec::Parameters
    ) -> Result<ff::format::stream::StreamMut<'a> , ff::Error> {

        let mut stream = self.output.add_stream(codec)?;

        stream.set_parameters(param);

        Ok(stream)
    }

    pub fn add_aac_track<'a>(
        &'a mut self, 
        samplerate: i32,
        channels: i32,
    ) -> Result<ff::format::stream::StreamMut<'a> , ff::Error> {
        self.add_audio_track(ff::codec::Id::AAC, samplerate, channels, Some(1024))
    }

    pub fn add_audio_track<'a>(
        &'a mut self, 
        codec_id: ff::codec::Id,
        samplerate: i32,
        channels: i32,
        frame_size: Option<i32>,
    ) -> Result<ff::format::stream::StreamMut<'a> , ff::Error> {
        let mut o_param = FFParameters::new();
        o_param.set_audio(codec_id.into(), samplerate, channels);

        if let Some(frame_size) = frame_size {
            o_param.set_frame_size(frame_size);
        }

        let codec = ff::encoder::find(codec_id);
        self.add_track(codec, o_param.into())
    }

    pub fn add_h264_track<'a>(
        &'a mut self, 
        width: i32,
        height: i32,
        spspps: &[u8],
        // fps_time_base: ff::Rational,
    ) -> Result<ff::format::stream::StreamMut<'a> , ff::Error> {

        self.add_video_track(
            ff::codec::Id::H264, 
            width, 
            height, 
            spspps, 
            // fps_time_base,
        )
    }

    pub fn add_video_track<'a>(
        &'a mut self, 
        codec_id: ff::codec::Id,
        width: i32,
        height: i32,
        extra: &[u8],
        // fps_time_base: ff::Rational,
    ) -> Result<ff::format::stream::StreamMut<'a> , ff::Error> {

        let mut o_param = FFParameters::new();
        
        o_param.set_video(
            codec_id.into(), 
            width, 
            height, 
            extra,
        );
        
        // o_param.set_framerate(fps_time_base.into());

        let codec = ff::encoder::find(codec_id);
        let mut o_track = self.add_track(codec, o_param.into())?;
        o_track.set_time_base( ff::Rational::new(1, 90000) );
        
        Ok(o_track)
    }

    pub fn begin_write(mut self) -> Result<FFWriter, ff::Error> {
        self.output.write_header()?;
        Ok(FFWriter {
            output: self.output,
            wrote_trailer: false,
        })
    }

}

pub struct FFWriter {
    output: ff::format::context::output::Output,
    wrote_trailer: bool,
}

impl FFWriter {

    pub fn num_tracks(&self) -> usize {
        self.output.nb_streams() as usize
    }

    pub fn tracks_iter<'a>(&'a self) -> impl Iterator<Item = FFTrack> + 'a {
        self.output.streams().map(|x|FFTrack::from(x))
    }

    pub fn get_track(&self, index: usize) -> Option<FFTrack> {
        self.output.stream(index)
        .map(|x|x.into())
    }

    pub fn write_packet(
        &mut self, 
        track: &FFTrack,
        src_time_base: ff::Rational,
        packet: &mut ff::Packet,

    ) -> Result<(), ff::Error> {

        track.fill_packet(src_time_base, packet);
        packet.write(self.inner_mut())?;
        Ok(())
    }

    pub fn write_packet_interleaved(
        &mut self, 
        track: &FFTrack,
        src_time_base: ff::Rational,
        packet: &mut ff::Packet,
    ) -> Result<(), ff::Error> {

        track.fill_packet(src_time_base, packet);
        packet.write_interleaved(self.inner_mut())?;
        Ok(())
    }

    pub fn write_trailer(&mut self) -> Result<(), ff::Error> {
        if !self.wrote_trailer {
            self.wrote_trailer = true;
            self.output.write_trailer()?;
        }
        Ok(())
    }

    // pub fn inner(&self) -> &ff::format::context::output::Output {
    //     &self.output
    // }

    fn inner_mut(&mut self) -> &mut ff::format::context::output::Output {
        &mut self.output
    }
}

impl Drop for FFWriter {
    fn drop(&mut self) {
        let _r = self.write_trailer();
    }
}


pub fn rescal_packet_ts(packet: &mut ff::Packet, source: ff::Rational, destination: ff::Rational) {
    let pts = packet.pts().map(|ts| ts.rescale(source, destination));
    packet.set_pts(pts);

    let dts = packet.dts().map(|ts| ts.rescale(source, destination));
    packet.set_dts(dts);

    // 用 packet.rescale_ts 会导致 裸h264 转 mp4 时， ffplay 播放 mp4  显示出来的帧率不准
    // packet.rescale_ts(source, destination);

}

#[derive(Debug, Clone)]
pub struct FFTrack {
    pub index: usize,
    pub time_base: ff::Rational,
}

impl FFTrack {
    fn fill_packet(&self, src_time_base: ff::Rational, packet: &mut ff::Packet,) {
        packet.set_stream(self.index);
        packet.set_position(-1);

        rescal_packet_ts(packet, src_time_base, self.time_base);

        // packet.rescale_ts(src_time_base, track.time_base);
    }
}

impl<'a> From<&ff::format::stream::StreamMut<'a>> for FFTrack {
    fn from(track: &ff::format::stream::StreamMut<'a>) -> Self {
        Self { index: track.index(), time_base: track.time_base() }
    }
}

impl<'a> From<&ff::format::stream::Stream<'a>> for FFTrack {
    fn from(track: &ff::format::stream::Stream<'a>) -> Self {
        Self { index: track.index(), time_base: track.time_base() }
    }
}

impl<'a> From<ff::format::stream::Stream<'a>> for FFTrack {
    fn from(track: ff::format::stream::Stream<'a>) -> Self {
        Self { index: track.index(), time_base: track.time_base() }
    }
}
