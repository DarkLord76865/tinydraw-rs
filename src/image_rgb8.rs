use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use bytemuck::try_cast_slice;
use std::cmp::{min, max};


fn bytes_to_rgb8(bytes: &[u8]) -> Vec<[u8; 3]> {
    // converts a slice of bytes to a vector of pixels
    try_cast_slice::<u8, [u8; 3]>(bytes).expect("This shouldn't fail!").to_vec()
}

fn rgb8_to_bytes(rgb8: &[[u8; 3]]) -> &[u8] {
    // returns a slice of bytes of a vector (slice) of pixels
    try_cast_slice(rgb8).expect("This shouldn't fail!")
}

enum BackgroundRGB8 {
    Color([u8; 3]),
    Image(Vec<[u8; 3]>)
}


pub struct ImageRGB8 {
    /// The width of the image
    pub width: usize,
    /// The height of the image
    pub height: usize,
    /// The image pixel data
    pub image_data: Vec<[u8; 3]>,
    background_data: BackgroundRGB8
}

impl ImageRGB8 {
    pub fn new(width: usize, height: usize, background: [u8; 3]) -> Self {
        //! Returns new [ImageRGB8].
        //! ```width```, ```height``` are image dimensions.
        //! ```background``` is image's color.

        Self {width, height, image_data: vec![background; width * height], background_data: BackgroundRGB8::Color(background)}
    }

    pub fn from_png(path: &str) -> Result<Self, &'static str> {
        //! Reads image data from PNG file.
        //! Returns [Result] which holds new [ImageRGB8] or [Err] with informative message.
        //! ```path``` is the path to PNG file.
        //! The PNG file should be RGB or RGBA with bit depth 8.

