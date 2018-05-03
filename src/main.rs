extern crate image;
extern crate rand;

use std::cmp;
use std::env::args;
use std::path::Path;
use std::io::stdout;
use std::io::Write;

use rand::{
    Rng,
};

use image::{
    GenericImage,
    Pixel,
    Rgb,
};

#[derive(Clone, Copy)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn black() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
        }
    }
    pub fn rand<R>(rng: &mut R, palette: &Option<&Vec<Color>>) -> Self
        where R: Rng {
            match palette {
                &Some(p) => {
                    let index = rng.gen_range::<usize>(0, p.len());
                    p[index]
                },
                &None =>
                    Color {
                        r: rng.gen_range::<u8>(0u8, 255u8),
                        g: rng.gen_range::<u8>(0u8, 255u8),
                        b: rng.gen_range::<u8>(0u8, 255u8),
                    }
            }
    }
    // pub fn dist(&self, other: &Color) -> u32 {
    //     ((self.r as i32 - other.r as i32).abs() +
    //      (self.g as i32 - other.g as i32).abs() +
    //      (self.b as i32 - other.b as i32).abs()) as u32
    // }
    pub fn dist(&self, other: &Color) -> u32 {
        let dr = (self.r as i32 - other.r as i32).abs() as u32;
        let dg = (self.g as i32 - other.g as i32).abs() as u32;
        let db = (self.b as i32 - other.b as i32).abs() as u32;
        let sq_sum = (dr * dr + dg * dg + db * db) as f32;
        sq_sum.sqrt() as u32
    }
}

struct Circle {
    pub x: i32,
    pub y: i32,
    pub r: u32,
    pub c: Color,
}

impl Circle {
    pub fn rand<R>(rng: &mut R, width: u32, height: u32, palette: &Option<&Vec<Color>>) -> Self
        where R: Rng {
        let color = Color::rand(rng, palette);
        Circle {
            x: rng.gen_range::<i32>(0i32, width as i32),
            y: rng.gen_range::<i32>(0i32, height as i32),
            r: rng.gen_range::<u32>(0u32, width / 4u32),
            c: color,
        }
    }
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        let dist_sq = (dx * dx + dy * dy) as u32;
        let r_sq = self.r * self.r;
        dist_sq <= r_sq
    }
    pub fn left(&self) -> i32 {
        self.x - self.r as i32
    }
    pub fn right(&self) -> i32 {
        self.x + self.r as i32
    }
    pub fn top(&self) -> i32 {
        self.y - self.r as i32
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.r as i32
    }
}

fn shape_improves_image(target_pixels: &Vec<Vec<Color>>, current_pixels: &Vec<Vec<Color>>, width: usize, height: usize, circle: &Circle) -> bool {
    let mut original_score = 0;
    let mut modified_score = 0;
    let xmin = cmp::max(0, circle.left()) as usize;
    let xmax = cmp::min(width as i32, circle.right()) as usize;
    let ymin = cmp::max(0, circle.top()) as usize;
    let ymax = cmp::min(height as i32, circle.bottom()) as usize;
    for x in xmin..xmax {
        for y in ymin..ymax {
            let target_color = target_pixels[x][y];
            let current_color = current_pixels[x][y];
            let updated_color = if circle.contains_point(x as i32, y as i32) {
                circle.c
            }
            else {
                current_pixels[x][y]
            };

            original_score = original_score + current_color.dist(&target_color);
            modified_score = modified_score + updated_color.dist(&target_color);
        }
    }

    modified_score < original_score
}

fn update_pixels(pixels: &mut Vec<Vec<Color>>, width: usize, height: usize, circle: &Circle) {
    let xmin = cmp::max(0, circle.left()) as usize;
    let xmax = cmp::min(width as i32, circle.right()) as usize;
    let ymin = cmp::max(0, circle.top()) as usize;
    let ymax = cmp::min(height as i32, circle.bottom()) as usize;
    for x in xmin..xmax {
        for y in ymin..ymax {
            if circle.contains_point(x as i32, y as i32) {
                pixels[x][y] = circle.c;
            }
        }
    }
}

fn dump_image(pixels: &Vec<Vec<Color>>, width: u32, height: u32, path: &String) {
    let mut result = image::ImageBuffer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let color = &pixels[x as usize][y as usize];
            result.put_pixel(x, y, Rgb::from_channels(color.r, color.g, color.b, 0u8));
        }
    }

    let path = Path::new(path);
    result.save(&path).unwrap();
}

fn main() {
    let default_iterations = 100;
    let input = args().nth(1).expect("first argument must be the input image");
    let output = args().nth(2).expect("second argument must be the output image");
    let iterations = match args().nth(3) {
        Some(i) => i.parse::<u32>().unwrap(),
        None => default_iterations,
    };

    let img = image::open(&input).unwrap();
    let (width, height) = img.dimensions();

    let mut current_pixels = vec![];
    for _ in 0..width {
        current_pixels.push(vec![Color::black(); height as usize]);
    }

    // load the original image
    let mut target_pixels = vec![];
    let mut all_pixels = vec![];
    for _ in 0..width {
        target_pixels.push(vec![Color::black(); height as usize]);
    }
    for (x, y, pixel) in img.pixels() {
        target_pixels[x as usize][y as usize] = Color { r: pixel.data[0], g: pixel.data[1], b: pixel.data[2] };
        all_pixels.push(vec![pixel.data[0] as f32, pixel.data[1] as f32, pixel.data[2] as f32]);
    }

    let mut rng = rand::thread_rng();
    let mut kept_count = 0;
    let mut last_pct = 0;
    for iteration in 0..iterations {
        let candidate_shape = Circle::rand(&mut rng, width, height, &None);
        if shape_improves_image(&target_pixels, &current_pixels, width as usize, height as usize, &candidate_shape) {
            update_pixels(&mut current_pixels, width as usize, height as usize, &candidate_shape);
            kept_count = kept_count + 1;
        }

        // display status
        let current_pct = (iteration * 100 / iterations) + 1;
        if current_pct > last_pct {
            print!("\x08\x08\x08\x08{:3}%", current_pct);
            let _ = stdout().flush();
            last_pct = current_pct;
        }
    }

    println!();
    println!("kept {} of {} iterations", kept_count, iterations);

    // create the final image
    dump_image(&current_pixels, width, height, &output);
}
