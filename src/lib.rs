//! **tinydraw** is a small library for 2D drawing in Rust.
//! It is used for drawing basic, anti-aliased shapes onto images.
//! Support for reading and exporting images as PNG or bytes is included.
//!
//! Example usage:
//! ```rust
//! use tinydraw::ImageRGB8;
//!
//! let background_color: [u8; 3] = [255, 155, 0];
//! let mut image: ImageRGB8 = ImageRGB8::new(640, 360, background_color);
//!
//! image.draw_line(0, 0, 639, 359, [255, 255, 255], 1, 1.0);
//! image.draw_line(0, 359, 639, 0, [255, 255, 255], 1, 1.0);
//! image.draw_rectangle(0, 0, 639, 359, [255, 255, 255], 3, 1.0);
//! image.draw_ellipse(319, 179, 300, 150, [0, 0, 0], 0, 0.5);
//! image.draw_circle(149, 179, 30, [255, 255, 255], 0, 1.0);
//! image.draw_circle(149, 179, 20, [0, 0, 0], 0, 1.0);
//! image.draw_circle(489, 179, 30, [255, 255, 255], 0, 1.0);
//! image.draw_circle(489, 179, 20, [0, 0, 0], 0, 1.0);
//! image.draw_ellipse(319, 90, 80, 30, [255, 255, 255], 0, 1.0);
//! image.draw_ellipse(319, 90, 60, 20, [0, 0, 0], 0, 1.0);
//!
//! let bytes: &[u8] = image.to_bytes(); // get image as bytes
//! // image.to_png("image.png").unwrap(); // export image as PNG
//! ```
//!
//! **Shapes:** line, rectangle, ellipse, circle
//!
//! **Colorspaces:** RGB8

pub mod image;
#[doc(inline)]
pub use image::ImageRGB8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let background_color: [u8; 3] = [255, 155, 0];
        let mut image: ImageRGB8 = ImageRGB8::new(640, 360, background_color);

        image.draw_line(0, 0, 639, 359, [255, 255, 255], 1, 1.0);
        image.draw_line(0, 359, 639, 0, [255, 255, 255], 1, 1.0);

        image.draw_rectangle(0, 0, 639, 359, [255, 255, 255], 3, 1.0);

        image.draw_ellipse(319, 179, 300, 150, [0, 0, 0], 0, 0.5);

        image.draw_circle(149, 179, 30, [255, 255, 255], 0, 1.0);
        image.draw_circle(149, 179, 20, [0, 0, 0], 0, 1.0);

        image.draw_circle(489, 179, 30, [255, 255, 255], 0, 1.0);
        image.draw_circle(489, 179, 20, [0, 0, 0], 0, 1.0);


        image.draw_ellipse(319, 90, 80, 30, [255, 255, 255], 0, 1.0);
        image.draw_ellipse(319, 90, 60, 20, [0, 0, 0], 0, 1.0);

        let _bytes: &[u8] = image.to_bytes();
        // image.to_png("image.png").unwrap();
    }
}
