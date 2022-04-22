use crate::metadata::*;
use color_lib::palette::*;
use image::{Rgb, RgbImage};

// A base trait for any ANSI image frame encoder, automatically implementing most of the encoding based on a few getter methods.
pub trait AnsiEncoder {
    fn color(&self, pixel: &Rgb<u8>, fg: bool) -> String {
        match self.needs_color() {
            ColorMode::EightBit => {
                if fg {
                    format!("\x1B[38;5;{}m", REVERSE_PALETTE[&pixel.0])
                } else {
                    format!("\x1B[48;5;{}m", REVERSE_PALETTE[&pixel.0])
                }
            }
            _ => {
                if fg {
                    format!(
                        "\x1B[38;2;{r};{g};{b}m",
                        r = pixel[0],
                        g = pixel[1],
                        b = pixel[2]
                    )
                } else {
                    format!(
                        "\x1B[48;2;{r};{g};{b}m",
                        r = pixel[0],
                        g = pixel[1],
                        b = pixel[2]
                    )
                }
            }
        }
    }

    fn encode_frame(&mut self, image: &RgbImage) -> (String, u32) {
        let mut last_upper: Option<Rgb<u8>> = None;
        let mut last_lower: Option<Rgb<u8>> = None;
        let mut instructions = 0;

        let mut frame = String::with_capacity((image.width() * image.height()) as usize);
        for y in (0..image.height() - 1).step_by(2) {
            for x in 0..image.width() {
                let upper = image.get_pixel(x, y);
                let lower = image.get_pixel(x, y + 1);

                if last_upper.is_none() || &last_upper.unwrap() != upper {
                    frame += &self.color(upper, true);
                    instructions += 1;
                }

                if last_lower.is_none() || &last_lower.unwrap() != lower {
                    frame += &self.color(lower, false);
                    instructions += 1;
                }

                frame += "▀";

                last_upper = Some(*upper);
                last_lower = Some(*lower);
            }
            frame += "\x1b[1E";
            instructions += 1;
        }

        (frame, instructions)
    }

    fn needs_width(&self) -> u32;
    fn needs_height(&self) -> u32;
    fn needs_color(&self) -> ColorMode;
    fn needs_dither(&self) -> DitherMode {
        DitherMode::None
    }
}
