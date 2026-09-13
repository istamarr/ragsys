use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use log::info;
use std::convert::Infallible;

use crate::handler::dfsr_img::{ImageConfig, RustDiffuser, GenerationType, ColorScheme};
use crate::handler::documents::{report_pdf, report_generate, report_notes};

#[derive(Debug, Deserialize)]
pub struct ImageRequest {
    pub img_type: String,        // "generate", "logo", "art"
    pub gen_type: String,        // "diffusion", "plasma", "circular", etc.
    pub color: Option<String>,   // "rainbow", "ocean", etc.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocRequest {
    pub doc_type: String,        // "pdf", "generate", "notes"
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
    pub output_path: Option<String>,
}

fn parse_generation_type(s: &str) -> GenerationType {
    match s.to_lowercase().as_str() {
        "noise" => GenerationType::NoisePattern,
        "mandelbrot" | "fractal" => GenerationType::FractalMandelbrot,
        "julia" => GenerationType::FractalJulia,
        "perlin" => GenerationType::PerlinNoise,
        "voronoi" => GenerationType::VoronoiDiagram,
        "diffusion" => GenerationType::DiffusionPattern,
        "geometric" => GenerationType::GeometricArt,
        "plasma" => GenerationType::PlasmaEffect,
        "waves" => GenerationType::WaveInterference,
        "cellular" => GenerationType::CellularAutomata,
        "vector" => GenerationType::VectorLines,
        "circular" | "circle" => GenerationType::CircularLogo,
        "polygon" | "hex" => GenerationType::PolygonLogo,
        "minimal" | "lines" => GenerationType::MinimalLines,
        "grid" | "tech" => GenerationType::TechGrid,
        _ => GenerationType::DiffusionPattern,
    }
}

fn parse_color_scheme(s: &str) -> ColorScheme {
    match s.to_lowercase().as_str() {
        "grayscale" | "gray" => ColorScheme::Grayscale,
        "rainbow" => ColorScheme::Rainbow,
        "sunset" => ColorScheme::Sunset,
        "ocean" => ColorScheme::Ocean,
        "forest" => ColorScheme::Forest,
        "fire" => ColorScheme::Fire,
        "electric" => ColorScheme::Electric,
        "pastel" => ColorScheme::Pastel,
        _ => ColorScheme::Rainbow,
    }
}

async fn handle_image(req: ImageRequest) -> Result<impl Reply, Infallible> {
    info!("HTTP /api/img - type: {}, gen: {}", req.img_type, req.gen_type);
    
    let output_path = req.output.unwrap_or_else(|| {
        format!("rag_output/agent_diffuser/img_{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"))
    });
    
    // Ensure output directory exists
    std::fs::create_dir_all("../../../../../../rag_output/agent_diffuser").ok();
    
    let gen_type = match req.img_type.as_str() {
        "logo" => match req.gen_type.as_str() {
            "circular" | "circle" => GenerationType::CircularLogo,
            "polygon" | "hex" => GenerationType::PolygonLogo,
            "minimal" | "lines" => GenerationType::MinimalLines,
            "grid" | "tech" => GenerationType::TechGrid,
            "vector" => GenerationType::VectorLines,
            _ => GenerationType::CircularLogo,
        },
        "art" => parse_generation_type(&req.gen_type),
        _ => parse_generation_type(&req.gen_type),
    };
    
    let config = ImageConfig {
        width: req.width.unwrap_or(512),
        height: req.height.unwrap_or(512),
        output_path: output_path.clone(),
        generation_type: gen_type,
        seed: None,
        iterations: 100,
        color_scheme: parse_color_scheme(&req.color.unwrap_or_else(|| "rainbow".to_string())),
    };
    
    let mut diffuser = RustDiffuser::new(config);

    println!("save image into path");
    let response = match diffuser.generate_image() {
        Ok(image) => {
            match diffuser.save_image(&image,&output_path) {
                Ok(_) => ApiResponse {
                    status: "success".to_string(),
                    message: "Image generated successfully".to_string(),
                    output_path: Some(output_path),
                },
                Err(e) => ApiResponse {
                    status: "error".to_string(),
                    message: format!("Failed to save image: {}", e),
                    output_path: None,
                },
            }
        },
        Err(e) => ApiResponse {
            status: "error".to_string(),
            message: format!("Failed to generate image: {}", e),
            output_path: None,
        },
    };
    
    Ok(warp::reply::json(&response))
}

async fn handle_document(req: DocRequest) -> Result<impl Reply, Infallible> {
    info!("HTTP /api/doc - type: {}", req.doc_type);
    
    let output_path = req.output.unwrap_or_else(|| {
        format!("rag_output/agent_diffuser/doc_{}.txt", chrono::Local::now().format("%Y%m%d%H%M%S"))
    });
    
    // Ensure output directory exists
    std::fs::create_dir_all("../../../../../../rag_output/agent_diffuser").ok();
    
    let result = match req.doc_type.as_str() {
        "pdf" => {
            report_pdf();
            let content = format!(
                "PDF Report Generated\n===================\nInput: {}\nOutput: {}\nGenerated: {}\n",
                req.input.as_deref().unwrap_or("(none)"),
                output_path,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            std::fs::write(&output_path, content)
        },
        "notes" => {
            report_notes();
            let input_content = req.input.as_ref()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .unwrap_or_default();
            let content = format!(
                "# Notes\n\nGenerated: {}\n\n## Source\n{}\n\n## Extracted Notes\n- Processing complete\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                if input_content.is_empty() { "(No input)" } else { &input_content }
            );
            std::fs::write(&output_path, content)
        },
        _ => {
            report_generate();
            let content = format!(
                "# Document Report\n\nGenerated: {}\n\n## Summary\nAuto-generated report from DIFSR.\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            std::fs::write(&output_path, content)
        },
    };
    
    let response = match result {
        Ok(_) => ApiResponse {
            status: "success".to_string(),
            message: format!("Document {} generated", req.doc_type),
            output_path: Some(output_path),
        },
        Err(e) => ApiResponse {
            status: "error".to_string(),
            message: format!("Failed: {}", e),
            output_path: None,
        },
    };
    
    Ok(warp::reply::json(&response))
}

async fn handle_health() -> Result<impl Reply, Infallible> {
    let response = ApiResponse {
        status: "ok".to_string(),
        message: "DIFSR service running".to_string(),
        output_path: None,
    };
    Ok(warp::reply::json(&response))
}

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let health = warp::path!("api" / "health")
        .and(warp::get())
        .and_then(handle_health);
    
    let img = warp::path!("api" / "img")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_image);
    
    let doc = warp::path!("api" / "doc")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_document);
    
    health.or(img).or(doc)
}

pub async fn start_server(port: u16) {
    info!("Starting DIFSR HTTP server on port {}", port);
    println!("=== DIFSR Service ===");
    println!("Listening on http://0.0.0.0:{}", port);
    println!("Endpoints:");
    println!("  GET  /api/health");
    println!("  POST /api/img");
    println!("  POST /api/doc");
    
    warp::serve(routes())
        .run(([0, 0, 0, 0], port))
        .await;
}
