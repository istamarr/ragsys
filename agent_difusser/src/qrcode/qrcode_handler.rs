use std::env;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use qrcode::QrCode;
use qrcode::render::svg;

#[derive(Debug, Clone, PartialEq)]
pub enum QrOutputFormat {
    Png,
    Svg,
}

#[derive(Debug, Clone)]
pub struct QrConfig {
    pub content: String,
    pub size: u32,
    pub quiet_zone: u32,
    pub fg_color: [u8; 4],
    pub bg_color: [u8; 4],
    pub logo_path: Option<String>,
    pub logo_size_ratio: f32,
    pub output_path: String,
    pub format: QrOutputFormat,
}

impl QrConfig {
    pub fn from_env() -> Self {
        let base_url = env::var("QR_BASE_URL").unwrap_or_default();
        let content_raw = env::var("QR_CONTENT").unwrap_or_else(|_| "https://example.com".to_string());
        let content = if content_raw.starts_with("http") || base_url.is_empty() {
            content_raw
        } else {
            format!("{}{}", base_url, content_raw)
        };

        Self {
            content,
            size: env::var("QR_SIZE").ok().and_then(|v| v.parse().ok()).unwrap_or(512),
            quiet_zone: env::var("QR_QUIET_ZONE").ok().and_then(|v| v.parse().ok()).unwrap_or(4),
            fg_color: parse_hex_color(&env::var("QR_FG_COLOR").unwrap_or_else(|_| "000000FF".to_string())),
            bg_color: parse_hex_color(&env::var("QR_BG_COLOR").unwrap_or_else(|_| "FFFFFFFF".to_string())),
            logo_path: env::var("QR_LOGO_PATH").ok(),
            logo_size_ratio: env::var("QR_LOGO_SIZE_RATIO").ok().and_then(|v| v.parse().ok()).unwrap_or(0.22),
            output_path: env::var("QR_OUTPUT_PATH")
                .unwrap_or_else(|_| "rag_output/agent_diffuser/qrcode.png".to_string()),
            format: if env::var("QR_FORMAT").as_deref() == Ok("svg") {
                QrOutputFormat::Svg
            } else {
                QrOutputFormat::Png
            },
        }
    }
}

pub struct QrHandler {
    pub config: QrConfig,
}

impl QrHandler {
    pub fn new(config: QrConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Self {
        Self::new(QrConfig::from_env())
    }

    pub fn generate(&self) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let code = QrCode::new(self.config.content.as_bytes())?;

        let modules   = code.width();
        let cell_size = (self.config.size as usize / (modules + self.config.quiet_zone as usize * 2)).max(1);
        let canvas_size = ((modules + self.config.quiet_zone as usize * 2) * cell_size) as u32;

        let mut img: RgbaImage = ImageBuffer::new(canvas_size, canvas_size);

        let [bg_r, bg_g, bg_b, bg_a] = self.config.bg_color;
        let [fg_r, fg_g, fg_b, fg_a] = self.config.fg_color;

        for pixel in img.pixels_mut() {
            *pixel = Rgba([bg_r, bg_g, bg_b, bg_a]);
        }

        for row in 0..modules {
            for col in 0..modules {
                if code[(col, row)] == qrcode::Color::Dark {
                    let x_start = ((col + self.config.quiet_zone as usize) * cell_size) as u32;
                    let y_start = ((row + self.config.quiet_zone as usize) * cell_size) as u32;

                    for dy in 0..cell_size as u32 {
                        for dx in 0..cell_size as u32 {
                            let px = x_start + dx;
                            let py = y_start + dy;
                            if px < canvas_size && py < canvas_size {
                                img.put_pixel(px, py, Rgba([fg_r, fg_g, fg_b, fg_a]));
                            }
                        }
                    }
                }
            }
        }

        let mut dynamic = DynamicImage::ImageRgba8(img);

        if let Some(ref logo_path) = self.config.logo_path {
            dynamic = self.overlay_logo(dynamic, logo_path)?;
        }

        Ok(dynamic)
    }

