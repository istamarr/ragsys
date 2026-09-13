use std::f32::consts::PI;
use image::{ImageBuffer, Rgb, RgbImage, DynamicImage};
use rand::prelude::*;
use std::env;
use std::time::Instant;
use crate::shared::helperUtils::current_time;

#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub output_path: String,
    pub generation_type: GenerationType,
    pub seed: Option<u64>,
    pub iterations: u32,
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Clone)]
pub enum GenerationType {
    NoisePattern,
    FractalMandelbrot,
    FractalJulia,
    PerlinNoise,
    VoronoiDiagram,
    DiffusionPattern,
    GeometricArt,
    PlasmaEffect,
    WaveInterference,
    CellularAutomata,
    VectorLines,
    CircularLogo,
    PolygonLogo,
    MinimalLines,
    TechGrid,
}

#[derive(Debug, Clone)]
pub enum ColorScheme {
    Grayscale,
    Rainbow,
    Sunset,
    Ocean,
    Forest,
    Fire,
    Electric,
    Pastel,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            output_path: "rag_output/agent_diffuser/generated_image.png".to_string(),
            generation_type: GenerationType::DiffusionPattern,
            seed: None,
            iterations: 100,
            color_scheme: ColorScheme::Rainbow,
        }
    }
}

pub struct RustDiffuser {
    config: ImageConfig,
    rng: StdRng,
}

