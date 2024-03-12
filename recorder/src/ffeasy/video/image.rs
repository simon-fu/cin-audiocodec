use ffmpeg_next as ff;

use super::{Point, VideoSize, YuvColor};





pub struct FFYuvImage {
    // size: Size,
    pub(super) frame: ff::util::frame::Video,
    // bg_color: YuvColor, // background color
}

impl FFYuvImage {

    pub const FORMAT: ff::util::format::Pixel = ff::util::format::Pixel::YUV420P;

    pub fn new(size: VideoSize) -> Self {

        let me = Self {
            frame: new_image(size),
            // size,
            // bg_color: YuvColor::BLACK,
        };
        // me.draw_bk();

        me
    }

    pub fn size(&self) -> VideoSize {
        VideoSize {
            width: self.frame.width() ,
            height: self.frame.height() ,
        }
    }

    pub fn frame(&self) -> &ff::util::frame::Video {
        &self.frame
    }
    
    // pub fn bk_pixel(&self) -> &YuvColor {
    //     &self.bg_color
    // }

    pub fn fill_color(&mut self, color: &YuvColor) {
        fill_frame_color(&mut self.frame, color);
    }

    pub fn draw_images_iter<'a, I>(&mut self, channels: I) 
    where
        I: Iterator<Item = (Point, &'a ff::util::frame::Video)>,
    {
        for (pos, frame) in channels {
            draw_yuv_frame(frame, &mut self.frame, pos.x, pos.y);
        }
    }

    pub fn draw_image(&mut self, at: Point, image: &FFYuvImage) {
        draw_yuv_frame(&image.frame, &mut self.frame, at.x, at.y);
    }

    pub fn output(&self) -> &ff::util::frame::Video {
        &self.frame
    }

}

impl From<ff::util::frame::Video> for FFYuvImage {
    fn from(value: ff::util::frame::Video) -> Self {
        Self {
            frame: value,
        }
    }
}


fn new_image(size: VideoSize) -> ff::util::frame::Video {
    ff::util::frame::Video::new(
        ff::util::format::Pixel::YUV420P, 
        size.width, 
        size.height,
    )
}

fn fill_frame_color(frame: &mut ff::util::frame::Video, color: &YuvColor) {
    frame.data_mut(0).fill(color.y);
    frame.data_mut(1).fill(color.u);
    frame.data_mut(2).fill(color.v);
}



// from https://blog.csdn.net/zwz1984/article/details/50403150
fn draw_yuv_frame(
    src: &ff::util::frame::Video,
    dst: &mut ff::util::frame::Video,
    off_x: u32,
    off_y: u32,
)
{
    let off_x = off_x as usize;
    let off_y = off_y as usize;

	// if (NULL == src || NULL == dst) return;
	// if (src->w > dst->w || src->h > dst->h) return;
	// if (NULL == src->y || NULL == src->u || NULL == src->v) return;
	// if (NULL == dst->y || NULL == dst->u || NULL == dst->v) return;
 
	// UINT nOff = 0;
    // let mut noff = 0_usize;

    
    let src_y = src.data(0);
    let src_w = src.width() as usize;
    let src_h = src.height() as usize;

    let dst_w = dst.width() as usize;
    let dst_y = dst.data_mut(0);

    // for (int i = 0; i < src->h; i++)
    for i in 0..src_h {
        // nOff = dst->w * (nOffY + i) + nOffX;
		let noff = dst_w * (off_y + i) + off_x;

		// copy each line
		// memcpy(dst->y + nOff, src->y + src->w*i, src->w);
        dst_y[noff..noff + src_w].clone_from_slice(&src_y[src_w*i..src_w*i+src_w]);
    }
	

	// UINT nUVOffX = nOffX / 2, nUVOffY = nOffY / 2;
	// UINT nUVSrcW = src->w / 2, nUVSrcH = src->h / 2;
	// UINT nUVDstW = dst->w / 2, nUVDstH = dst->h / 2;
    let uv_off_x = off_x / 2;
    let uv_off_y = off_y / 2;
    let uv_src_w = src_w / 2;
    let uv_src_h = src_h / 2;
    let uv_dst_w = (dst.width() / 2) as usize;
    // let uv_dst_h = dst.h / 2;
 
    
	// for (int j = 0; j < nUVSrcH; j++)
    for j in 0..uv_src_h {
		// nOff = nUVDstW*(nUVOffY + j) + nUVOffX;
        // memcpy(dst->u + nOff, src->u + nUVSrcW*j, nUVSrcW);
		// memcpy(dst->v + nOff, src->v + nUVSrcW*j, nUVSrcW);

        let noff = uv_dst_w * (uv_off_y + j) + uv_off_x;
		dst.data_mut(1)[noff..noff+uv_src_w].copy_from_slice(&src.data(1)[uv_src_w*j..uv_src_w*j+uv_src_w]);
        dst.data_mut(2)[noff..noff+uv_src_w].copy_from_slice(&src.data(2)[uv_src_w*j..uv_src_w*j+uv_src_w]);
	}
}
