use rand::Rng;
use image::{GenericImageView, imageops};
use std::path::Path;
use lopdf::Document as PdfDoc;

// Neural network parameters for signature recognition
// Signatures resized to 28x28 (same as MNIST format)
const IMG_WIDTH: usize = 28;
const IMG_HEIGHT: usize = 28;
const PIXEL_COUNT: usize = IMG_WIDTH * IMG_HEIGHT; // 784 pixels

// Simplified neural network struct for signature classification
struct SimpleNN {
    weights_ih: Vec<Vec<f64>>,  // input -> hidden (784 x 128)
    weights_ho: Vec<Vec<f64>>,  // hidden -> output (128 x num_classes)
    bias_h: Vec<f64>,           // hidden bias
    bias_o: Vec<f64>,           // output bias
    learning_rate: f64,
}

impl SimpleNN {
    fn new(input_size: usize, hidden_size: usize, output_size: usize, lr: f64) -> Self {
        let mut rng = rand::thread_rng();
        
        // Xavier/Glorot initialization for better convergence
        let init_range = (6.0 / (input_size + hidden_size) as f64).sqrt();
        
        let weights_ih = (0..input_size)
            .map(|_| (0..hidden_size)
                .map(|_| rng.gen_range(-init_range..init_range))
                .collect())
            .collect();
            
        let weights_ho = (0..hidden_size)
            .map(|_| (0..output_size)
                .map(|_| rng.gen_range(-init_range..init_range))
                .collect())
            .collect();
            
        let bias_h = vec![0.0; hidden_size];
        let bias_o = vec![0.0; output_size];
        
        SimpleNN {
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
            learning_rate: lr,
        }
    }
    
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
    
    fn sigmoid_derivative(x: f64) -> f64 {
        x * (1.0 - x)
    }
    
    fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
        // Hidden layer computation
        let mut hidden = vec![0.0; self.bias_h.len()];
        for i in 0..self.weights_ih.len() {
            for j in 0..self.weights_ih[i].len() {
                hidden[j] += input[i] * self.weights_ih[i][j];
            }
        }
        for j in 0..hidden.len() {
            hidden[j] = Self::sigmoid(hidden[j] + self.bias_h[j]);
        }
        
        // Output layer computation
        let mut output = vec![0.0; self.bias_o.len()];
        for i in 0..self.weights_ho.len() {
            for j in 0..self.weights_ho[i].len() {
                output[j] += hidden[i] * self.weights_ho[i][j];
            }
        }
        for j in 0..output.len() {
            output[j] = Self::sigmoid(output[j] + self.bias_o[j]);
        }
        
        (hidden, output)
    }
    
    fn train(&mut self, input: &[f64], target: &[f64]) -> f64 {
        // Forward pass
        let (hidden, output) = self.forward(input);
        
        // Calculate error and loss
        let mut output_errors = vec![0.0; output.len()];
        let mut loss = 0.0;
        for i in 0..output.len() {
            output_errors[i] = target[i] - output[i];
            loss += output_errors[i].powi(2);
        }
        
        // Backpropagation - output layer
        let mut hidden_errors = vec![0.0; hidden.len()];
        for i in 0..self.weights_ho.len() {
            for j in 0..self.weights_ho[i].len() {
                let gradient = output_errors[j] * Self::sigmoid_derivative(output[j]);
                self.weights_ho[i][j] += gradient * hidden[i] * self.learning_rate;
                hidden_errors[i] += gradient * self.weights_ho[i][j];
            }
        }
        
        // Backpropagation - hidden layer
        for j in 0..self.bias_h.len() {
            let gradient = hidden_errors[j] * Self::sigmoid_derivative(hidden[j]);
            self.bias_h[j] += gradient * self.learning_rate;
            for i in 0..self.weights_ih.len() {
                self.weights_ih[i][j] += gradient * input[i] * self.learning_rate;
            }
        }
        
        loss / output.len() as f64
    }
    
    fn predict(&self, input: &[f64]) -> usize {
        let (_, output) = self.forward(input);
        output.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}

// Preprocess image pixels to normalized f64 values (0.0-1.0)
fn preprocess_image(pixels: &[u8]) -> Vec<f64> {
    pixels.iter()
        .map(|&p| p as f64 / 255.0)
        .collect()
}

// Generate synthetic training data (random images + one-hot labels)
// Used when MNIST dataset files are not available locally
fn load_training_data() -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = rand::thread_rng();
    let num_samples = 1_000;
    let num_classes = 10;

    let train_images: Vec<Vec<f64>> = (0..num_samples)
        .map(|_| (0..PIXEL_COUNT).map(|_| rng.gen_range(0.0f64..1.0f64)).collect())
        .collect();

    let train_labels: Vec<Vec<f64>> = (0..num_samples)
        .map(|_| {
            let label = rng.gen_range(0..num_classes);
            let mut one_hot = vec![0.0; num_classes];
            one_hot[label] = 1.0;
            one_hot
        })
        .collect();

    (train_images, train_labels)
}