impl RustDiffuser {
    pub fn new(config: ImageConfig) -> Self {
        let seed = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });

        Self {
            config,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn generate_image(&mut self) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        println!("Starting image generation...");
        println!("Dimensions: {}x{}", self.config.width, self.config.height);
        println!("Type: {:?}", self.config.generation_type);
        println!("Color Scheme: {:?}", self.config.color_scheme);

        let image = match self.config.generation_type {
            GenerationType::NoisePattern => self.generate_noise_pattern(),
            GenerationType::FractalMandelbrot => self.generate_mandelbrot(),
            GenerationType::FractalJulia => self.generate_julia(),
            GenerationType::PerlinNoise => self.generate_perlin_noise(),
            GenerationType::VoronoiDiagram => self.generate_voronoi(),
            GenerationType::DiffusionPattern => self.generate_diffusion_pattern(),
            GenerationType::GeometricArt => self.generate_geometric_art(),
            GenerationType::PlasmaEffect => self.generate_plasma_effect(),
            GenerationType::WaveInterference => self.generate_wave_interference(),
            GenerationType::CellularAutomata => self.generate_cellular_automata(),
            GenerationType::VectorLines => self.generate_vector_lines(),
            GenerationType::CircularLogo => self.generate_circular_logo(),
            GenerationType::PolygonLogo => self.generate_polygon_logo(),
            GenerationType::MinimalLines => self.generate_minimal_lines(),
            GenerationType::TechGrid => self.generate_tech_grid(),
        };

        let duration = start_time.elapsed();
        println!("Image generated in {:.2}s", duration.as_secs_f32());

        Ok(DynamicImage::ImageRgb8(image))
    }

    fn generate_noise_pattern(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);

        for (_x, _y, pixel) in img.enumerate_pixels_mut() {
            let noise_value = self.rng.r#gen::<f32>();
            let color = self.apply_color_scheme(noise_value);
            *pixel = color;
        }

        img
    }

    fn generate_mandelbrot(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let max_iter = self.config.iterations;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let cx = (x as f64 / self.config.width as f64) * 3.0 - 2.0;
            let cy = (y as f64 / self.config.height as f64) * 3.0 - 1.5;

            let mut zx = 0.0;
            let mut zy = 0.0;
            let mut iter = 0;

            while zx * zx + zy * zy < 4.0 && iter < max_iter {
                let temp = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = temp;
                iter += 1;
            }

            let intensity = if iter == max_iter { 0.0 } else { iter as f32 / max_iter as f32 };
            let color = self.apply_color_scheme(intensity);
            *pixel = color;
        }

        img
    }

    fn generate_julia(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let max_iter = self.config.iterations;
        let c_real = -0.7;
        let c_imag = 0.27015;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let zx = (x as f64 / self.config.width as f64) * 3.0 - 1.5;
            let zy = (y as f64 / self.config.height as f64) * 3.0 - 1.5;

            let mut zx_temp = zx;
            let mut zy_temp = zy;
            let mut iter = 0;

            while zx_temp * zx_temp + zy_temp * zy_temp < 4.0 && iter < max_iter {
                let temp = zx_temp * zx_temp - zy_temp * zy_temp + c_real;
                zy_temp = 2.0 * zx_temp * zy_temp + c_imag;
                zx_temp = temp;
                iter += 1;
            }

            let intensity = if iter == max_iter { 0.0 } else { iter as f32 / max_iter as f32 };
            let color = self.apply_color_scheme(intensity);
            *pixel = color;
        }

        img
    }

    fn generate_perlin_noise(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let scale = 0.01;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let noise_value = self.perlin_noise(x as f32 * scale, y as f32 * scale);
            let normalized = (noise_value + 1.0) / 2.0; // Normalize to 0-1
            let color = self.apply_color_scheme(normalized);
            *pixel = color;
        }

        img
    }

    fn generate_voronoi(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let num_points = 50;

        // Generate random seed points
        let mut points = Vec::new();
        for _ in 0..num_points {
            points.push((
                self.rng.gen_range(0..self.config.width),
                self.rng.gen_range(0..self.config.height),
                self.rng.r#gen::<f32>(),
            ));
        }

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let mut min_distance = f32::MAX;
            let mut closest_value = 0.0;

            for &(px, py, value) in &points {
                let distance = ((x as f32 - px as f32).powi(2) + (y as f32 - py as f32).powi(2)).sqrt();
                if distance < min_distance {
                    min_distance = distance;
                    closest_value = value;
                }
            }

            let color = self.apply_color_scheme(closest_value);
            *pixel = color;
        }

        img
    }

    fn generate_diffusion_pattern(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            // Create diffusion-like pattern
            let noise1 = self.perlin_noise(x as f32 * 0.01, y as f32 * 0.01);
            let noise2 = self.perlin_noise(x as f32 * 0.005, y as f32 * 0.005);
            let radial = (distance / (self.config.width as f32 / 2.0)).min(1.0);
            let angular = (angle + PI) / (2.0 * PI);

            let intensity = (noise1 * 0.4 + noise2 * 0.3 + radial * 0.2 + angular * 0.1 + 1.0) / 2.0;
            let color = self.apply_color_scheme(intensity.clamp(0.0, 1.0));
            *pixel = color;
        }

        img
    }

    fn generate_geometric_art(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let fx = x as f32 / self.config.width as f32;
            let fy = y as f32 / self.config.height as f32;

            // Create geometric patterns
            let pattern1 = ((fx * 20.0).sin() * (fy * 20.0).sin()).abs();
            let pattern2 = ((fx * 15.0 + fy * 15.0).cos()).abs();
            let pattern3 = ((fx * fx + fy * fy) * 50.0).sin().abs();

            let intensity = (pattern1 + pattern2 + pattern3) / 3.0;
            let color = self.apply_color_scheme(intensity);
            *pixel = color;
        }

        img
    }

    fn generate_plasma_effect(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let fx = x as f32 / self.config.width as f32;
            let fy = y as f32 / self.config.height as f32;

            let plasma = (
                (fx * 16.0).sin() +
                    (fy * 16.0).sin() +
                    ((fx * 16.0 + fy * 16.0) / 2.0).sin() +
                    ((fx * fx + fy * fy).sqrt() * 8.0).sin()
            ) / 4.0;

            let intensity = (plasma + 1.0) / 2.0;
            let color = self.apply_color_scheme(intensity);
            *pixel = color;
        }

        img
    }

    fn generate_wave_interference(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        // Multiple wave sources
        let sources = vec![
            (center_x * 0.3, center_y * 0.3),
            (center_x * 1.7, center_y * 0.3),
            (center_x * 0.3, center_y * 1.7),
            (center_x * 1.7, center_y * 1.7),
        ];

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let mut wave_sum = 0.0;

            for &(sx, sy) in &sources {
                let distance = ((x as f32 - sx).powi(2) + (y as f32 - sy).powi(2)).sqrt();
                wave_sum += (distance * 0.1).sin() / (distance * 0.01 + 1.0);
            }

            let intensity = (wave_sum + 2.0) / 4.0;
            let color = self.apply_color_scheme(intensity.clamp(0.0, 1.0));
            *pixel = color;
        }

        img
    }

    fn generate_cellular_automata(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let mut grid = vec![vec![false; self.config.height as usize]; self.config.width as usize];

        // Initialize with random cells
        for x in 0..self.config.width {
            for y in 0..self.config.height {
                grid[x as usize][y as usize] = self.rng.gen_bool(0.4);
            }
        }

        // Run cellular automata iterations
        for _ in 0..10 {
            let mut new_grid = grid.clone();

            for x in 1..(self.config.width - 1) {
                for y in 1..(self.config.height - 1) {
                    let mut neighbors = 0;
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if dx == 0 && dy == 0 { continue; }
                            if grid[(x as i32 + dx) as usize][(y as i32 + dy) as usize] {
                                neighbors += 1;
                            }
                        }
                    }

                    // Conway's Game of Life rules
                    new_grid[x as usize][y as usize] = match (grid[x as usize][y as usize], neighbors) {
                        (true, 2) | (true, 3) => true,
                        (false, 3) => true,
                        _ => false,
                    };
                }
            }

            grid = new_grid;
        }

        // Convert to image
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let intensity = if grid[x as usize][y as usize] { 1.0 } else { 0.0 };
            let color = self.apply_color_scheme(intensity);
            *pixel = color;
        }

        img
    }

    fn generate_vector_lines(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw clean vector-style lines
        let num_lines = 8;
        let line_thickness = 3;

        for i in 0..num_lines {
            let angle = (i as f32 * 2.0 * PI) / num_lines as f32;
            let length = (self.config.width.min(self.config.height) as f32) * 0.3;

            let start_x = center_x + (angle.cos() * length * 0.3) as f32;
            let start_y = center_y + (angle.sin() * length * 0.3) as f32;
            let end_x = center_x + (angle.cos() * length) as f32;
            let end_y = center_y + (angle.sin() * length) as f32;

            self.draw_line(&mut img, start_x as i32, start_y as i32, end_x as i32, end_y as i32, line_thickness);
        }

        img
    }

    fn generate_circular_logo(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw concentric circles
        let max_radius = (self.config.width.min(self.config.height) as f32) * 0.4;
        let num_circles = 5;
        let line_thickness = 4;

        for i in 1..=num_circles {
            let radius = (max_radius * i as f32) / num_circles as f32;
            self.draw_circle(&mut img, center_x as i32, center_y as i32, radius as i32, line_thickness);
        }

        // Add cross lines
        self.draw_line(&mut img,
                       (center_x - max_radius) as i32, center_y as i32,
                       (center_x + max_radius) as i32, center_y as i32, line_thickness);
        self.draw_line(&mut img,
                       center_x as i32, (center_y - max_radius) as i32,
                       center_x as i32, (center_y + max_radius) as i32, line_thickness);

        img
    }

    fn generate_polygon_logo(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw nested polygons
        let sides = 6; // Hexagon
        let max_radius = (self.config.width.min(self.config.height) as f32) * 0.4;
        let num_polygons = 3;
        let line_thickness = 3;

        for poly in 1..=num_polygons {
            let radius = (max_radius * poly as f32) / num_polygons as f32;
            let mut points = Vec::new();

            for i in 0..sides {
                let angle = (i as f32 * 2.0 * PI) / sides as f32;
                let x = center_x + (angle.cos() * radius);
                let y = center_y + (angle.sin() * radius);
                points.push((x as i32, y as i32));
            }

            // Draw polygon edges
            for i in 0..sides {
                let start = points[i];
                let end = points[(i + 1) % sides];
                self.draw_line(&mut img, start.0, start.1, end.0, end.1, line_thickness);
            }
        }

        img
    }

    fn generate_minimal_lines(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);
        let center_x = self.config.width as f32 / 2.0;
        let center_y = self.config.height as f32 / 2.0;

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw minimal geometric lines
        let size = (self.config.width.min(self.config.height) as f32) * 0.3;
        let line_thickness = 2;

        // Draw triangle
        let triangle_height = size * 0.866; // sqrt(3)/2
        let p1 = (center_x as i32, (center_y - triangle_height * 0.5) as i32);
        let p2 = ((center_x - size * 0.5) as i32, (center_y + triangle_height * 0.5) as i32);
        let p3 = ((center_x + size * 0.5) as i32, (center_y + triangle_height * 0.5) as i32);

        self.draw_line(&mut img, p1.0, p1.1, p2.0, p2.1, line_thickness);
        self.draw_line(&mut img, p2.0, p2.1, p3.0, p3.1, line_thickness);
        self.draw_line(&mut img, p3.0, p3.1, p1.0, p1.1, line_thickness);

        // Draw inner lines
        let inner_size = size * 0.5;
        self.draw_line(&mut img,
                       (center_x - inner_size) as i32, center_y as i32,
                       (center_x + inner_size) as i32, center_y as i32, line_thickness);

        img
    }

    fn generate_tech_grid(&mut self) -> RgbImage {
        let mut img = ImageBuffer::new(self.config.width, self.config.height);

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw tech-style grid
        let grid_size = 40;
        let line_thickness = 1;

        // Vertical lines
        for x in (0..self.config.width).step_by(grid_size) {
            self.draw_line(&mut img, x as i32, 0, x as i32, self.config.height as i32, line_thickness);
        }

        // Horizontal lines
        for y in (0..self.config.height).step_by(grid_size) {
            self.draw_line(&mut img, 0, y as i32, self.config.width as i32, y as i32, line_thickness);
        }

        // Add accent elements
        let center_x = self.config.width / 2;
        let center_y = self.config.height / 2;
        let accent_size = (grid_size * 3) as u32;

        // Draw accent rectangle
        self.draw_line(&mut img,
                       (center_x - accent_size) as i32, (center_y - accent_size) as i32,
                       (center_x + accent_size) as i32, (center_y - accent_size) as i32, 3);
        self.draw_line(&mut img,
                       (center_x + accent_size) as i32, (center_y - accent_size) as i32,
                       (center_x + accent_size) as i32, (center_y + accent_size) as i32, 3);
        self.draw_line(&mut img,
                       (center_x + accent_size) as i32, (center_y + accent_size) as i32,
                       (center_x - accent_size) as i32, (center_y + accent_size) as i32, 3);
        self.draw_line(&mut img,
                       (center_x - accent_size) as i32, (center_y + accent_size) as i32,
                       (center_x - accent_size) as i32, (center_y - accent_size) as i32, 3);

        img
    }

    fn draw_line(&self, img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, thickness: i32) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            // Draw thick line by drawing multiple pixels around the center
            for dx in -(thickness/2)..=(thickness/2) {
                for dy in -(thickness/2)..=(thickness/2) {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        let color = self.get_line_color();
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }

            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_circle(&self, img: &mut RgbImage, center_x: i32, center_y: i32, radius: i32, thickness: i32) {
        for angle in 0..360 {
            let rad = (angle as f32) * PI / 180.0;
            let x = center_x + (rad.cos() * radius as f32) as i32;
            let y = center_y + (rad.sin() * radius as f32) as i32;

            // Draw thick circle by drawing multiple pixels
            for dx in -(thickness/2)..=(thickness/2) {
                for dy in -(thickness/2)..=(thickness/2) {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        let color = self.get_line_color();
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    fn get_line_color(&self) -> Rgb<u8> {
        match self.config.color_scheme {
            ColorScheme::Grayscale => Rgb([0, 0, 0]),
            ColorScheme::Ocean => Rgb([0, 100, 150]),
            ColorScheme::Fire => Rgb([200, 50, 0]),
            ColorScheme::Forest => Rgb([0, 120, 50]),
            ColorScheme::Electric => Rgb([0, 255, 255]),
            ColorScheme::Sunset => Rgb([255, 100, 0]),
            ColorScheme::Pastel => Rgb([150, 150, 200]),
            ColorScheme::Rainbow => Rgb([100, 50, 200]),
        }
    }

    fn perlin_noise(&self, x: f32, y: f32) -> f32 {
        // Simplified Perlin noise implementation
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;

        let u = fade(xf);
        let v = fade(yf);

        let aa = grad(hash(xi, yi), xf, yf);
        let ab = grad(hash(xi, yi + 1), xf, yf - 1.0);
        let ba = grad(hash(xi + 1, yi), xf - 1.0, yf);
        let bb = grad(hash(xi + 1, yi + 1), xf - 1.0, yf - 1.0);

        lerp(v, lerp(u, aa, ba), lerp(u, ab, bb))
    }

    fn apply_color_scheme(&self, intensity: f32) -> Rgb<u8> {
        let clamped = intensity.clamp(0.0, 1.0);

        match self.config.color_scheme {
            ColorScheme::Grayscale => {
                let gray = (clamped * 255.0) as u8;
                Rgb([gray, gray, gray])
            },
            ColorScheme::Rainbow => {
                let hue = clamped * 360.0;
                hsv_to_rgb(hue, 1.0, 1.0)
            },
            ColorScheme::Sunset => {
                let r = (clamped * 255.0) as u8;
                let g = ((clamped * 0.7) * 255.0) as u8;
                let b = ((clamped * 0.3) * 255.0) as u8;
                Rgb([r, g, b])
            },
            ColorScheme::Ocean => {
                let r = ((clamped * 0.2) * 255.0) as u8;
                let g = ((clamped * 0.6) * 255.0) as u8;
                let b = (clamped * 255.0) as u8;
                Rgb([r, g, b])
            },
            ColorScheme::Forest => {
                let r = ((clamped * 0.3) * 255.0) as u8;
                let g = (clamped * 255.0) as u8;
                let b = ((clamped * 0.4) * 255.0) as u8;
                Rgb([r, g, b])
            },
            ColorScheme::Fire => {
                let r = (clamped * 255.0) as u8;
                let g = ((clamped * clamped) * 255.0) as u8;
                let b = ((clamped * clamped * clamped) * 255.0) as u8;
                Rgb([r, g, b])
            },
            ColorScheme::Electric => {
                let r = ((clamped * 0.8 + 0.2) * 255.0) as u8;
                let g = (clamped * 255.0) as u8;
                let b = ((1.0 - clamped) * 255.0) as u8;
                Rgb([r, g, b])
            },
            ColorScheme::Pastel => {
                let base = clamped * 0.6 + 0.4;
                let r = (base * 255.0) as u8;
                let g = ((base * 0.9) * 255.0) as u8;
                let b = ((base * 1.1).min(1.0) * 255.0) as u8;
                Rgb([r, g, b])
            },
        }
    }

    pub fn save_image(&self, image: &DynamicImage, output_path: &String) -> Result<(), Box<dyn std::error::Error>> {
        println!("Saving image to: {}", output_path);
        image.save(&output_path)?;
        println!("Image saved successfully!");
        Ok(())
    }
}

// Helper functions for Perlin noise
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: i32, x: f32, y: f32) -> f32 {
    let h = hash & 3;
    let u = if h < 2 { x } else { y };
    let v = if h < 2 { y } else { x };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

fn hash(x: i32, y: i32) -> i32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = h.wrapping_add(h >> 13);
    h = h.wrapping_mul(1274126177);
    h = h.wrapping_add(h >> 16);
    h & 255
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Rgb<u8> {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Rgb([
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ])
}

fn parse_generation_type(s: &str) -> Result<GenerationType, String> {
    match s.to_lowercase().as_str() {
        "noise" => Ok(GenerationType::NoisePattern),
        "mandelbrot" => Ok(GenerationType::FractalMandelbrot),
        "julia" => Ok(GenerationType::FractalJulia),
        "perlin" => Ok(GenerationType::PerlinNoise),
        "voronoi" => Ok(GenerationType::VoronoiDiagram),
        "diffusion" => Ok(GenerationType::DiffusionPattern),
        "geometric" => Ok(GenerationType::GeometricArt),
        "plasma" => Ok(GenerationType::PlasmaEffect),
        "waves" => Ok(GenerationType::WaveInterference),
        "cellular" => Ok(GenerationType::CellularAutomata),
        "vector" | "vectorlines" => Ok(GenerationType::VectorLines),
        "circular" | "circle" => Ok(GenerationType::CircularLogo),
        "polygon" | "hex" => Ok(GenerationType::PolygonLogo),
        "minimal" | "lines" => Ok(GenerationType::MinimalLines),
        "grid" | "techgrid" => Ok(GenerationType::TechGrid),
        _ => Err(format!("Unknown generation type: {}", s)),
    }
}

fn parse_color_scheme(s: &str) -> Result<ColorScheme, String> {
    match s.to_lowercase().as_str() {
        "grayscale" | "gray" => Ok(ColorScheme::Grayscale),
        "rainbow" => Ok(ColorScheme::Rainbow),
        "sunset" => Ok(ColorScheme::Sunset),
        "ocean" => Ok(ColorScheme::Ocean),
        "forest" => Ok(ColorScheme::Forest),
        "fire" => Ok(ColorScheme::Fire),
        "electric" => Ok(ColorScheme::Electric),
        "pastel" => Ok(ColorScheme::Pastel),
        _ => Err(format!("Unknown color scheme: {}", s)),
    }
}

// fn print_help() {
//     println!("rustimg - difusi - Advanced Image Generation Tool");
//     println!();
//     println!("USAGE:");
//     println!("    cargo run --bin rustimg [OPTIONS]");
//     println!();
//     println!("OPTIONS:");
//     println!("    --type <TYPE>        Generation type [default: diffusion]");
//     println!("                         Options: noise, mandelbrot, julia, perlin, voronoi,");
//     println!("                                 diffusion, geometric, plasma, waves, cellular,");
//     println!("                                 vector, circular, polygon, minimal, grid");
//     println!("    --width <WIDTH>      Image width [default: 512]");
//     println!("    --height <HEIGHT>    Image height [default: 512]");
//     println!("    --output <PATH>      Output file path [default: generated_image.png]");
//     println!("    --color <SCHEME>     Color scheme [default: rainbow]");
//     println!("                         Options: grayscale, rainbow, sunset, ocean, forest,");
//     println!("                                 fire, electric, pastel");
//     println!("    --seed <SEED>        Random seed for reproducible results");
//     println!("    --iterations <N>     Number of iterations for fractal generation [default: 100]");
//     println!("    --help               Show this help message");
//     println!();
//     println!("EXAMPLES:");
//     println!("    cargo run --bin rustimg --type mandelbrot --width 1024 --height 1024");
//     println!("    cargo run --bin rustimg --type diffusion --color sunset --seed 12345");
//     println!("    cargo run --bin rustimg --type plasma --color electric --output plasma.png");
// }

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        // print_help();
        return Ok(());
    }

    let mut config = ImageConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                if i + 1 < args.len() {
                    config.generation_type = parse_generation_type(&args[i + 1])?;
                    i += 2;
                } else {
                    return Err("--type requires a value".into());
                }
            },
            "--width" => {
                if i + 1 < args.len() {
                    config.width = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("--width requires a value".into());
                }
            },
            "--height" => {
                if i + 1 < args.len() {
                    config.height = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("--height requires a value".into());
                }
            },
            "--output" => {
                if i + 1 < args.len() {
                    config.output_path = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".into());
                }
            },
            "--color" => {
                if i + 1 < args.len() {
                    config.color_scheme = parse_color_scheme(&args[i + 1])?;
                    i += 2;
                } else {
                    return Err("--color requires a value".into());
                }
            },
            "--seed" => {
                if i + 1 < args.len() {
                    config.seed = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    return Err("--seed requires a value".into());
                }
            },
            "--iterations" => {
                if i + 1 < args.len() {
                    config.iterations = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("--iterations requires a value".into());
                }
            },
            _ => {
                println!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    println!("Rust Img Difusi Starting...");
    println!("{}", "=".repeat(50));


    let output_path =  format!("rag_output/agent_diffuser/img_{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"));

    let mut diffuser = RustDiffuser::new(config.clone());
    let image = diffuser.generate_image()?;
    diffuser.save_image(&image, &output_path)?;

    println!("{}", "=".repeat(50));
    println!("Image generation completed - {:?}",current_time());
    println!("Output: {}", config.output_path);

    // Display file size
    if let Ok(metadata) = std::fs::metadata(&config.output_path) {
        println!("File size: {:.2} KB", metadata.len() as f64 / 1024.0);
    }

    Ok(())
}

// ── FlyerConfig ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct FlyerConfig {
    pub title: String, pub subtitle: String, pub headline: String,
    pub subheadline: String, pub body_text: String,
    pub bullet_points: Vec<String>, pub cta_text: String,
    pub footer_contact: String, pub footer_website: String,
    pub hero_bg_color: Rgb<u8>, pub hero_accent_color: Rgb<u8>, pub accent_color: Rgb<u8>,
    pub output_path: String, pub width: u32, pub height: u32,
    pub with_ai_background: bool,
    pub header_left_logo_path: Option<String>, pub header_right_logo_path: Option<String>,
    pub body_product_image_path: Option<String>, pub footer_qr_image_path: Option<String>,
    pub font_path: Option<String>,
}
impl Default for FlyerConfig {
    fn default() -> Self {
        Self {
            title:                   env::var("FLYER_TITLE").unwrap_or_else(|_| "PROMO HARI INI".to_string()),
            subtitle:                env::var("FLYER_SUBTITLE").unwrap_or_default(),
            headline:                env::var("FLYER_HEADLINE").unwrap_or_else(|_| "Bunga Rendah, Proses Cepat".to_string()),
            subheadline:             env::var("FLYER_SUBHEADLINE").unwrap_or_default(),
            body_text:               env::var("FLYER_BODY_TEXT").unwrap_or_default(),
            bullet_points:           env::var("FLYER_BULLETS").map(|s| s.split('|').map(|b| b.trim().to_string()).collect()).unwrap_or_default(),
            cta_text:                env::var("FLYER_CTA").unwrap_or_else(|_| "Hubungi Kami Sekarang".to_string()),
            footer_contact:          env::var("FLYER_CONTACT").unwrap_or_else(|_| "1500 569".to_string()),
            footer_website:          env::var("FLYER_WEBSITE").unwrap_or_else(|_| "www.pegadaian.co.id".to_string()),
            hero_bg_color:           Rgb([218, 165, 32]),
            hero_accent_color:       Rgb([255, 215, 0]),
            accent_color:            Rgb([34, 139, 34]),
            output_path:             env::var("FLYER_OUTPUT").unwrap_or_else(|_| "flyer_output.png".to_string()),
            width:                   env::var("FLYER_WIDTH").ok().and_then(|v| v.parse().ok()).unwrap_or(600),
            height:                  env::var("FLYER_HEIGHT").ok().and_then(|v| v.parse().ok()).unwrap_or(850),
            with_ai_background:      env::var("FLYER_WITH_AI").map(|v| v == "true").unwrap_or(false),
            header_left_logo_path:   env::var("FLYER_LOGO_LEFT").ok(),
            header_right_logo_path:  env::var("FLYER_LOGO_RIGHT").ok(),
            body_product_image_path: env::var("FLYER_PRODUCT_IMAGE").ok(),
            footer_qr_image_path:    env::var("FLYER_QR_IMAGE").ok(),
            font_path:               env::var("FLYER_FONT_PATH").ok(),
        }
    }
}