    fn overlay_logo(&self, base: DynamicImage, logo_path: &str) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let logo = image::open(logo_path)?;
        let base_size = base.width().min(base.height());
        let logo_size = (base_size as f32 * self.config.logo_size_ratio) as u32;

        let logo_resized = logo.resize(logo_size, logo_size, image::imageops::FilterType::Lanczos3);

        let padding  = (logo_size as f32 * 0.15) as u32;
        let box_size = logo_size + padding * 2;

        let mut base_rgba = base.to_rgba8();

        let offset_x = (base_rgba.width() - box_size) / 2;
        let offset_y = (base_rgba.height() - box_size) / 2;

        let mut logo_bg: RgbaImage = ImageBuffer::new(box_size, box_size);
        for pixel in logo_bg.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }

        image::imageops::overlay(&mut logo_bg, &logo_resized.to_rgba8(), padding as i64, padding as i64);
        image::imageops::overlay(&mut base_rgba, &logo_bg, offset_x as i64, offset_y as i64);

        Ok(DynamicImage::ImageRgba8(base_rgba))
    }

    pub fn generate_svg(&self) -> Result<String, Box<dyn std::error::Error>> {
        let code = QrCode::new(self.config.content.as_bytes())?;
        let svg_str = code
            .render::<svg::Color>()
            .min_dimensions(self.config.size, self.config.size)
            .quiet_zone(true)
            .build();
        Ok(svg_str)
    }

    pub fn save(&self, image: &DynamicImage, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        image.save(path)?;
        Ok(())
    }

    pub fn generate_and_save(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = self.config.output_path.clone();

        match self.config.format {
            QrOutputFormat::Svg => {
                let svg_content = self.generate_svg()?;
                if let Some(parent) = std::path::Path::new(&output).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let svg_path = if output.ends_with(".svg") {
                    output.clone()
                } else {
                    output.replace(".png", ".svg")
                };
                std::fs::write(&svg_path, &svg_content)?;
                println!("QR code (SVG) saved: {}", svg_path);
                Ok(svg_path)
            }
            QrOutputFormat::Png => {
                let img = self.generate()?;
                self.save(&img, &output)?;
                println!("QR code (PNG) saved: {}", output);
                Ok(output)
            }
        }
    }
}

fn parse_hex_color(hex: &str) -> [u8; 4] {
    let clean  = hex.trim_start_matches('#');
    let padded = if clean.len() == 6 { format!("{}FF", clean) } else { clean.to_string() };
    let r = u8::from_str_radix(padded.get(0..2).unwrap_or("00"), 16).unwrap_or(0);
    let g = u8::from_str_radix(padded.get(2..4).unwrap_or("00"), 16).unwrap_or(0);
    let b = u8::from_str_radix(padded.get(4..6).unwrap_or("00"), 16).unwrap_or(0);
    let a = u8::from_str_radix(padded.get(6..8).unwrap_or("FF"), 16).unwrap_or(255);
    [r, g, b, a]
}

pub fn generate_qr_from_env() -> Result<String, Box<dyn std::error::Error>> {
    QrHandler::from_env().generate_and_save()
}

pub fn generate_qr_number(number: u64) -> Result<String, Box<dyn std::error::Error>> {
    let base = env::var("QR_BASE_URL").unwrap_or_else(|_| "https://example.com/verify/".to_string());
    let content = format!("{}{}", base, number);
    let mut config = QrConfig::from_env();
    config.content = content;
    QrHandler::new(config).generate_and_save()
}

pub fn generate_qr_link(link: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = QrConfig::from_env();
    config.content = link.to_string();
    QrHandler::new(config).generate_and_save()
}

pub fn generate_qr_with_logo(content: &str, logo_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = QrConfig::from_env();
    config.content    = content.to_string();
    config.logo_path  = Some(logo_path.to_string());
    QrHandler::new(config).generate_and_save()
}