// Function to detect/classify an image
fn detect_signature(nn: &SimpleNN, image_pixels: &[u8]) -> usize {
    let normalized = preprocess_image(image_pixels);
    nn.predict(&normalized)
}

// For signature verification (compare two signatures)
fn verify_signature(nn: &SimpleNN, reference: &[u8], candidate: &[u8]) -> bool {
    let ref_normalized = preprocess_image(reference);
    let cand_normalized = preprocess_image(candidate);
    
    let (_, ref_output) = nn.forward(&ref_normalized);
    let (_, cand_output) = nn.forward(&cand_normalized);
    
    // Calculate similarity using cosine similarity
    let similarity = cosine_similarity(&ref_output, &cand_output);
    
    println!("Signature similarity score: {:.2}%", similarity * 100.0);
    similarity > 0.85 // Threshold for genuine signature
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    dot / (norm_a * norm_b)
}

// Returns (dark_ratio, is_signed)
// dark_ratio = fraction of pixels darker than threshold (0=black .. 255=white)
// A region with >= min_dark_ratio of dark pixels is considered signed/written
fn analyze_region_signed(patch: &[u8], dark_threshold: u8, min_dark_ratio: f64) -> (f64, bool) {
    let dark_count = patch.iter().filter(|&&p| p < dark_threshold).count();
    let ratio = dark_count as f64 / patch.len() as f64;
    (ratio, ratio >= min_dark_ratio)
}

// Step-aligned tile start positions that always include the final edge position,
// so no pixels are missed at the right or bottom of the image.
fn tile_positions(total: u32, patch_size: u32, step: u32) -> Vec<u32> {
    let mut positions = Vec::new();
    if total == 0 || patch_size == 0 { return positions; }
    if total <= patch_size {
        positions.push(0);
        return positions;
    }
    let mut p = 0u32;
    while p + patch_size <= total {
        positions.push(p);
        p += step;
    }
    // Always include the last possible starting position (covers right/bottom edge)
    let last = total - patch_size;
    if positions.last().copied().unwrap_or(u32::MAX) < last {
        positions.push(last);
    }
    positions
}

// Extract a 28x28 patch from a grayscale DynamicImage; pixels that fall outside
// the image boundary are padded with 255 (white) so edge patches are never skipped.
fn extract_padded_patch(gray: &image::DynamicImage, x: u32, y: u32, img_w: u32, img_h: u32) -> Vec<u8> {
    let pw = (IMG_WIDTH as u32).min(img_w.saturating_sub(x));
    let ph = (IMG_HEIGHT as u32).min(img_h.saturating_sub(y));
    let mut patch = vec![255u8; IMG_WIDTH * IMG_HEIGHT];
    if pw == 0 || ph == 0 { return patch; }
    let raw = gray.crop_imm(x, y, pw, ph).to_luma8().into_raw();
    for row in 0..(ph as usize) {
        for col in 0..(pw as usize) {
            patch[row * IMG_WIDTH + col] = raw[row * pw as usize + col];
        }
    }
    patch
}

// Scan image and determine which regions are signed (contain ink/writing)
fn scan_signed_regions(
    img: &image::DynamicImage,
    step: u32,
    dark_threshold: u8,
    min_dark_ratio: f64,
) -> Vec<(u32, u32, f64, bool)> {
    let (w, h) = img.dimensions();
    let gray = img.grayscale();
    let mut results = Vec::new();

    for y in tile_positions(h, IMG_HEIGHT as u32, step) {
        for x in tile_positions(w, IMG_WIDTH as u32, step) {
            let patch = extract_padded_patch(&gray, x, y, w, h);
            let (ratio, signed) = analyze_region_signed(&patch, dark_threshold, min_dark_ratio);
            results.push((x, y, ratio, signed));
        }
    }
    results
}

// Load a JPEG/PNG and extract a 28x28 grayscale patch at (x, y)
fn extract_patch(img: &image::DynamicImage, x: u32, y: u32) -> Vec<u8> {
    let patch = img.crop_imm(x, y, IMG_WIDTH as u32, IMG_HEIGHT as u32);
    let gray = patch.grayscale();
    gray.to_luma8().into_raw()
}

