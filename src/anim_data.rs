use std::ops::Index;

use image::{RgbImage, Rgb, GenericImage};

#[derive(Clone,Debug)]
pub enum BlendType {
    Linear,
    Log,
    Cubic,
}

use BlendType::*;

// TODO: Change this and relevant functions to
// include support for the alpha channel,
// as would make sense for an overlay
#[derive(Clone, Debug)]
pub struct Overlay {
    pub pixels: Vec<Option<[u8; 3]>>,
    pub width: usize,
    pub height: usize,
    pub opacity: f64, // intended to be between 0 and 1 inclusive
    pub blend: Option<BlendType>,
}

impl Overlay {
    pub fn new(width: usize, height: usize) -> Overlay {

        let mut data = Vec::with_capacity(width*height);
        for _ in 0..width*height {
            data.push(None);
        }

        Overlay {
            pixels: data,
            width,
            height,
            blend: None,
            opacity: 1.0,
        }
    }
    pub fn as_image(&self) -> RgbImage {
        let mut image = RgbImage::new(self.width as u32, self.height as u32);

        for (old_pixel, new_pixel) in self.pixels.iter().zip(image.pixels_mut()) {
            if let Some(pixel) = old_pixel {
                let [r, g, b] = pixel;
                *new_pixel = (*pixel).into();
            }
        }

        image
    }
    pub fn is_image(&mut self, image: RgbImage) {
        let mut pixels = Vec::new();
        for pixel in image.pixels() {
            let (r, g, b) = (pixel.index(0), pixel.index(1), pixel.index(2));
            pixels.push(Some([*r, *g, *b]));
        }
        self.pixels = pixels;
    }
    pub fn from_rgb_image(image: RgbImage) -> Self {
        let mut pixels = Vec::new();
        for pixel in image.pixels() {
            let (r, g, b) = (pixel.index(0), pixel.index(1), pixel.index(2));
            pixels.push(Some([*r, *g, *b]));
        }
        let (width, height) = image.dimensions();

        Self {
            pixels,
            width: width as usize,
            height: height as usize,
            opacity: 1.0,
            blend: None
        }
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<[u8; 3]> {
        let len = self.pixels.len();
        self.pixels[ ((len / self.height) * y) + x ]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: &[u8; 3]) -> Result<(), i8> {
        let len = self.pixels.len();
        if x < self.width && y < self.height { 
            self.pixels[ ((len / self.height) * y) + x ] = Some(*rgb);
        } else {
            return Err(-1);
        }
        Ok(())
    }
    pub fn set_opacity(&mut self, opacity: f64) -> &mut Self {
        self.opacity = opacity.clamp(0.0,1.0);
        self
    }
    pub fn set_blend(&mut self, blend: BlendType) -> &mut Self {
        self.blend = Some(blend);
        self
    }
    pub fn set_linear(&mut self) -> &mut Self {
        self.blend = Some(Linear);
        self
    }
    pub fn set_log(&mut self) -> &mut Self {
        self.blend = Some(Log);
        self
    }
    pub fn no_blend(&mut self) -> &mut Self {
        self.blend = None;
        self
    }
}

pub trait Alter {
    fn overlay(&mut self, top: usize, left: usize, width: f64, height: f64, overlay: &Overlay);
}

impl Alter for RgbImage {
    fn overlay(&mut self, top: usize, left: usize, width: f64, height: f64, overlay: &Overlay) {
        // width and height here are percentages

        let w_size = (overlay.width as f64 * width + 0.5) as usize + left;
        let h_size = (overlay.height as f64 * height + 0.5) as usize + top;
        let len = overlay.pixels.len();
        for (ox, x) in (left..w_size).enumerate() {
            for (oy, y) in (top..h_size).enumerate() {
                
                // TODO: Following from below, if the size has been reduced,
                // find the area that has been squished into the area of a 
                // single pixel and combine them
                //
                // Use nearest neighbor interpolation
                // for blend: None
                // if the size is increased
                //
                // For BlendType::Linear, get the weighted average
                // of a containing area for each weighted position
                // when the area is decreased, and if it is increased,
                // do the same thing but stretch the change across multiple
                // pixels in a linear fashion
                // 
                // For BlendType::Log, do the same thing but in a logarithmic
                // fashion
                //
                // other kinds of interpolation maybe...

                if let Some(pixel) = overlay.get_pixel(ox, oy) {
                    let (w, h) = self.dimensions();
                    
                    // TODO: change this so that pixels are merged into each other
                    // if the size is less than normal for a given direction, 
                    // or both if both are less
                    //
                    // If the size is greater than normal, then use a gradient
                    // from one pixel to the next based on the distance between
                    // the position in the overlay and the position in the base image
                    //

                    if ox < w as usize && oy < h as usize {
                        self.put_pixel(x as u32, y as u32, Rgb::<u8>(pixel));
                    } else {
                        println!("Warning: overlay out of bounds");
                        for i in 0..25 {
                            self.put_pixel(i, i, Rgb::<u8>([i as u8,i as u8,i as u8]));
                        }
                    }
                }
            }
        }
    }
}
