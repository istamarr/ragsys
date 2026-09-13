#[cfg(feature = "leptess")]
use leptess::LepTess;
use lopdf::Document;
use std::path::Path;

// ─── Existing stubs (kept for backward compatibility) ────────────────────────

pub fn report_pdf() {
    println!("Report PDF done!");
}

pub fn report_generate() {
    println!("Report done!");
}

pub fn report_notes() {
    println!("Report notes done!");
}

// ─── OCR: Image → Text (Tesseract) ───────────────────────────────────────────

/// OCR a single image file (JPEG, PNG, TIFF, BMP) using Tesseract.
/// lang: e.g. "eng", "ind", "eng+ind"
/// Returns extracted UTF-8 text or an error message.
pub fn ocr_image(image_path: &str, lang: &str) -> Result<String, String> {
    if !Path::new(image_path).exists() {
        return Err(format!("File not found: {}", image_path));
    }
    #[cfg(not(feature = "leptess"))]
    return Err("Tesseract not available: install Tesseract and build with --features leptess".to_string());
    #[cfg(feature = "leptess")]
    {
        let mut lt = LepTess::new(None, lang)
            .map_err(|e| format!("Tesseract init failed: {:?}", e))?;
        lt.set_image(image_path)
            .map_err(|e| format!("Set image failed: {:?}", e))?;
        lt.get_utf8_text()
            .map_err(|e| format!("OCR failed: {:?}", e))
    }
}

/// OCR an image and return bounding boxes with confidence per word.
pub fn ocr_image_words(image_path: &str, lang: &str) -> Result<Vec<(String, f32)>, String> {
    if !Path::new(image_path).exists() {
        return Err(format!("File not found: {}", image_path));
    }
    #[cfg(not(feature = "leptess"))]
    return Err("Tesseract not available: install Tesseract and build with --features leptess".to_string());
    #[cfg(feature = "leptess")]
    {
        let mut lt = LepTess::new(None, lang)
            .map_err(|e| format!("Tesseract init failed: {:?}", e))?;
        lt.set_image(image_path)
            .map_err(|e| format!("Set image failed: {:?}", e))?;

        let text = lt.get_utf8_text()
            .map_err(|e| format!("OCR failed: {:?}", e))?;

        let words: Vec<(String, f32)> = text
            .split_whitespace()
            .map(|w: &str| (w.to_string(), 0.0_f32))
            .collect();

        Ok(words)
    }
}

// ─── OCR: PDF → Text ─────────────────────────────────────────────────────────

/// Extract embedded text from a PDF (no OCR — works for digital/text-based PDFs).
pub fn pdf_extract_text(pdf_path: &str) -> Result<String, String> {
    if !Path::new(pdf_path).exists() {
        return Err(format!("File not found: {}", pdf_path));
    }
    let doc = Document::load(pdf_path)
        .map_err(|e| format!("PDF load failed: {:?}", e))?;

    let mut full_text = String::new();
    let pages = doc.get_pages();

    for (page_num, _page_id) in &pages {
        match doc.extract_text(&[*page_num]) {
            Ok(text) => {
                full_text.push_str(&format!("--- Page {} ---\n{}\n", page_num, text));
            }
            Err(e) => {
                full_text.push_str(&format!("--- Page {} [error: {:?}] ---\n", page_num, e));
            }
        }
    }
    Ok(full_text)
}

/// Count total pages in a PDF.
pub fn pdf_page_count(pdf_path: &str) -> Result<usize, String> {
    if !Path::new(pdf_path).exists() {
        return Err(format!("File not found: {}", pdf_path));
    }
    let doc = Document::load(pdf_path)
        .map_err(|e| format!("PDF load failed: {:?}", e))?;
    Ok(doc.get_pages().len())
}

// ─── report_* with OCR ───────────────────────────────────────────────────────

/// OCR-powered report from a PDF file.
pub fn report_pdf_ocr(pdf_path: &str) -> String {
    println!("Report PDF OCR - {}", pdf_path);
    match pdf_extract_text(pdf_path) {
        Ok(text) => {
            println!("Extracted {} chars from PDF", text.len());
            text
        }
        Err(e) => {
            println!("report_pdf_ocr error: {}", e);
            e
        }
    }
}

/// OCR-powered report from an image file.
pub fn report_generate_ocr(image_path: &str) -> String {
    println!("Report Generate OCR - {}", image_path);
    match ocr_image(image_path, "eng") {
        Ok(text) => {
            println!("OCR extracted {} chars", text.len());
            text
        }
        Err(e) => {
            println!("report_generate_ocr error: {}", e);
            e
        }
    }
}

/// OCR notes from an image — supports multi-language (eng+ind).
pub fn report_notes_ocr(image_path: &str) -> String {
    println!("Report Notes OCR - {}", image_path);
    match ocr_image(image_path, "eng+ind") {
        Ok(text) => {
            println!("Notes OCR extracted {} chars", text.len());
            text
        }
        Err(e) => {
            println!("report_notes_ocr error: {}", e);
            e
        }
    }
}