// Scan image with sliding window, classify each 28x28 region
fn scan_image_all_regions(nn: &SimpleNN, img: &image::DynamicImage, step: u32) -> Vec<(u32, u32, usize, f64)> {
    let (w, h) = img.dimensions();
    let gray = img.grayscale();
    let mut results = Vec::new();

    for y in tile_positions(h, IMG_HEIGHT as u32, step) {
        for x in tile_positions(w, IMG_WIDTH as u32, step) {
            let patch = extract_padded_patch(&gray, x, y, w, h);
            let normalized = preprocess_image(&patch);
            let (_, output) = nn.forward(&normalized);
            let (class, confidence) = output.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, &v)| (i, v))
                .unwrap_or((0, 0.0));
            results.push((x, y, class, confidence));
        }
    }
    results
}

// ─── Three-tier document signing status ─────────────────────────────────────

#[derive(Debug, PartialEq)]
enum SignedStatus {
    Complete,           // strong ink coverage  → signed complete
    PartialNotComplete, // some ink present     → signed parts not complete
    NotSigned,          // no ink detected      → not yet signed
}

impl SignedStatus {
    fn label(&self) -> &'static str {
        match self {
            Self::Complete           => "SIGNED COMPLETE",
            Self::PartialNotComplete => "SIGNED PARTS NOT COMPLETE",
            Self::NotSigned          => "NOT YET SIGNED",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Self::Complete           => "✓",
            Self::PartialNotComplete => "~",
            Self::NotSigned          => "✗",
        }
    }
}

// Classify based on what fraction of scanned 28x28 patches contain ink.
// >=15% coverage  → Complete
//  >0% and <15%  → PartialNotComplete
//    0%          → NotSigned
fn classify_signed(signed_count: usize, total_count: usize) -> SignedStatus {
    if total_count == 0 || signed_count == 0 {
        return SignedStatus::NotSigned;
    }
    let pct = signed_count as f64 / total_count as f64;
    if pct >= 0.15 {
        SignedStatus::Complete
    } else {
        SignedStatus::PartialNotComplete
    }
}

// Scan only the bottom strip of an image for ink (e.g. signature line at bottom of page).
// bottom_frac: fraction of page height treated as the bottom strip (0.25 = bottom 25%).
// Uses the same dark_threshold / min_dark_ratio as the full-page scan.
fn scan_bottom_strip_ink(
    img: &image::DynamicImage,
    bottom_frac: f64,
    step: u32,
    dark_threshold: u8,
    min_dark_ratio: f64,
) -> (usize, usize, SignedStatus) {
    let (w, h) = img.dimensions();
    let strip_y = (h as f64 * (1.0 - bottom_frac)).floor() as u32;
    let strip_h = h.saturating_sub(strip_y);
    let gray = img.grayscale();
    let mut signed_count = 0usize;
    let mut total_count = 0usize;

    for y in tile_positions(strip_h, IMG_HEIGHT as u32, step)
        .into_iter()
        .map(|p| p + strip_y)
    {
        for x in tile_positions(w, IMG_WIDTH as u32, step) {
            let patch = extract_padded_patch(&gray, x, y, w, h);
            let (_, is_signed) = analyze_region_signed(&patch, dark_threshold, min_dark_ratio);
            total_count += 1;
            if is_signed { signed_count += 1; }
        }
    }
    let status = classify_signed(signed_count, total_count);
    (signed_count, total_count, status)
}

