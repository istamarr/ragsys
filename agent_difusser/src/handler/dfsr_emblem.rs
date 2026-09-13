use std::fs;
use std::io::Write;
use std::path::Path;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::{draw_filled_rect_mut, draw_filled_circle_mut};
use imageproc::rect::Rect;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

pub const DEFAULT_OUTPUT_DIR: &str =
    r"C:\Users\thinkpad123\TCPNRS\DEV_ISTA\rag_system\rag_output\agent_diffuser_output";
pub const DEFAULT_EMBLEM_NAME: &str =
    "conversation_communication_bubble_comment_message_chat_icon_260636";

// ── Source mode ───────────────────────────────────────────────────────────────

pub enum EmblemSource<'a> {
    /// Generate locally with imageproc drawing primitives (no network needed)
    Vector,
    /// Fetch PNG from an OpenAI-compatible image-generation API
    AiRequest { prompt: &'a str, api_url: &'a str, api_key: &'a str },
}

// ── PNG — vector path (imageproc) ─────────────────────────────────────────────

pub fn create_emblem_png_vector(dir: &Path, name: &str) -> std::io::Result<()> {
    let path = dir.join(format!("{}.png", name));
    if path.exists() {
        return Ok(());
    }

    let size: u32 = 64;
    let blue  = Rgba([33u8, 150, 243, 255]);  // #2196F3
    let white = Rgba([255u8, 255, 255, 255]);
    let clear = Rgba([0u8, 0, 0, 0]);

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(size, size, clear);

    // ── rounded-rect bubble body ──────────────────────────────────────────────
    // Three overlapping filled rects + four corner circles simulate rounded corners.
    let r = 8i32;
    // vertical centre strip (avoids corner areas)
    draw_filled_rect_mut(
        &mut img,
        Rect::at(4 + r, 4).of_size(56 - 2 * r as u32, 44),
        blue,
    );
    // horizontal centre strip
    draw_filled_rect_mut(
        &mut img,
        Rect::at(4, 4 + r).of_size(56, 44 - 2 * r as u32),
        blue,
    );
    // four rounded corners
    for &(cx, cy) in &[(4 + r, 4 + r), (60 - r, 4 + r), (4 + r, 48 - r), (60 - r, 48 - r)] {
        draw_filled_circle_mut(&mut img, (cx, cy), r, blue);
    }

    // ── tail triangle — scanline fill: vertices (8,48),(20,48),(8,58) ─────
    for row in 48i32..59 {
        let x_right = 8 + (58 - row); // right edge of the right-angle triangle
        for col in 8..=x_right {
            if col >= 0 && col < size as i32 && row >= 0 && row < size as i32 {
                img.put_pixel(col as u32, row as u32, blue);
            }
        }
    }

    // ── three dots inside bubble ──────────────────────────────────────────────
    for &cx in &[22i32, 32, 42] {
        draw_filled_circle_mut(&mut img, (cx, 26), 3, white);
    }

    img.save(&path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

// ── PNG — AI request path (async, OpenAI-compatible API) ─────────────────────

pub async fn create_emblem_png_ai(
    dir: &Path,
    name: &str,
    prompt: &str,
    api_url: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = dir.join(format!("{}.png", name));
    if path.exists() {
        return Ok(());
    }

    let client = reqwest::Client::new();
    let url = format!("{}/v1/images/generations", api_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "prompt": prompt,
            "n": 1,
            "size": "256x256",
            "response_format": "b64_json"
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let b64 = resp["data"][0]["b64_json"]
        .as_str()
        .ok_or("No b64_json field in API response")?;

    let bytes = BASE64.decode(b64)?;
    fs::write(&path, &bytes)?;
    println!("  AI PNG saved → {}", path.display());
    Ok(())
}

// ── ICO — embeds existing PNG (modern ICO with PNG payload) ───────────────────

pub fn create_emblem_ico(dir: &Path, name: &str) -> std::io::Result<()> {
    let ico_path = dir.join(format!("{}.ico", name));
    if ico_path.exists() {
        return Ok(());
    }

    let png_path = dir.join(format!("{}.png", name));
    if !png_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "PNG must exist before creating ICO",
        ));
    }
    let png_bytes = fs::read(&png_path)?;

    let mut ico: Vec<u8> = Vec::new();
    // ICONDIR  (6 bytes)
    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    // ICONDIRENTRY (16 bytes)
    ico.push(64); ico.push(64);                                      // width, height
    ico.push(0);  ico.push(0);                                       // color count, reserved
    ico.extend_from_slice(&[1u8, 0]);                                // planes
    ico.extend_from_slice(&[32u8, 0]);                               // bits per pixel
    ico.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());  // image data size
    ico.extend_from_slice(&22u32.to_le_bytes());                     // offset = 6+16
    // PNG payload
    ico.extend_from_slice(&png_bytes);

    let mut f = fs::File::create(&ico_path)?;
    f.write_all(&ico)
}

// ── SVG ───────────────────────────────────────────────────────────────────────

pub fn create_emblem_svg(dir: &Path, name: &str) -> std::io::Result<()> {
    let path = dir.join(format!("{}.svg", name));
    if path.exists() {
        return Ok(());
    }

    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">
  <rect x="4" y="4" width="56" height="44" rx="8" ry="8" fill="#2196F3"/>
  <polygon points="8,48 20,48 8,58" fill="#2196F3"/>
  <circle cx="22" cy="26" r="3" fill="white"/>
  <circle cx="32" cy="26" r="3" fill="white"/>
  <circle cx="42" cy="26" r="3" fill="white"/>
</svg>"##;

    let mut f = fs::File::create(&path)?;
    f.write_all(svg.as_bytes())
}

