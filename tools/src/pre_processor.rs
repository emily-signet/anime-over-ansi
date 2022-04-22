use ansi_lib::metadata::*;

use color_lib::pattern;
use fast_image_resize as fr;
use image::{buffer::ConvertBuffer, imageops, RgbImage, RgbaImage};

use std::collections::HashSet;

use std::num::NonZeroU32;


/// A processor that resizes and dithers images as needed.
pub struct ProcessorPipeline {
    pub filter: fr::FilterType,
    pub width: u32,
    pub height: u32,
    pub dither_modes: HashSet<DitherMode>,
}

impl ProcessorPipeline {
    /// Process an image, returning a vector with resized versions of it in every color mode requested.
    pub fn process(&self, img: &RgbaImage) -> Vec<(DitherMode, RgbImage)> {
        let src_image = fr::Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            img.clone().into_raw(),
            fr::PixelType::U8x4,
        )
        .unwrap();

        let mut dst_image = fr::Image::new(
            NonZeroU32::new(self.width).unwrap(),
            NonZeroU32::new(self.height).unwrap(),
            src_image.pixel_type(),
        );
        let mut dst_view = dst_image.view_mut();
        let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(self.filter));
        resizer.resize(&src_image.view(), &mut dst_view).unwrap();

        let frame: RgbImage =
            RgbaImage::from_raw(self.width, self.height, dst_image.buffer().to_vec())
                .unwrap()
                .convert();

        let mut res = Vec::with_capacity(self.dither_modes.len());

        for mode in &self.dither_modes {
            match *mode {
                DitherMode::FloydSteinberg(map) => {
                    let mut dframe = frame.clone();
                    imageops::dither(&mut dframe, &map);
                    res.push((*mode, dframe));
                }
                DitherMode::Pattern(map, size, multiplier) => {
                    let mut dframe = frame.clone();
                    pattern::dither(&mut dframe, size, multiplier as f32 / 10_000.0, map);
                    res.push((*mode, dframe));
                }
                DitherMode::None => {
                    res.push((*mode, frame.clone()));
                }
            }
        }

        res
    }
}