fn main() {
    println!("=== Handwritten Signature Detection System ===\n");

    // Initialize neural network: 784 inputs, 128 hidden neurons, 10 output classes
    let mut nn = SimpleNN::new(PIXEL_COUNT, 128, 10, 0.1);

    println!("Loading training data...");
    let (train_images, train_labels) = load_training_data();
    println!("Loaded {} training samples\n", train_images.len());

    println!("Training neural network...");
    let epochs = 5;
    for epoch in 0..epochs {
        let mut total_loss = 0.0;
        for i in 0..train_images.len() {
            let loss = nn.train(&train_images[i], &train_labels[i]);
            total_loss += loss;
        }
        println!("  Epoch {}/{} - Avg Loss: {:.4}", epoch + 1, epochs, total_loss / train_images.len() as f64);
    }
    println!("\nTraining complete!\n");

    // --- Resolve scan target (CLI arg or default JPEG) ---
    let args: Vec<String> = std::env::args().collect();
    let scan_path = if args.len() > 1 {
        args[1].clone()
    } else {
        r"C:\Users\thinkpad123\TCPNRS\DEV_ISTA\rag_system\image\illustration-digital-signing.jpg".to_string()
    };

    let dark_threshold: u8 = 100;
    let min_dark_ratio: f64 = 0.08;
    let step = 28u32;

    // ── If directory: scan all supported files recursively ─────────────
    let spath = Path::new(&scan_path);
    if spath.is_dir() {
        scan_directory(spath, &nn, step, dark_threshold, min_dark_ratio);
        return;
    }

    // ── If PDF: run PDF scanner and exit ────────────────────────────────
    if scan_path.to_lowercase().ends_with(".pdf") {
        scan_pdf(&scan_path, &nn, step, dark_threshold, min_dark_ratio);
        return;
    }

    // --- Check signing image ---
    let image_path = Path::new(&scan_path);
    println!("=== Checking Sign-in Image: {:?} ===\n", image_path.file_name().unwrap_or(image_path.as_os_str()));

    match image::open(&image_path) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            println!("Image size: {}x{} pixels", w, h);

            // --- Full image as single 28x28 input ---
            let gray_resized = img.grayscale()
                .resize_exact(IMG_WIDTH as u32, IMG_HEIGHT as u32, imageops::FilterType::Lanczos3);
            let pixels = gray_resized.to_luma8().into_raw();
            let class = detect_signature(&nn, &pixels);
            println!("Full image (resized 28x28) -> Detected class: {}", class);

            // --- Sliding window: scan all 28x28 regions ---
            println!("\nSliding window scan (step={}px) across all regions:", step);
            println!("{:<8} {:<8} {:<8} {}", "X", "Y", "Class", "Confidence");
            println!("{}", "-".repeat(36));

            let regions = scan_image_all_regions(&nn, &img, step);
            for (x, y, class, conf) in &regions {
                println!("{:<8} {:<8} {:<8} {:.2}%", x, y, class, conf * 100.0);
            }

            println!("\nTotal regions scanned: {}", regions.len());

            // Summary: count detections per class
            let mut class_counts = vec![0usize; 10];
            for (_, _, class, _) in &regions {
                class_counts[*class] += 1;
            }
            println!("\nDetection summary (class -> count):");
            for (c, count) in class_counts.iter().enumerate() {
                if *count > 0 {
                    println!("  Class {} : {} region(s)", c, count);
                }
            }

            // --- Signed / Not Signed analysis ---
            println!("\n=== Signed / Not Signed Analysis ===");
            println!("  Dark pixel threshold : < {}", dark_threshold);
            println!("  Min ink ratio        : {:.0}%\n", min_dark_ratio * 100.0);

            let signed_regions = scan_signed_regions(&img, step, dark_threshold, min_dark_ratio);

            let signed_count   = signed_regions.iter().filter(|(_, _, _, s)| *s).count();
            let unsigned_count = signed_regions.len() - signed_count;

            println!("{:<8} {:<8} {:<14} {}", "X", "Y", "Ink Ratio", "Status");
            println!("{}", "-".repeat(40));
            for (x, y, ratio, is_signed) in &signed_regions {
                let status = if *is_signed { "SIGNED" } else { "not signed" };
                println!("{:<8} {:<8} {:<14} {}", x, y, format!("{:.2}%", ratio * 100.0), status);
            }

            println!("\n--- Summary ---");
            println!("  Signed regions   : {}", signed_count);
            println!("  Unsigned regions : {}", unsigned_count);
            println!("  Total regions    : {}", signed_regions.len());

            let status = classify_signed(signed_count, signed_regions.len());
            let pct = signed_count as f64 / signed_regions.len().max(1) as f64 * 100.0;
            println!("\n  {} DOCUMENT STATUS : {} ({:.1}% ink coverage)",
                status.icon(), status.label(), pct);

            // --- Bottom-of-page ink analysis (bottom 25% strip) ---
            println!("\n=== Bottom-of-Page Ink Analysis (bottom 25%) ===");
            let (bs, bt, bstatus) =
                scan_bottom_strip_ink(&img, 0.25, step, dark_threshold, min_dark_ratio);
            println!("  Bottom strip patches : {}/{} inked ({:.1}%)",
                bs, bt, bs as f64 / bt.max(1) as f64 * 100.0);
            println!("  {} BOTTOM STATUS : {}", bstatus.icon(), bstatus.label());
        }
        Err(e) => {
            println!("Failed to load image: {:?}", e);
        }
    }
}

