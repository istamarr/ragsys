use std::env;
use agent_difusser::qrcode::qrcode_handler::{
    QrConfig, QrHandler, QrOutputFormat,
    generate_qr_from_env, generate_qr_number, generate_qr_link,
};

fn main() {
    println!("=== DIFSR QR Code Agent ===");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    match args[1].as_str() {
        "env"    => run_from_env(),
        "number" => run_number(&args[2..]),
        "link"   => run_link(&args[2..]),
        "custom" => run_custom(&args[2..]),
        _        => {
            println!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!();
    println!("USAGE:");
    println!("  cargo run --bin agent_qr <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  env      Generate QR from environment variables");
    println!("  number   Generate QR for a number (appended to QR_BASE_URL)");
    println!("  link     Generate QR for a URL/link");
    println!("  custom   Generate QR with full options");
    println!();
    println!("ENV VARS:");
    println!("  QR_CONTENT        QR code content (URL, text, number)");
    println!("  QR_BASE_URL       Base URL prepended to number  [default: https://example.com/verify/]");
    println!("  QR_SIZE           Output image size in px       [default: 512]");
    println!("  QR_QUIET_ZONE     Quiet zone cell count         [default: 4]");
    println!("  QR_FG_COLOR       Foreground hex color RRGGBBAA [default: 000000FF]");
    println!("  QR_BG_COLOR       Background hex color RRGGBBAA [default: FFFFFFFF]");
    println!("  QR_LOGO_PATH      Path to logo image (PNG/JPG)");
    println!("  QR_LOGO_SIZE_RATIO Logo size as fraction of QR  [default: 0.22]");
    println!("  QR_OUTPUT_PATH    Output file path              [default: rag_output/agent_diffuser/qrcode.png]");
    println!();
    println!("OPTIONS (for 'custom' command):");
    println!("  --content <TEXT>     QR content");
    println!("  --size <PX>          Image size");
    println!("  --fg <RRGGBBAA>      Foreground color");
    println!("  --bg <RRGGBBAA>      Background color");
    println!("  --logo <PATH>        Logo image path");
    println!("  --output <PATH>      Output path");
    println!("  --svg                Output as SVG instead of PNG");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin agent_qr env");
    println!("  cargo run --bin agent_qr number 123456");
    println!("  cargo run --bin agent_qr link https://example.com/product/42");
    println!("  cargo run --bin agent_qr custom --content \"TOKEN-001\" --logo brand.png --output qr_001.png");
}

fn run_from_env() {
    println!("Generating QR from environment variables...");
    match generate_qr_from_env() {
        Ok(path) => println!("QR saved: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn run_number(args: &[String]) {
    let number: u64 = args.first()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("Usage: agent_qr number <NUMBER>");
            std::process::exit(1);
        });

    println!("Generating QR for number: {}", number);
    let base = env::var("QR_BASE_URL").unwrap_or_else(|_| "https://example.com/verify/".to_string());
    println!("Base URL: {}", base);

    match generate_qr_number(number) {
        Ok(path) => println!("QR saved: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn run_link(args: &[String]) {
    let link = args.first().map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("Usage: agent_qr link <URL>");
        std::process::exit(1);
    });

    println!("Generating QR for link: {}", link);

    match generate_qr_link(link) {
        Ok(path) => println!("QR saved: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn run_custom(args: &[String]) {
    let mut config = QrConfig::from_env();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--content" if i + 1 < args.len() => { config.content = args[i + 1].clone(); i += 2; }
            "--size" if i + 1 < args.len() => { config.size = args[i + 1].parse().unwrap_or(512); i += 2; }
            "--fg" if i + 1 < args.len() => {
                config.fg_color = parse_hex_color_arg(&args[i + 1]);
                i += 2;
            }
            "--bg" if i + 1 < args.len() => {
                config.bg_color = parse_hex_color_arg(&args[i + 1]);
                i += 2;
            }
            "--logo" if i + 1 < args.len() => { config.logo_path = Some(args[i + 1].clone()); i += 2; }
            "--output" if i + 1 < args.len() => { config.output_path = args[i + 1].clone(); i += 2; }
            "--svg" => { config.format = QrOutputFormat::Svg; i += 1; }
            _ => { println!("Unknown arg: {}", args[i]); i += 1; }
        }
    }

    println!("Content : {}", config.content);
    println!("Size    : {}px", config.size);
    println!("Logo    : {:?}", config.logo_path);
    println!("Output  : {}", config.output_path);

    let handler = QrHandler::new(config);
    match handler.generate_and_save() {
        Ok(path) => println!("QR saved: {}", path),
        Err(e)   => eprintln!("Error: {}", e),
    }
}

fn parse_hex_color_arg(hex: &str) -> [u8; 4] {
    let clean = hex.trim_start_matches('#');
    let padded = if clean.len() == 6 { format!("{}FF", clean) } else { clean.to_string() };
    let r = u8::from_str_radix(&padded[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&padded[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&padded[4..6], 16).unwrap_or(0);
    let a = u8::from_str_radix(&padded[6..8], 16).unwrap_or(255);
    [r, g, b, a]
}