// ── FlyerGenerator ────────────────────────────────────────────────────────────
pub struct FlyerGenerator { config: FlyerConfig }

impl FlyerGenerator {
    pub fn new(config: FlyerConfig) -> Self { Self { config } }

    pub fn render(&self) -> Result<RgbImage, Box<dyn std::error::Error>> {
        let (w, h) = (self.config.width, self.config.height);
        let mut img: RgbImage = ImageBuffer::new(w, h);
        for p in img.pixels_mut() { *p = Rgb([255u8, 255, 255]); }
        let hdr  = (h as f32 * 0.13) as u32;
        let hero = (h as f32 * 0.18) as u32;
        let body = (h as f32 * 0.52) as u32;
        let cta  = (h as f32 * 0.10) as u32;
        let mut y = 0u32;
        self.draw_header(&mut img, 0, y, w, hdr);  y += hdr;
        self.draw_hero  (&mut img, 0, y, w, hero); y += hero;
        self.draw_body  (&mut img, 0, y, w, body); y += body;
        self.draw_cta   (&mut img, 0, y, w, cta);  y += cta;
        self.draw_footer(&mut img, 0, y, w, h.saturating_sub(y));
        Ok(img)
    }

    fn draw_header(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
        self.fill_rect(img, x, y, w, h, Rgb([255, 255, 255]));
        self.fill_rect(img, x, y + h.saturating_sub(2), w, 2, Rgb([200, 200, 200]));
        if let Some(ref p) = self.config.header_left_logo_path  { self.overlay_img(img, p, x + 10,     y + 5, 80, h.saturating_sub(10)); }
        if let Some(ref p) = self.config.header_right_logo_path { self.overlay_img(img, p, x + w - 90, y + 5, 80, h.saturating_sub(10)); }
    }