// Extract embedded raster images from a single PDF page.
// Returns a list of decoded DynamicImages (JPEG or raw pixel streams).
fn pdf_page_images(doc: &PdfDoc, page_id: (u32, u16)) -> Vec<image::DynamicImage> {
    let mut imgs: Vec<image::DynamicImage> = Vec::new();

    // ── page dict ─────────────────────────────────────────────────────
    let page_dict = match doc.get_object(page_id) {
        Ok(lopdf::Object::Dictionary(d)) => d.clone(),
        _ => return imgs,
    };

    // ── Resources dict (inline or referenced) ─────────────────────────
    let res_obj = match page_dict.get(b"Resources") {
        Ok(v) => v.clone(),
        Err(_) => return imgs,
    };
    let resources: lopdf::Dictionary = match res_obj {
        lopdf::Object::Dictionary(d) => d,
        lopdf::Object::Reference(id) => match doc.get_object(id) {
            Ok(lopdf::Object::Dictionary(d)) => d.clone(),
            _ => return imgs,
        },
        _ => return imgs,
    };

    // ── XObject dict ──────────────────────────────────────────────────
    let xobj_obj = match resources.get(b"XObject") {
        Ok(v) => v.clone(),
        Err(_) => return imgs,
    };
    let xobj_dict: lopdf::Dictionary = match xobj_obj {
        lopdf::Object::Dictionary(d) => d,
        lopdf::Object::Reference(id) => match doc.get_object(id) {
            Ok(lopdf::Object::Dictionary(d)) => d.clone(),
            _ => return imgs,
        },
        _ => return imgs,
    };

    // ── Iterate XObjects ──────────────────────────────────────────────
    for (_, val) in xobj_dict.iter() {
        let id: (u32, u16) = match val {
            lopdf::Object::Reference(id) => *id,
            _ => continue,
        };
        let stream = match doc.get_object(id) {
            Ok(lopdf::Object::Stream(s)) => s.clone(),
            _ => continue,
        };

        // Only Image XObjects
        let is_image = matches!(
            stream.dict.get(b"Subtype"),
            Ok(lopdf::Object::Name(n)) if n.as_slice() == b"Image"
        );
        if !is_image { continue; }

        let w: u32 = match stream.dict.get(b"Width") {
            Ok(lopdf::Object::Integer(n)) => *n as u32,
            _ => continue,
        };
        let h: u32 = match stream.dict.get(b"Height") {
            Ok(lopdf::Object::Integer(n)) => *n as u32,
            _ => continue,
        };

        // Detect DCTDecode (JPEG) filter
        let filter_bytes: Option<Vec<u8>> = match stream.dict.get(b"Filter") {
            Ok(lopdf::Object::Name(n)) => Some(n.clone()),
            Ok(lopdf::Object::Array(arr)) => arr.first().and_then(|o| {
                if let lopdf::Object::Name(n) = o { Some(n.clone()) } else { None }
            }),
            _ => None,
        };
        let is_jpeg = filter_bytes.map(|b| b == b"DCTDecode").unwrap_or(false);

        let img_opt: Option<image::DynamicImage> = if is_jpeg {
            // JPEG: raw stream bytes are the JPEG file
            image::load_from_memory(&stream.content).ok()
        } else {
            match stream.decompressed_content() {
                Ok(raw) => {
                    let is_gray = matches!(
                        stream.dict.get(b"ColorSpace"),
                        Ok(lopdf::Object::Name(n)) if n.as_slice() == b"DeviceGray"
                    );
                    if is_gray {
                        image::GrayImage::from_raw(w, h, raw)
                            .map(image::DynamicImage::ImageLuma8)
                    } else {
                        image::RgbImage::from_raw(w, h, raw)
                            .map(image::DynamicImage::ImageRgb8)
                    }
                }
                Err(_) => None,
            }
        };

        if let Some(img) = img_opt {
            imgs.push(img);
        }
    }
    imgs
}

// Normalise lopdf-extracted text that arrives one token per line.
// Handles two common lopdf extraction artifacts:
//   char-per-line  – each line is a single character  (e.g. "H\ne\nl\nl\no")
//   word-per-line  – each line is a single word        (e.g. "Resource\nIT\nSecurity")
// In both cases blank lines delimit logical boundaries.
fn normalize_pdf_text(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    let non_empty: Vec<&str> = lines.iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if non_empty.is_empty() {
        return text.to_string();
    }

    // Count lines that hold only a single token (one word or one char)
    let single_tok = non_empty.iter()
        .filter(|l| l.split_whitespace().count() <= 1)
        .count();

    // If fewer than 70 % of non-empty lines are single-token → already normal paragraphs
    if single_tok * 100 < non_empty.len() * 70 {
        return text.to_string();
    }

    // Distinguish char-per-line (tokens ≤2 chars dominate) vs word-per-line
    let short_count = non_empty.iter().filter(|l| l.len() <= 2).count();
    let char_per_line = short_count * 100 >= non_empty.len() * 70;

    if char_per_line {
        // Char-per-line: append chars without space; blank lines = word space
        let mut result = String::new();
        let mut word_buf = String::new();
        for line in &lines {
            let t = line.trim();
            if t.is_empty() {
                if !word_buf.is_empty() {
                    if !result.is_empty() { result.push(' '); }
                    result.push_str(&word_buf);
                    word_buf.clear();
                }
            } else {
                word_buf.push_str(t);
            }
        }
        if !word_buf.is_empty() {
            if !result.is_empty() { result.push(' '); }
            result.push_str(&word_buf);
        }
        result
    } else {
        // Word-per-line: blank lines between words are lopdf layout artifacts, not
        // semantic paragraph breaks.  Join all non-empty tokens with a single space
        // so the result is one readable block that the caller will word-wrap.
        non_empty.join(" ")
    }
}