// ── HTML — inline SVG preview page ───────────────────────────────────────────

pub fn create_emblem_html(dir: &Path, name: &str) -> std::io::Result<()> {
    let path = dir.join(format!("{}.html", name));
    if path.exists() {
        return Ok(());
    }

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Chat Emblem</title>
  <style>
    body {{ display:flex; align-items:center; justify-content:center;
            min-height:100vh; margin:0; background:#f0f4f8; font-family:sans-serif; }}
    .card {{ text-align:center; padding:32px 40px; background:#fff;
             border-radius:16px; box-shadow:0 6px 24px rgba(0,0,0,.10); }}
    h3 {{ color:#555; margin:16px 0 0; font-size:11px;
          word-break:break-all; max-width:300px; }}
  </style>
</head>
<body>
  <div class="card">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="128" height="128">
      <rect x="4" y="4" width="56" height="44" rx="8" ry="8" fill="#2196F3"/>
      <polygon points="8,48 20,48 8,58" fill="#2196F3"/>
      <circle cx="22" cy="26" r="3" fill="white"/>
      <circle cx="32" cy="26" r="3" fill="white"/>
      <circle cx="42" cy="26" r="3" fill="white"/>
    </svg>
    <h3>{name}</h3>
  </div>
</body>
</html>"##,
        name = name
    );

    let mut f = fs::File::create(&path)?;
    f.write_all(html.as_bytes())
}

// ── Scanner + orchestrator ────────────────────────────────────────────────────

/// Scan `dir` for `name`.{png,ico,svg,html}; create any that are missing.
/// `source` controls how the PNG is generated when absent.
///
/// For `EmblemSource::AiRequest` the PNG creation is async — call
/// `ensure_emblem_files_ai` instead.
pub fn ensure_emblem_files(dir: &str, name: &str, source: EmblemSource) {
    let dir_path = Path::new(dir);
    if let Err(e) = fs::create_dir_all(dir_path) {
        eprintln!("  ERROR creating directory: {}", e);
        return;
    }

    println!("Scanning: {}", dir);
    println!("{:<6} {:<58} {}", "TYPE", "FILE", "STATUS");
    println!("{}", "-".repeat(72));

    for ext in &["png", "ico", "svg", "html"] {
        let p = dir_path.join(format!("{}.{}", name, ext));
        if p.exists() {
            println!("{:<6} {:<58} EXISTS", ext.to_uppercase(), format!("{}.{}", name, ext));
            continue;
        }

        let result: std::io::Result<()> = match *ext {
            "png" => match source {
                EmblemSource::Vector => create_emblem_png_vector(dir_path, name),
                EmblemSource::AiRequest { .. } => {
                    println!("{:<6} {:<58} SKIPPED (use ensure_emblem_files_ai for AI path)",
                        "PNG", format!("{}.png", name));
                    continue;
                }
            },
            "ico"  => { let _ = create_emblem_png_vector(dir_path, name); create_emblem_ico(dir_path, name) }
            "svg"  => create_emblem_svg(dir_path, name),
            "html" => create_emblem_html(dir_path, name),
            _      => Ok(()),
        };

        match result {
            Ok(_)  => println!("{:<6} {:<58} CREATED", ext.to_uppercase(), format!("{}.{}", name, ext)),
            Err(e) => println!("{:<6} {:<58} ERROR: {}", ext.to_uppercase(), format!("{}.{}", name, ext), e),
        }
    }
}

/// Async variant — generates PNG via AI API, then creates ICO/SVG/HTML.
pub async fn ensure_emblem_files_ai(
    dir: &str,
    name: &str,
    prompt: &str,
    api_url: &str,
    api_key: &str,
) {
    let dir_path = Path::new(dir);
    if let Err(e) = fs::create_dir_all(dir_path) {
        eprintln!("  ERROR creating directory: {}", e);
        return;
    }

    println!("Scanning (AI mode): {}", dir);

    // PNG via AI
    let png_path = dir_path.join(format!("{}.png", name));
    if png_path.exists() {
        println!("  PNG  : EXISTS");
    } else {
        match create_emblem_png_ai(dir_path, name, prompt, api_url, api_key).await {
            Ok(_)  => println!("  PNG  : CREATED (AI)"),
            Err(e) => {
                println!("  PNG  : ERROR ({}) — falling back to vector", e);
                match create_emblem_png_vector(dir_path, name) {
                    Ok(_)  => println!("  PNG  : CREATED (vector fallback)"),
                    Err(e) => println!("  PNG  : ERROR (vector fallback): {}", e),
                }
            }
        }
    }

    // ICO / SVG / HTML (sync)
    for (label, result) in [
        ("ICO ", create_emblem_ico(dir_path, name)),
        ("SVG ", create_emblem_svg(dir_path, name)),
        ("HTML", create_emblem_html(dir_path, name)),
    ] {
        match result {
            Ok(_)  => println!("  {}  : OK", label),
            Err(e) => println!("  {}  : ERROR: {}", label, e),
        }
    }
}
