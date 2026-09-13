use std::env;
use agent_difusser::handler::dfsr_img::{ImageConfig, RustDiffuser, GenerationType, ColorScheme};

fn main() {
    println!("=== DIFSR Image Agent ===");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }
    
    match args[1].as_str() {
        "generate" => {
            let config = parse_config(&args[2..]);
            run_generate(config);
        },
        "logo" => {
            let logo_type = args.get(2).map(|s| s.as_str()).unwrap_or("circular");
            run_logo(logo_type, &args[3..]);
        },
        "art" => {
            let art_type = args.get(2).map(|s| s.as_str()).unwrap_or("diffusion");
            run_art(art_type, &args[3..]);
        },
        _ => {
            println!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!();
    println!("USAGE:");
    println!("  cargo run --bin agent_img <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  generate   Generate image with full options");
    println!("  logo       Generate logo (circular, polygon, minimal, grid)");
    println!("  art        Generate art (diffusion, plasma, waves, fractal)");
    println!();
    println!("OPTIONS:");
    println!("  --type <TYPE>      Generation type");
    println!("  --width <WIDTH>    Image width [default: 512]");
    println!("  --height <HEIGHT>  Image height [default: 512]");
    println!("  --output <PATH>    Output file path");
    println!("  --color <SCHEME>   Color scheme (rainbow, ocean, fire, etc.)");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin agent_img generate --type diffusion --color rainbow");
    println!("  cargo run --bin agent_img logo circular --output logo.png");
    println!("  cargo run --bin agent_img art plasma --width 1024 --height 1024");
}

fn parse_config(args: &[String]) -> ImageConfig {
    let mut config = ImageConfig::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--type" if i + 1 < args.len() => {
                config.generation_type = match args[i + 1].to_lowercase().as_str() {
                    "noise" => GenerationType::NoisePattern,
                    "mandelbrot" => GenerationType::FractalMandelbrot,
                    "julia" => GenerationType::FractalJulia,
                    "perlin" => GenerationType::PerlinNoise,
                    "voronoi" => GenerationType::VoronoiDiagram,
                    "diffusion" => GenerationType::DiffusionPattern,
                    "geometric" => GenerationType::GeometricArt,
                    "plasma" => GenerationType::PlasmaEffect,
                    "waves" => GenerationType::WaveInterference,
                    "cellular" => GenerationType::CellularAutomata,
                    _ => GenerationType::DiffusionPattern,
                };
                i += 2;
            },
            "--width" if i + 1 < args.len() => {
                config.width = args[i + 1].parse().unwrap_or(512);
                i += 2;
            },
            "--height" if i + 1 < args.len() => {
                config.height = args[i + 1].parse().unwrap_or(512);
                i += 2;
            },
            "--output" if i + 1 < args.len() => {
                config.output_path = args[i + 1].clone();
                i += 2;
            },
            "--color" if i + 1 < args.len() => {
                config.color_scheme = match args[i + 1].to_lowercase().as_str() {
                    "grayscale" | "gray" => ColorScheme::Grayscale,
                    "rainbow" => ColorScheme::Rainbow,
                    "sunset" => ColorScheme::Sunset,
                    "ocean" => ColorScheme::Ocean,
                    "forest" => ColorScheme::Forest,
                    "fire" => ColorScheme::Fire,
                    "electric" => ColorScheme::Electric,
                    "pastel" => ColorScheme::Pastel,
                    _ => ColorScheme::Rainbow,
                };
                i += 2;
            },
            _ => i += 1,
        }
    }
    
    config
}

fn run_generate(config: ImageConfig) {
    println!("Generating image...");
    println!("  Type: {:?}", config.generation_type);
    println!("  Size: {}x{}", config.width, config.height);
    println!("  Color: {:?}", config.color_scheme);
    println!("  Output: {}", config.output_path);
    
    let mut diffuser = RustDiffuser::new(config.clone());
    let output_path =  format!("rag_output/agent_diffuser/img_{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"));
    match diffuser.generate_image() {
        Ok(image) => {
            if let Err(e) = diffuser.save_image(&image, &output_path) {
                eprintln!("Error saving image: {}", e);
            } else {
                println!("Image generated successfully: {}", config.output_path);
            }
        },
        Err(e) => eprintln!("Error generating image: {}", e),
    }
}

fn run_logo(logo_type: &str, args: &[String]) {
    let mut config = parse_config(args);
    
    config.generation_type = match logo_type {
        "circular" | "circle" => GenerationType::CircularLogo,
        "polygon" | "hex" => GenerationType::PolygonLogo,
        "minimal" | "lines" => GenerationType::MinimalLines,
        "grid" | "tech" => GenerationType::TechGrid,
        "vector" => GenerationType::VectorLines,
        _ => GenerationType::CircularLogo,
    };
    
    if config.output_path == "rag_output/agent_diffuser/generated_image.png" {
        config.output_path = format!("rag_output/agent_diffuser/logo_{}.png", logo_type);
    }
    
    println!("Generating {} logo...", logo_type);
    run_generate(config);
}

fn run_art(art_type: &str, args: &[String]) {
    let mut config = parse_config(args);
    
    config.generation_type = match art_type {
        "diffusion" => GenerationType::DiffusionPattern,
        "plasma" => GenerationType::PlasmaEffect,
        "waves" => GenerationType::WaveInterference,
        "mandelbrot" | "fractal" => GenerationType::FractalMandelbrot,
        "julia" => GenerationType::FractalJulia,
        "geometric" => GenerationType::GeometricArt,
        "voronoi" => GenerationType::VoronoiDiagram,
        "cellular" => GenerationType::CellularAutomata,
        _ => GenerationType::DiffusionPattern,
    };
    
    if config.output_path == "rag_output/agent_diffuser/generated_image.png" {
        config.output_path = format!("rag_output/agent_diffuser/art_{}.png", art_type);
    }
    
    println!("Generating {} art...", art_type);
    run_generate(config);
}