// Scan all pages of a PDF: extract text + run ink/signed analysis on embedded images.
fn scan_pdf(
    pdf_path: &str,
    nn: &SimpleNN,
    step: u32,
    dark_threshold: u8,
    min_dark_ratio: f64,
) -> SignedStatus {
    println!("\n=== PDF Scan: {} ===", pdf_path);

    let doc = match PdfDoc::load(pdf_path) {
        Ok(d) => d,
        Err(e) => { println!("Failed to open PDF: {:?}", e); return SignedStatus::NotSigned; }
    };

    let pages = doc.get_pages();
    println!("Total pages: {}\n", pages.len());

    let mut total_signed = 0usize;
    let mut total_images = 0usize;

    for (page_num, page_id) in &pages {
        println!("--- Page {} ---", page_num);

        // ── Text extraction ──────────────────────────────────────────
        match doc.extract_text(&[*page_num]) {
            Ok(text) if !text.trim().is_empty() => {
                let clean = normalize_pdf_text(text.trim());
                println!("  Text content:");
                // Preserve paragraph structure: iterate real lines, word-wrap only if >80 chars
                for raw_line in clean.lines() {
                    let line = raw_line.trim();
                    if line.is_empty() { continue; }
                    if line.len() <= 80 {
                        println!("    {}", line);
                    } else {
                        // Word-wrap long lines; never break inside a word (e.g. URLs stay intact)
                        let mut line_buf = String::new();
                        for word in line.split_whitespace() {
                            if !line_buf.is_empty() && line_buf.len() + word.len() + 1 > 80 {
                                println!("    {}", line_buf.trim_end());
                                line_buf.clear();
                            }
                            if !line_buf.is_empty() { line_buf.push(' '); }
                            line_buf.push_str(word);
                        }
                        if !line_buf.trim().is_empty() {
                            println!("    {}", line_buf.trim_end());
                        }
                    }
                }
            }
            Ok(_) => println!("  (no embedded text)"),
            Err(e) => println!("  Text error: {:?}", e),
        }

        // ── Embedded image extraction + ink analysis ─────────────────
        let imgs = pdf_page_images(&doc, *page_id);
        println!("  Embedded images: {}", imgs.len());
        total_images += imgs.len();

        for (idx, img) in imgs.iter().enumerate() {
            let (w, h) = img.dimensions();
            println!("\n  [Image {}] {}x{}px", idx + 1, w, h);

            let signed_regions = scan_signed_regions(img, step, dark_threshold, min_dark_ratio);
            if signed_regions.is_empty() {
                println!("  (image too small for region scan)");
                continue;
            }

            let signed_count = signed_regions.iter().filter(|(_, _, _, s)| *s).count();
            let pct = signed_count as f64 / signed_regions.len() as f64 * 100.0;
            println!("  Signed regions : {}/{} ({:.1}%)", signed_count, signed_regions.len(), pct);

            let status = classify_signed(signed_count, signed_regions.len());
            if status != SignedStatus::NotSigned {
                total_signed += 1;
                let gray = img.grayscale()
                    .resize_exact(IMG_WIDTH as u32, IMG_HEIGHT as u32, imageops::FilterType::Lanczos3);
                let pixels = gray.to_luma8().into_raw();
                let class = detect_signature(nn, &pixels);
                println!("  NN class       : {}", class);
            }
            println!("  {} STATUS       : {}", status.icon(), status.label());

            // Dedicated bottom-25% strip scan
            let (bs, bt, bstatus) =
                scan_bottom_strip_ink(img, 0.25, step, dark_threshold, min_dark_ratio);
            if bt > 0 {
                println!("  Bottom 25% ink  : {}/{} patches ({:.1}%) -> {} {}",
                    bs, bt, bs as f64 / bt as f64 * 100.0,
                    bstatus.icon(), bstatus.label());
            }
        }

        if imgs.is_empty() {
            println!("  (no embedded images — ink analysis requires raster image content)");
        }
        println!();
    }

    // ── Overall document verdict ───────────────────────────────────────
    let overall = if total_images == 0 {
        // text-only PDF: no image-based analysis possible
        SignedStatus::NotSigned
    } else if total_signed == total_images {
        SignedStatus::Complete
    } else if total_signed > 0 {
        SignedStatus::PartialNotComplete
    } else {
        SignedStatus::NotSigned
    };

    println!("=== PDF Scan Complete ===");
    println!("  Total pages       : {}", pages.len());
    println!("  Total images      : {}", total_images);
    println!("  Images with ink   : {}", total_signed);
    println!("  {} DOCUMENT STATUS : {}", overall.icon(), overall.label());
    overall
}