        match File::open(path) {
            Ok(file) =>
                {
                    let decoder = png::Decoder::new(file);
                    match decoder.read_info() {
                        Ok(information) =>
                            {
                                let mut reader = information;
                                // Allocate the output buffer.
                                let mut buf = vec![0; reader.output_buffer_size()];
                                // Read the next frame. An APNG might contain multiple frames.
                                match reader.next_frame(&mut buf) {
                                    Ok(new_information) =>
                                        {
                                            let info = new_information;
                                            // Grab the bytes of the image.
                                            let bytes: &[u8];
                                            if info.bit_depth == png::BitDepth::Eight {
                                                // if image is not RGB panic, if it is RGBA convert to RGB
                                                match info.color_type {
                                                    png::ColorType::Rgb => {
                                                            bytes = &buf[..info.buffer_size()];
                                                            // return ImageRGB8 struct
                                                            Ok(Self::from_bytes(info.width as usize, info.height as usize, bytes).expect("This shouldn't fail!"))
                                                        },
                                                    png::ColorType::Rgba => {
                                                            buf.truncate(info.buffer_size());
                                                            let mut iterator = 1..(buf.len() + 1);
                                                            buf.retain(|_| iterator.next().expect("This shouldn't fail!") % 4 != 0);
                                                            bytes = &buf;
                                                            // return ImageRGB8 struct
                                                            Ok(Self::from_bytes(info.width as usize, info.height as usize, bytes).expect("This shouldn't fail!"))
                                                        },
                                                    _ => Err("Image color not RGB or RGBA!")
                                                }
                                            } else {
                                                Err("Image bit depth is not 8!")
                                            }
                                        },
                                    Err(_) => Err("Can't read file!")
                                }
                            },
                        Err(_) => Err("Can't read file!"),
                    }
                },
            Err(_) => Err("Can't open file!"),
        }
    }

    pub fn from_bytes(width: usize, height: usize, bytes: &[u8]) -> Result<Self, &'static str> {
        //! Returns [Result] with new [ImageRGB8] or [Err] with informative message.
        //! It is constructed from ```width```, ```height``` and ```bytes```

        if width * height * 3 != bytes.len() {
            // if number of bytes doesn't match expected number of bytes, panic
            Err("Number of bytes does not match an RGB image with given dimensions!")
        } else {
            // generate RGB image from bytes separately as it needs to be cloned as two separate instances are needed
            let img = bytes_to_rgb8(bytes);
            Ok(Self {width, height, image_data: img.clone(), background_data: BackgroundRGB8::Image(img)})
        }
    }

    pub fn to_png(&self, path: &str) {
        // saves image as PNG
        let path = Path::new(path);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(self.to_bytes()).unwrap();
    }

    pub fn to_bytes(&self) -> &[u8] {
        // returns slice of bytes of image_data
        rgb8_to_bytes(&self.image_data)
    }

    pub fn clear(&mut self) {
        // clear image of any drawings (by filling with background or replacing with background_data)

        match &self.background_data {
            BackgroundRGB8::Color(color) => self.image_data.fill(*color),
            BackgroundRGB8::Image(img) => self.image_data = img.clone(),
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 3] {
        // returns RGB value of single pixel
        if x >= self.width || y >= self.height {
            panic!("Given coordinates exceed image limits!")
        }
        self.image_data[self.width * (self.height - 1 - y) + x]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        // change color of single pixel
        self.image_data[self.width * (self.height - 1 - y) + x] = color;
    }

    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: [u8; 3]) {
        // draws anti aliased line

        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            // panic if any of the coordinates go out of the image
            panic!("Given coordinates exceed image limits!")
        } else if x1 == x2 {
            // if line is vertical just draw it
            for y in y1..(y2 + 1) {
                self.image_data[self.width * (self.height - 1 - y) + x1] = color;
            }
        } else {
            // if line has slope use Xiaolin Wu's algorithm to draw it anti aliased
            // if slope is more horizontal (<= 1), antialiasing with pixels above and below
            // if slope is more vertical (> 1), antialiasing with pixels left and right
            let slope: f64 = ((y1 as f64) - (y2 as f64)) / ((x1 as f64) - (x2 as f64));
            if slope.abs() <= 1.0 {
                for x in x1..(x2 + 1) {
                    let y: f64 = slope * ((x - x1) as f64) + (y1 as f64);

                    if (y - y.round()).abs() < 0.00001 {
                        // if point is very close to integer, just draw it on that pixel
                        self.image_data[self.width * (self.height - 1 - (y.round() as usize)) + x] = color;
                    } else {
                        // split point between two pixels
                        let pix1_percentage: f64 = y - y.floor();
                        let pix2_percentage: f64 = 1.0 - pix1_percentage;

                        let pix1_ind: usize = self.width * (self.height - 1 - (y.ceil() as usize)) + x;
                        let pix2_ind: usize = pix1_ind + self.width;

                        for channel in 0..color.len() {
                            // background color aware ===> color = color + (new_color - color) * color_percentage ===> color = color * (1 - color_percentage) + new_color * color_percentage
                            self.image_data[pix1_ind][channel] = ((self.image_data[pix1_ind][channel] as f64) * (pix2_percentage) + (color[channel] as f64) * pix1_percentage).round() as u8;
                            self.image_data[pix2_ind][channel] = ((self.image_data[pix2_ind][channel] as f64) * (pix1_percentage) + (color[channel] as f64) * pix2_percentage).round() as u8;
                        }
                    }
                }
            } else {
                for y in y1..(y2 + 1) {
                    let x: f64 = (((y - y1) as f64) / slope) + (x1 as f64);

                    if (x - x.round()).abs() < 0.00001 {
                        // if point is very close to integer, just draw it on that pixel
                        self.image_data[self.width * (self.height - 1 - y) + (x.round() as usize)] = color;
                    } else {
                        // split point between two pixels
                        let pix1_percentage: f64 = x.ceil() - x;
                        let pix2_percentage: f64 = 1.0 - pix1_percentage;

                        let pix1_ind: usize = self.width * (self.height - 1 - y) + (x.floor() as usize);
                        let pix2_ind: usize = pix1_ind + 1;

                        for channel in 0..color.len() {
                            // background color aware ===> color = color + (new_color - color) * color_percentage ===> color = color * (1 - color_percentage) + new_color * color_percentage
                            self.image_data[pix1_ind][channel] = ((self.image_data[pix1_ind][channel] as f64) * (pix2_percentage) + (color[channel] as f64) * pix1_percentage).round() as u8;
                            self.image_data[pix2_ind][channel] = ((self.image_data[pix2_ind][channel] as f64) * (pix1_percentage) + (color[channel] as f64) * pix2_percentage).round() as u8;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_rectangle(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: [u8; 3], thickness: usize) {

        // find which x is bigger to not get integer overflows when subtracting (because we are using usize which doesn't support negative integers)
        let smaller_x = min(x1, x2);
        let bigger_x = max(x1, x2);

        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            // panic if any of the coordinates go out of the image
            panic!("Given coordinates exceed image limits!");
        } else if thickness > (((bigger_x - smaller_x) / 2) + 1) {
            // if thickness set too high panic to avoid long, needless loops
            panic!("Thickness set too high!")
        }

        // find which y is bigger to know which one to put into iterator first and which second
        let smaller_y = min(y1, y2);
        let bigger_y = max(y1, y2);

        // draw horizontal sides
        self.image_data[(self.width * (self.height - 1 - y1) + smaller_x)..(self.width * (self.height - 1 - y1) + bigger_x + 1)].fill(color);
        self.image_data[(self.width * (self.height - 1 - y2) + smaller_x)..(self.width * (self.height - 1 - y2) + bigger_x + 1)].fill(color);
        // draw vertical sides
        for y in smaller_y..(bigger_y + 1) {
            let base_location = self.width * (self.height - 1 - y);
            self.image_data[base_location + smaller_x] = color;
            self.image_data[base_location + bigger_x] = color;
        }

        // if thickness is more than one call this function again to draw an    other, smaller rectangle inside this one
        if thickness > 1 {
            self.draw_rectangle(smaller_x + 1, smaller_y + 1, bigger_x - 1, bigger_y - 1, color, thickness - 1);
        }
    }

    pub fn draw_rectangle_filled(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: [u8; 3],) {

        if x1 >= self.width || x2 >= self.width || y1 >= self.height || y2 >= self.height {
            // panic if any of the coordinates go out of the image
            panic!("Given coordinates exceed image limits!");
        }

        // calculate which x, y is bigger to know how to properly index image_data
        let smaller_x = min(x1, x2);
        let bigger_x = max(x1, x2);
        let smaller_y = min(y1, y2);
        let bigger_y = max(y1, y2);
        // draw line by line onto image (faster than regular draw rectangle with high thickness as it doesn't need to call function repeatedly to draw smaller rectangles)
        for y in smaller_y..(bigger_y + 1) {
            let base_location = self.width * (self.height - 1 - y);
            self.image_data[(base_location + smaller_x)..(base_location + bigger_x + 1)].fill(color);
        }
    }

    pub fn draw_circle() {

    }

    pub fn draw_circle_filled() {

    }
}