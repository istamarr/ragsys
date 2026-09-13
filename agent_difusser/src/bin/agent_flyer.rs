use std::env;
use agent_difusser::handler::dfsr_img::{FlyerConfig, generate_flyer_image};
use image::Rgb;

fn main() {
    println!("=== DIFSR Flyer Agent ===");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    match args[1].as_str() {
        "generate" => run_generate(&args[2..]),
        "preview"  => run_preview(),
        _          => {
            println!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!();
    println!("USAGE:");
    println!("  cargo run --bin agent_flyer <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  generate   Generate flyer from env/args");
    println!("  preview    Generate sample flyer with default values");
    println!();
    println!("OPTIONS (override env vars inline):");
    println!("  --title <TITLE>          Hero title text  [env: FLYER_TITLE]");
    println!("  --subtitle <TEXT>        Hero subtitle    [env: FLYER_SUBTITLE]");
    println!("  --headline <TEXT>        Body headline    [env: FLYER_HEADLINE]");
    println!("  --cta <TEXT>             CTA text         [env: FLYER_CTA]");
    println!("  --contact <NUMBER>       Footer contact   [env: FLYER_CONTACT]");
    println!("  --website <URL>          Footer website   [env: FLYER_WEBSITE]");
    println!("  --logo-left <PATH>       Left header logo [env: FLYER_LOGO_LEFT]");
    println!("  --logo-right <PATH>      Right header logo[env: FLYER_LOGO_RIGHT]");
    println!("  --product <PATH>         Product image    [env: FLYER_PRODUCT_IMAGE]");
    println!("  --qr <PATH>              Footer QR image  [env: FLYER_QR_IMAGE]");
    println!("  --font <PATH>            TTF font path    [env: FLYER_FONT_PATH]");
    println!("  --output <PATH>          Output PNG path  [env: FLYER_OUTPUT]");
    println!("  --width <W>              Width in px      [env: FLYER_WIDTH]");
    println!("  --height <H>             Height in px     [env: FLYER_HEIGHT]");
    println!("  --ai                     Apply noise AI bg[env: FLYER_WITH_AI=true]");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin agent_flyer generate --title \"SALE!\" --headline \"50% OFF TODAY\"");
    println!("  cargo run --bin agent_flyer generate --logo-left logo.png --qr qr.png --output flyer.png");
    println!("  cargo run --bin agent_flyer preview");
}

fn run_generate(args: &[String]) {
    let mut config = FlyerConfig::default();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--title" if i + 1 < args.len() => { config.title = args[i + 1].clone(); i += 2; }
            "--subtitle" if i + 1 < args.len() => { config.subtitle = args[i + 1].clone(); i += 2; }
            "--headline" if i + 1 < args.len() => { config.headline = args[i + 1].clone(); i += 2; }
            "--subheadline" if i + 1 < args.len() => { config.subheadline = args[i + 1].clone(); i += 2; }
            "--body" if i + 1 < args.len() => { config.body_text = args[i + 1].clone(); i += 2; }
            "--cta" if i + 1 < args.len() => { config.cta_text = args[i + 1].clone(); i += 2; }
            "--contact" if i + 1 < args.len() => { config.footer_contact = args[i + 1].clone(); i += 2; }
            "--website" if i + 1 < args.len() => { config.footer_website = args[i + 1].clone(); i += 2; }
            "--logo-left" if i + 1 < args.len() => { config.header_left_logo_path = Some(args[i + 1].clone()); i += 2; }
            "--logo-right" if i + 1 < args.len() => { config.header_right_logo_path = Some(args[i + 1].clone()); i += 2; }
            "--product" if i + 1 < args.len() => { config.body_product_image_path = Some(args[i + 1].clone()); i += 2; }
            "--qr" if i + 1 < args.len() => { config.footer_qr_image_path = Some(args[i + 1].clone()); i += 2; }
            "--font" if i + 1 < args.len() => { config.font_path = Some(args[i + 1].clone()); i += 2; }
            "--output" if i + 1 < args.len() => { config.output_path = args[i + 1].clone(); i += 2; }
            "--width" if i + 1 < args.len() => { config.width = args[i + 1].parse().unwrap_or(600); i += 2; }
            "--height" if i + 1 < args.len() => { config.height = args[i + 1].parse().unwrap_or(850); i += 2; }
            "--ai" => { config.with_ai_background = true; i += 1; }
            _ => { println!("Unknown arg: {}", args[i]); i += 1; }
        }
    }

    print_config_summary(&config);

    match generate_flyer_image(config) {
        Ok(path) => println!("Flyer generated: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn run_preview() {
    println!("Generating preview flyer with sample content...");

    let config = FlyerConfig {
        title:                    "GIVEAWAY".to_string(),
        subtitle:                 "Tring! Digital Lounge".to_string(),
        headline:                 "MAU EMAS GRATIS?".to_string(),
        subheadline:              "Mampir ke Mall Kota Kasablanka Jakarta".to_string(),
        body_text:                "Untuk 2 orang terpilih mendapatkan saldo TABUNGAN EMAS 0,5 Gram.".to_string(),
        bullet_points:            vec![
            "Datang ke Tring! Digital Lounge".to_string(),
            "Scan QR Code".to_string(),
            "Isi Formulir".to_string(),
            "Jawab Kuis".to_string(),
        ],
        cta_text:                 "Untuk 2 orang terpilih akan mendapatkan saldo TABUNGAN EMAS 0,5 Gram setiap akhir minggu.".to_string(),
        footer_contact:           "1500 569".to_string(),
        footer_website:           "sahabat.example.co.id | www.example.co.id".to_string(),
        hero_bg_color:            Rgb([218, 165, 32]),
        hero_accent_color:        Rgb([255, 215, 0]),
        accent_color:             Rgb([34, 139, 34]),
        output_path:              format!(
            "rag_output/agent_diffuser/flyer_preview_{}.png",
            chrono::Local::now().format("%Y%m%d%H%M%S")
        ),
        ..FlyerConfig::default()
    };

    print_config_summary(&config);

    match generate_flyer_image(config) {
        Ok(path) => println!("Preview flyer saved: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn print_config_summary(config: &FlyerConfig) {
    println!("  Size     : {}x{}", config.width, config.height);
    println!("  Title    : {}", config.title);
    println!("  Headline : {}", config.headline);
    println!("  Output   : {}", config.output_path);
    println!("  Font     : {}", config.font_path.as_deref().unwrap_or("(none — text skipped)"));
    println!("  AI bg    : {}", config.with_ai_background);
}