    fn draw_hero(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
        self.fill_rect(img, x, y, w, h, self.config.hero_bg_color);
        let a = self.config.hero_accent_color;
        self.draw_circle(img, x + w - 40, y + h / 2, 60, a);
        self.draw_circle(img, x + 20, y + 10, 40, a);
        self.fill_rect(img, x + 20, y + h / 2 - 2, w.saturating_sub(40), 4, Rgb([255, 255, 255]));
    }

    fn draw_body(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
        self.fill_rect(img, x, y, w, h, Rgb([255, 255, 255]));
        let a = self.config.accent_color;
        self.fill_rect(img, x, y, 6, h, a);
        for i in 0..self.config.bullet_points.len().min(5) {
            let by = y + 60 + i as u32 * 35;
            if by + 12 <= y + h { self.draw_circle(img, x + 25, by + 6, 8, a); }
        }
        if let Some(ref p) = self.config.body_product_image_path {
            let (pw, ph) = (w / 3, h / 2);
            self.overlay_img(img, p, x + w - pw - 10, y + (h - ph) / 2, pw, ph);
        }
    }

    fn draw_cta(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
        self.fill_rect(img, x, y, w, h, Rgb([255, 140, 0]));
        self.fill_rect(img, x + 20, y + h / 2 - 2, w.saturating_sub(40), 4, Rgb([255, 255, 255]));
    }