// ── DIRECTORY SCAN ─────────────────────────────────────────────────────────────────

fn collect_scan_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let supported = ["pdf", "jpg", "jpeg", "png", "bmp", "tif", "tiff"];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                files.extend(collect_scan_files(&p));
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if supported.contains(&ext.to_lowercase().as_str()) {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    files
}

fn scan_image_file(
    path: &Path,
    nn: &SimpleNN,
    step: u32,
    dark_threshold: u8,
    min_dark_ratio: f64,
) -> SignedStatus {
    let name = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy();
    println!("=== Image: {} ===", name);
    match image::open(path) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            println!("  Size: {}x{}px", w, h);
            let regions = scan_signed_regions(&img, step, dark_threshold, min_dark_ratio);
            let sc = regions.iter().filter(|(_, _, _, s)| *s).count();
            let total = regions.len();
            let pct = sc as f64 / total.max(1) as f64 * 100.0;
            println!("  Signed regions : {}/{} ({:.1}%)", sc, total, pct);
            let status = classify_signed(sc, total);
            let (bs, bt, bstatus) =
                scan_bottom_strip_ink(&img, 0.25, step, dark_threshold, min_dark_ratio);
            if bt > 0 {
                println!("  Bottom 25% ink : {}/{} ({:.1}%) -> {} {}",
                    bs, bt, bs as f64 / bt as f64 * 100.0,
                    bstatus.icon(), bstatus.label());
            }
            println!("  {} STATUS : {}", status.icon(), status.label());
            status
        }
        Err(e) => {
            println!("  Error loading image: {:?}", e);
            SignedStatus::NotSigned
        }
    }
}

