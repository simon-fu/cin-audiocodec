
pub mod input;

pub mod resampler;

pub mod swr;

pub fn audio_packed_i16_format() -> ffmpeg_next::format::Sample {
    ffmpeg_next::format::Sample::I16(ffmpeg_next::format::sample::Type::Packed)
} 

pub fn audio_planar_f32_format() -> ffmpeg_next::format::Sample {
    ffmpeg_next::format::Sample::F32(ffmpeg_next::format::sample::Type::Planar)
}