    fn draw_footer(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
        self.fill_rect(img, x, y, w, h, Rgb([245, 245, 245]));
        self.fill_rect(img, x, y, w, 2, Rgb([200, 200, 200]));
        if let Some(ref p) = self.config.footer_qr_image_path {
            let sz = h.min(w / 4).saturating_sub(4);
            self.overlay_img(img, p, x + w - sz - 5, y + 2, sz, sz);
        }
    }

    fn fill_rect(&self, img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, c: Rgb<u8>) {
        let (iw, ih) = img.dimensions();
        for py in y..(y + h).min(ih) { for px in x..(x + w).min(iw) { img.put_pixel(px, py, c); } }
    }

    fn draw_circle(&self, img: &mut RgbImage, cx: u32, cy: u32, r: u32, c: Rgb<u8>) {
        let (iw, ih) = img.dimensions();
        let (cx, cy, r) = (cx as i64, cy as i64, r as i64);
        for dy in -r..=r { for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let (px, py) = (cx + dx, cy + dy);
                if px >= 0 && py >= 0 && px < iw as i64 && py < ih as i64 {
                    img.put_pixel(px as u32, py as u32, c);
                }
            }
        }}
    }

    fn overlay_img(&self, img: &mut RgbImage, path: &str, x: u32, y: u32, mw: u32, mh: u32) {
        use image::GenericImageView;
        if let Ok(src) = image::open(path) {
            let (sw, sh) = src.dimensions();
            let scale = (mw as f32 / sw as f32).min(mh as f32 / sh as f32).min(1.0);
            let resized = src.resize((sw as f32 * scale) as u32, (sh as f32 * scale) as u32, image::imageops::FilterType::Lanczos3);
            let (iw, ih) = img.dimensions();
            for (px, py, pixel) in resized.pixels() {
                let (tx, ty) = (x + px, y + py);
                if tx < iw && ty < ih { img.put_pixel(tx, ty, Rgb([pixel[0], pixel[1], pixel[2]])); }
            }
        }
    }
}

pub fn generate_flyer_image(config: FlyerConfig) -> Result<String, Box<dyn std::error::Error>> {
    let out = config.output_path.clone();
    if let Some(parent) = std::path::Path::new(&out).parent() { std::fs::create_dir_all(parent)?; }
    let img = FlyerGenerator::new(config).render()?;
    img.save(&out)?;
    println!("[FlyerGenerator] saved → {out}");
    Ok(out)
}