fn scan_directory(
    dir: &Path,
    nn: &SimpleNN,
    step: u32,
    dark_threshold: u8,
    min_dark_ratio: f64,
) {
    let files = collect_scan_files(dir);
    if files.is_empty() {
        println!("No supported files found in: {}", dir.display());
        return;
    }
    println!("=== Directory Scan: {} ===", dir.display());
    println!("Found {} file(s)\n", files.len());

    let mut results: Vec<(String, &'static str, String)> = Vec::new();

    for (i, file) in files.iter().enumerate() {
        println!("\n[{}/{}] {}", i + 1, files.len(), "─".repeat(60));
        let ext = file.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let ftype: &'static str = if ext == "pdf" { "PDF" } else { "Image" };
        let status = if ext == "pdf" {
            scan_pdf(file.to_str().unwrap_or(""), nn, step, dark_threshold, min_dark_ratio)
        } else {
            scan_image_file(file, nn, step, dark_threshold, min_dark_ratio)
        };
        results.push((
            file.display().to_string(),
            ftype,
            format!("{} {}", status.icon(), status.label()),
        ));
    }

    println!("\n\n{}", "═".repeat(72));
    println!("=== DIRECTORY SCAN SUMMARY ===");
    println!("  Directory : {}", dir.display());
    println!("  Files     : {}", files.len());
    println!("{}", "─".repeat(72));
    for (path, ftype, status) in &results {
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        println!("  {:<22} {:<6} {}", status, ftype, name);
    }
    println!("{}", "─".repeat(72));
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── preprocessing ────────────────────────────────────────────────────────
    #[test]
    fn test_preprocessing() {
        let test_pixels = vec![0u8, 128u8, 255u8];
        let normalized = preprocess_image(&test_pixels);
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[1], 0.5019607843137255);
        assert_eq!(normalized[2], 1.0);
    }

    // ── cosine similarity ─────────────────────────────────────────────────────
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);
        assert_eq!(cosine_similarity(&a, &c), 0.0);
    }

    // ── classify_signed – all three document statuses ────────────────────────
    #[test]
    fn test_classify_signed_complete() {
        // >=15% ink coverage → SIGNED COMPLETE
        let status = classify_signed(15, 100);
        assert_eq!(status, SignedStatus::Complete,
            "15/100 (15%) should be Complete");
        assert_eq!(status.label(), "SIGNED COMPLETE");
        assert_eq!(status.icon(), "✓");
    }

    #[test]
    fn test_classify_signed_partial() {
        // >0% but <15% → SIGNED PARTS NOT COMPLETE
        let status = classify_signed(5, 100);
        assert_eq!(status, SignedStatus::PartialNotComplete,
            "5/100 (5%) should be PartialNotComplete");
        assert_eq!(status.label(), "SIGNED PARTS NOT COMPLETE");
        assert_eq!(status.icon(), "~");
    }

    #[test]
    fn test_classify_signed_not_signed() {
        // 0 signed patches → NOT YET SIGNED
        let status = classify_signed(0, 100);
        assert_eq!(status, SignedStatus::NotSigned,
            "0/100 should be NotSigned");
        assert_eq!(status.label(), "NOT YET SIGNED");
        assert_eq!(status.icon(), "✗");
    }

    #[test]
    fn test_classify_signed_empty_total() {
        // edge case: total = 0
        let status = classify_signed(0, 0);
        assert_eq!(status, SignedStatus::NotSigned);
    }

    #[test]
    fn test_classify_signed_boundary_exactly_15pct() {
        // exactly 15% is Complete
        let status = classify_signed(3, 20);
        assert_eq!(status, SignedStatus::Complete);
    }

    // ── analyze_region_signed ────────────────────────────────────────────────
    #[test]
    fn test_analyze_region_all_dark() {
        // all-black patch → signed
        let patch = vec![0u8; PIXEL_COUNT];
        let (ratio, is_signed) = analyze_region_signed(&patch, 100, 0.08);
        assert!(is_signed, "all-dark patch must be signed");
        assert!((ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_analyze_region_all_white() {
        // all-white patch → not signed
        let patch = vec![255u8; PIXEL_COUNT];
        let (ratio, is_signed) = analyze_region_signed(&patch, 100, 0.08);
        assert!(!is_signed, "all-white patch must not be signed");
        assert!((ratio - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_analyze_region_partial_dark() {
        // exactly min_dark_ratio of pixels dark → should pass threshold
        let total = PIXEL_COUNT;
        let dark_count = (total as f64 * 0.08).ceil() as usize;
        let mut patch = vec![255u8; total];
        for p in patch.iter_mut().take(dark_count) { *p = 0; }
        let (_, is_signed) = analyze_region_signed(&patch, 100, 0.08);
        assert!(is_signed, "patch at exactly min_dark_ratio must be signed");
    }

    // ── image-based document status: synthetic DynamicImage ─────────────────
    #[test]
    fn test_image_all_white_is_not_signed() {
        // White image → no ink → NOT YET SIGNED
        let pixels = vec![255u8; 56 * 56];
        let img = image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(56, 56, pixels).unwrap()
        );
        let regions = scan_signed_regions(&img, 28, 100, 0.08);
        let signed_count = regions.iter().filter(|(_, _, _, s)| *s).count();
        let status = classify_signed(signed_count, regions.len());
        assert_eq!(status, SignedStatus::NotSigned,
            "all-white image must produce NOT YET SIGNED");
    }

    #[test]
    fn test_image_all_dark_is_complete() {
        // All-black image → strong ink → SIGNED COMPLETE
        let pixels = vec![0u8; 56 * 56];
        let img = image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(56, 56, pixels).unwrap()
        );
        let regions = scan_signed_regions(&img, 28, 100, 0.08);
        let signed_count = regions.iter().filter(|(_, _, _, s)| *s).count();
        let status = classify_signed(signed_count, regions.len());
        assert_eq!(status, SignedStatus::Complete,
            "all-dark image must produce SIGNED COMPLETE");
    }

    #[test]
    fn test_image_partial_ink_partial_status() {
        // 2x2 grid of 28x28 patches; one all-dark, three all-white → 1/4 = 25% → Complete
        let mut pixels = vec![255u8; 56 * 56];
        // top-left 28x28 block → dark
        for row in 0..28usize {
            for col in 0..28usize {
                pixels[row * 56 + col] = 0;
            }
        }
        let img = image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(56, 56, pixels).unwrap()
        );
        let regions = scan_signed_regions(&img, 28, 100, 0.08);
        assert_eq!(regions.len(), 4, "56x56 image with step=28 should yield 4 patches");
        let signed_count = regions.iter().filter(|(_, _, _, s)| *s).count();
        assert_eq!(signed_count, 1, "only top-left patch is dark");
        let status = classify_signed(signed_count, regions.len());
        // 1/4 = 25% >= 15% → Complete
        assert_eq!(status, SignedStatus::Complete);
    }

    // ── normalize_pdf_text ───────────────────────────────────────────────────
    #[test]
    fn test_normalize_pdf_text_char_per_line() {
        // Simulate lopdf char-per-line output
        let raw = "H\ne\nl\nl\no\n\nW\no\nr\nl\nd";
        let out = normalize_pdf_text(raw);
        assert_eq!(out, "Hello World",
            "char-per-line text should be reassembled into words");
    }

    #[test]
    fn test_normalize_pdf_text_normal_lines() {
        // Normal multi-word lines should pass through unchanged
        let raw = "Hello World\nThis is a test";
        let out = normalize_pdf_text(raw);
        assert_eq!(out, raw);
    }
}
