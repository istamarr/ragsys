use std::env;
use std::fs;
use std::io::Write as IoWrite;
use std::path::PathBuf;
use agent_difusser::handler::documents::{report_pdf, report_generate, report_notes};
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

fn main() {
    // Load .env file for AGENT_DOCS configuration
    dotenv::dotenv().ok();

    println!("=== DIFSR Document Agent ===");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    let a = |i: usize, d: &'static str| -> String {
        args.get(i).cloned().unwrap_or_else(|| d.to_string())
    };

    match args[1].as_str() {
        "pdf" => {
            run_pdf(&a(2, ""), &a(3, "rag_output/agent_diffuser/report.pdf"));
        }
        "csv" => {
            run_csv(&a(2, ""), &a(3, "rag_output/agent_diffuser/report.csv"));
        }
        "docx" => {
            run_docx(&a(2, ""), &a(3, "rag_output/agent_diffuser/report.docx"));
        }
        "generate" => {
            run_generate(&a(2, "report"), &a(3, "rag_output/agent_diffuser/document.txt"));
        }
        "notes" => {
            run_notes(&a(2, ""), &a(3, "rag_output/agent_diffuser/notes.md"));
        }
        "md" => {
            run_md(&a(2, ""), &a(3, "rag_output/agent_diffuser/document.md"));
        }
        "template" => {
            let ttype = a(2, "izin-kerja");
            let ext = match args.get(3).map(String::as_str) {
                Some(p) if p.ends_with(".pdf")  => "pdf",
                Some(p) if p.ends_with(".docx") => "docx",
                _                               => "md",
            };
            let default_out = format!("rag_output/agent_diffuser/template_{}.{}", ttype, ext);
            let out = args.get(3).cloned().unwrap_or(default_out);
            let use_form_content = env::var("AGENT_DOCS")
                .map(|v| v == "form" || v == "#formcontent")
                .unwrap_or(false);
            run_template(&ttype, &out, use_form_content);
        }
        "md2pdf" => {
            run_md2pdf(&a(2, ""), &a(3, "rag_output/agent_diffuser/document.pdf"));
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!();
    println!("USAGE:");
    println!("  cargo run --bin agent_doc <COMMAND> [INPUT] [OUTPUT]");
    println!();
    println!("COMMANDS:");
    println!("  pdf        <input.txt> <output.pdf>   Real binary PDF (Helvetica, multi-line)");
    println!("  csv        <input.txt> <output.csv>   CSV with row/content/length/timestamp");
    println!("  docx       <input.txt> <output.docx>  Real OOXML DOCX (opens in Word/LibreOffice)");
    println!("  generate   report|summary|analysis <output.txt>");
    println!("  notes      <input.md>  <output.md>    Extract and format notes");
    println!("  md         <input.txt> <output.md>    Wrap text in Markdown structure");
    println!("  md2pdf     <input.md>  <output.pdf>   Convert Markdown to PDF");
    println!("  template   <type>      <output>       Generate document template");
    println!("             types: izin-kerja / work-permit");
    println!("             output ext .md .pdf .docx auto-detected");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin agent_doc pdf      input.txt  report.pdf");
    println!("  cargo run --bin agent_doc csv      input.txt  data.csv");
    println!("  cargo run --bin agent_doc docx     input.txt  document.docx");
    println!("  cargo run --bin agent_doc generate report summary.txt");
    println!("  cargo run --bin agent_doc notes    input.md   notes.md");
    println!("  cargo run --bin agent_doc md       input.txt  document.md");
    println!("  cargo run --bin agent_doc md2pdf   input.md   document.pdf");
    println!("  cargo run --bin agent_doc template izin-kerja izin-kerja.md");
    println!("  cargo run --bin agent_doc template izin-kerja izin-kerja.pdf");
    println!("  cargo run --bin agent_doc template izin-kerja izin-kerja.docx");
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn ensure_dir(path: &str) {
    if let Some(p) = PathBuf::from(path).parent() {
        fs::create_dir_all(p).ok();
    }
}

fn read_input(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    fs::read_to_string(path).unwrap_or_default()
}

// ── PDF ───────────────────────────────────────────────────────────────────────

fn run_pdf(input: &str, output: &str) {
    println!("Generating PDF...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);
    report_pdf();

    let text = read_input(input);
    let header = format!(
        "DIFSR Document Agent - PDF Report\nGenerated: {}\n{}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        "-".repeat(48),
    );
    let mut lines: Vec<&str> = header.lines().collect();
    lines.extend(text.lines());

    match write_real_pdf(&lines, output) {
        Ok(_)  => println!("PDF written : {}", output),
        Err(e) => eprintln!("PDF error  : {}", e),
    }
}

fn write_real_pdf(lines: &[&str], output: &str) -> std::io::Result<()> {
    let mut stream = String::from("BT\n/F1 11 Tf\n72 720 Td\n14 TL\n");
    for line in lines.iter().take(48) {
        let safe = line
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        stream.push_str(&format!("({}) Tj T*\n", safe));
    }
    stream.push_str("ET\n");
    let stream_len = stream.len();

    let mut pdf: Vec<u8> = Vec::new();

    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut off = [0usize; 5];

    off[0] = pdf.len();
    pdf.extend_from_slice(b"1 0 obj\n<</Type /Catalog /Pages 2 0 R>>\nendobj\n");

    off[1] = pdf.len();
    pdf.extend_from_slice(b"2 0 obj\n<</Type /Pages /Kids [3 0 R] /Count 1>>\nendobj\n");

    off[2] = pdf.len();
    pdf.extend_from_slice(
        b"3 0 obj\n<</Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\
\n  /Contents 4 0 R /Resources <</Font <</F1 5 0 R>>>>>>\nendobj\n",
    );

    off[3] = pdf.len();
    let hdr = format!("4 0 obj\n<</Length {}>>\nstream\n", stream_len);
    pdf.extend_from_slice(hdr.as_bytes());
    pdf.extend_from_slice(stream.as_bytes());
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    off[4] = pdf.len();
    pdf.extend_from_slice(
        b"5 0 obj\n<</Type /Font /Subtype /Type1 /BaseFont /Helvetica>>\nendobj\n",
    );

    let xref_pos = pdf.len();
    pdf.extend_from_slice(b"xref\n0 6\n");
    pdf.extend_from_slice(b"0000000000 65535 f\r\n");
    for o in &off {
        pdf.extend_from_slice(format!("{:010} 00000 n\r\n", o).as_bytes());
    }
    pdf.extend_from_slice(
        format!("trailer\n<</Size 6 /Root 1 0 R>>\nstartxref\n{}\n%%EOF\n", xref_pos).as_bytes(),
    );

    fs::write(output, pdf)
}

// ── CSV ───────────────────────────────────────────────────────────────────────

fn run_csv(input: &str, output: &str) {
    println!("Generating CSV...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);

    let text = read_input(input);
    match write_real_csv(&text, output) {
        Ok(_)  => println!("CSV written : {}", output),
        Err(e) => eprintln!("CSV error  : {}", e),
    }
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn write_real_csv(text: &str, output: &str) -> std::io::Result<()> {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut out = String::from("row,content,length,timestamp\n");
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push_str(&format!(
            "{},{},{},{}\n",
            i + 1,
            csv_escape(line),
            line.len(),
            csv_escape(&ts),
        ));
    }
    fs::write(output, out)
}

// ── DOCX ──────────────────────────────────────────────────────────────────────

fn run_docx(input: &str, output: &str) {
    println!("Generating DOCX...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);

    let text = read_input(input);
    match write_real_docx(&text, output) {
        Ok(_)  => println!("DOCX written: {}", output),
        Err(e) => eprintln!("DOCX error : {}", e),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn write_real_docx(text: &str, output: &str) -> anyhow::Result<()> {
    use zip::write::SimpleFileOptions;
    use zip::CompressionMethod;

    let file = fs::File::create(output)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml"  ContentType="application/xml"/>
  <Override PartName="/word/document.xml"
    ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#)?;

    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
    Target="word/document.xml"/>
</Relationships>"#)?;

    zip.start_file("word/_rels/document.xml.rels", opts)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#)?;

    let mut doc = String::from(concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
        r#"<w:body>"#,
    ));
    for line in text.lines() {
        doc.push_str(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
            xml_escape(line)
        ));
    }
    doc.push_str("</w:body></w:document>");

    zip.start_file("word/document.xml", opts)?;
    zip.write_all(doc.as_bytes())?;

    zip.finish()?;
    Ok(())
}

// ── generate / notes ──────────────────────────────────────────────────────────

fn run_generate(doc_type: &str, output: &str) {
    println!("Generating {} document...", doc_type);
    println!("  Output: {}", output);
    ensure_dir(output);
    report_generate();

    let content = match doc_type {
        "summary"  => generate_summary_content(),
        "analysis" => generate_analysis_content(),
        _          => generate_report_content(),
    };
    match fs::write(output, content) {
        Ok(_)  => println!("Document written: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn run_notes(input: &str, output: &str) {
    println!("Extracting notes...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);
    report_notes();

    let src = read_input(input);
    let content = format!(
        "# Notes\n\nGenerated: {}\n\n## Source\n\n{}\n\n## Extracted\n\n- Processing complete\n- Ready for review\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        if src.is_empty() { "(No input provided)".to_string() } else { src }
    );
    match fs::write(output, content) {
        Ok(_)  => println!("Notes written: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn generate_report_content() -> String {
    format!(
        "# Document Report\n\nGenerated: {}\n\n## Summary\n\nAuto-generated report from DIFSR Document Agent.\n\n\
         ## Contents\n\n1. Overview\n2. Analysis\n3. Conclusions\n\n## Overview\n\nProcessing complete.\n\n\
         ## Analysis\n\n- Status: Complete\n- Format: Text\n\n## Conclusions\n\nReport generation successful.\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}

fn generate_summary_content() -> String {
    format!(
        "# Document Summary\n\nGenerated: {}\n\n## Key Points\n\n- Document processed\n- Summary extracted\n- Ready for review\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}

fn generate_analysis_content() -> String {
    format!(
        "# Document Analysis\n\nGenerated: {}\n\n## Results\n\n| Metric | Value |\n|--------|-------|\n\
         | Status | Complete |\n| Type | Analysis |\n| Format | Markdown |\n\n## Detail\n\nAnalysis complete.\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
}

// ── MD ────────────────────────────────────────────────────────────────────────

fn run_md(input: &str, output: &str) {
    println!("Generating Markdown...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);
    let src = read_input(input);
    let content = format!(
        "# Document\n\nGenerated: {}\n\n---\n\n{}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        if src.is_empty() { "_No input provided_".to_string() } else { src }
    );
    match fs::write(output, content) {
        Ok(_)  => println!("Markdown written: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// ── TEMPLATE ──────────────────────────────────────────────────────────────────

fn run_template(ttype: &str, output: &str, use_form_content: bool) {
    println!("Generating template: {}", ttype);
    println!("  Output: {}", output);
    println!("  Content mode: {}", if use_form_content { "form (empty placeholders)" } else { "default (filled content)" });
    ensure_dir(output);
    let ext = output.rsplit('.').next().unwrap_or("md");
    match ttype {
        "izin-kerja" | "izin_kerja" | "work-permit" => match ext {
            "pdf" => match write_template_pdf(&izin_kerja_pdf_lines(use_form_content), output) {
                Ok(_)  => println!("PDF template  : {}", output),
                Err(e) => eprintln!("PDF error     : {}", e),
            },
            "docx" => match write_real_docx(&izin_kerja_md_content(use_form_content), output) {
                Ok(_)  => println!("DOCX template : {}", output),
                Err(e) => eprintln!("DOCX error    : {}", e),
            },
            _ => match fs::write(output, izin_kerja_md_content(use_form_content)) {
                Ok(_)  => println!("MD template   : {}", output),
                Err(e) => eprintln!("Error         : {}", e),
            },
        },
        "cert" | "certified" | "certificate" => match ext {
            "pdf" => match write_template_pdf(&certificate_pdf_lines(use_form_content), output) {
                Ok(_)  => println!("PDF template  : {}", output),
                Err(e) => eprintln!("PDF error     : {}", e),
            },
            "docx" => match write_real_docx(&certificate_md_content(use_form_content), output) {
                Ok(_)  => println!("DOCX template : {}", output),
                Err(e) => eprintln!("DOCX error    : {}", e),
            },
            _ => match fs::write(output, certificate_md_content(use_form_content)) {
                Ok(_)  => println!("MD template   : {}", output),
                Err(e) => eprintln!("Error         : {}", e),
            },
        },
        _ => {
            println!("Unknown template type: '{}'", ttype);
            println!("Available: izin-kerja, work-permit");
        }
    }
}

fn izin_kerja_md_content(use_form_content: bool) -> String {
    let ts = chrono::Local::now().format("%Y-%m-%d").to_string();
    if use_form_content {
        format!(
"# IZIN KERJA / CHANGE REQUEST

| Field | Value |
|-------|-------|
| **Nomor CR** | _______________ |
| **Tanggal** | {ts} |
| **Prioritas** | `[ ] Normal` `[ ] High` `[ ] Critical` |
| **Kategori** | Infrastructure / Security / Application |
| **Divisi** | IT Security / Operations Service |

---

## I. TUJUAN

_Deskripsikan tujuan pekerjaan / change request ini:_

> [Isi tujuan di sini]

---

## II. RUANG LINGKUP

_Sebutkan sistem, aplikasi, atau komponen yang termasuk dalam ruang lingkup:_

> [Isi ruang lingkup di sini]

---

## III. INVESTIGASI

### a. Resource

| Peran | Nama | Jabatan |
|-------|------|---------|
| Pelaksana | _______________ | Officer Security Operations Service |
| PIC | _______________ | Kepala Departemen Security Operations Service |

### b. Jadwal Pelaksanaan

| Item | Detail |
|------|--------|
| Tanggal Pelaksanaan | _______________ |
| Waktu Mulai | _______________ |
| Waktu Selesai | _______________ |
| Kegiatan | _______________ |
| Lokasi | _______________ |

### c. Impact

_Seluruh aplikasi, server maupun service yang berada di area yang terdampak:_

> [Isi dampak di sini]

**Referensi Detail Impact:**  
<https://docs.google.com/spreadsheets/d/_______________>

---

## IV. RENCANA PEKERJAAN

| No | Kegiatan | Waktu | PIC |
|----|----------|-------|-----|
| 1 | _______________ | ___ | ___ |
| 2 | _______________ | ___ | ___ |
| 3 | _______________ | ___ | ___ |
| 4 | _______________ | ___ | ___ |

---

## V. PERSETUJUAN

_Bagian ini untuk persetujuan formal dari rekomendasi yang telah diberikan
termasuk perubahan yang dilakukan, jadwal dan dampak dari permintaan perubahan._

| | **Dibuat Oleh** | **Diperiksa Oleh** |
|---|---|---|
| **Jabatan** | Officer Security Operations Service | Kepala Departemen Security Operations Service |
| **Nama** | _______________ | _______________ |
| **NIK** | _______________ | _______________ |
| **Tanggal** | _______________ | _______________ |
| **Tanda Tangan** | | |

### Menyetujui dan Mengetahui

| | |
|---|---|
| **Jabatan** | Kepala Divisi IT Security |
| **Nama** | _______________ |
| **NIK** | _______________ |
| **Tanggal** | _______________ |
| **Tanda Tangan** | |

---
_Generated by DIFSR Document Agent — {ts}_
",
            ts = ts,
        )
    } else {
        format!(
"# IZIN KERJA / CHANGE REQUEST

| Field | Value |
|-------|-------|
| **Nomor CR** | CR-2026-{ts} |
| **Tanggal** | {ts} |
| **Prioritas** | `[X] Normal` `[ ] High` `[ ] Critical` |
| **Kategori** | Infrastructure / Security / Application |
| **Divisi** | IT Security / Operations Service |

---

## I. TUJUAN

_Deskripsikan tujuan pekerjaan / change request ini:_

> Melakukan upgrade dan konfigurasi ulang sistem firewall Checkpoint di ServerFarm DC Jakarta untuk meningkatkan keamanan jaringan dan memperbarui aturan akses sesuai kebutuhan operasional terbaru.

---

## II. RUANG LINGKUP

_Sebutkan sistem, aplikasi, atau komponen yang termasuk dalam ruang lingkup:_

> - Firewall Checkpoint di ServerFarm DC Jakarta
> - Sistem aplikasi yang terhubung ke firewall
> - Service yang berjalan di area serverfarm dan multisite
> - Konfigurasi rule akses dan policy security

---

## III. INVESTIGASI

### a. Resource

| Peran | Nama | Jabatan |
|-------|------|---------|
| Pelaksana | Budi Santoso | Officer Security Operations Service |
| PIC | Ahmad Rizki | Kepala Departemen Security Operations Service |

### b. Jadwal Pelaksanaan

| Item | Detail |
|------|--------|
| Tanggal Pelaksanaan | {ts} |
| Waktu Mulai | 22:00 WIB |
| Waktu Selesai | 02:00 WIB |
| Kegiatan | Upgrade Firewall ServerFarm DC Jakarta |
| Lokasi | ServerFarm DC Jakarta |

### c. Impact

_Seluruh aplikasi, server maupun service yang berada di area yang terdampak:_

> Impact Seluruh aplikasi, server maupun service yang berada di area serverfarm dan multisite. Selama proses upgrade, akan terjadi downtime sementara untuk service yang terhubung langsung ke firewall. Service lain yang tidak terdampak akan tetap berjalan normal.

**Referensi Detail Impact:**  
<https://docs.google.com/spreadsheets/d/1ZUSa8dLIC2B9HwIjBh2_xutx-m4g5rzPg9mR2IN2oQc/edit?gid=1616899155#gid=1616899155>

---

## IV. RENCANA PEKERJAAN

| No | Kegiatan | Waktu | PIC |
|----|----------|-------|-----|
| 1 | Backup konfigurasi firewall saat ini | 22:00-22:30 | Budi Santoso |
| 2 | Download dan install update firmware terbaru | 22:30-23:30 | Budi Santoso |
| 3 | Konfigurasi ulang rule akses dan policy | 23:30-01:00 | Ahmad Rizki |
| 4 | Testing konektivitas dan service terkait | 01:00-02:00 | Budi Santoso |

---

## V. PERSETUJUAN

_Bagian ini untuk persetujuan formal dari rekomendasi yang telah diberikan
termasuk perubahan yang dilakukan, jadwal dan dampak dari permintaan perubahan._

| | **Dibuat Oleh** | **Diperiksa Oleh** |
|---|---|---|
| **Jabatan** | Officer Security Operations Service | Kepala Departemen Security Operations Service |
| **Nama** | Budi Santoso | Ahmad Rizki |
| **NIK** | 12345678 | 87654321 |
| **Tanggal** | {ts} | {ts} |
| **Tanda Tangan** | | |

### Menyetujui dan Mengetahui

| | |
|---|---|
| **Jabatan** | Kepala Divisi IT Security |
| **Nama** | Diana Putri |
| **NIK** | 11223344 |
| **Tanggal** | {ts} |
| **Tanda Tangan** | |

---
_Generated by DIFSR Document Agent — {ts}_
",
            ts = ts,
        )
    }
}

fn certificate_md_content(use_form_content: bool) -> String {
    let ts = chrono::Local::now().format("%Y-%m-%d").to_string();
    if use_form_content {
        format!(
"# CERTIFICATE

| Field | Value |
|-------|-------|
| **Nomor CR** | _______________ |
| **Tanggal** | {ts} |
| **Token** | _______________ |
| **Mint** | _______________ |
| **Valuable Fee** | _______________ |

---

## Ipfs Hash

> [Isi IPFS Hash di sini]

---

## Item Brand

> [Isi Item Brand di sini]

---

## Item Category

> [Isi Item Category di sini]

---

## Cert Owner

> [Isi Cert Owner di sini]

---

## Currencies

> [Isi Currencies di sini]

---

## System Reg

| Field | Value |
|-------|-------|
| **City** | _______________ |
| **Country** | _______________ |

---

## Latest Trx

| Field | Value |
|-------|-------|
| **Network** | _______________ |
| **Transaction Hash** | _______________ |

---

## Valid Authenticator

> [Isi Valid Authenticator di sini]

---

## Summary

> [scan result, crowd fund, owner, mint trx success, bi checking, source origin]

---
_Generated by DIFSR Document Agent Asist — {ts}_
",
            ts = ts,
        )
    } else {
        format!(
"# CERTIFICATE

| Field | Value |
|-------|-------|
| **Nomor CR** | CR-2026-{ts} |
| **Tanggal** | {ts} |
| **Token** | TKN-0001 |
| **Mint** | 2026-01-01 |
| **Valuable Fee** | 0.005 ETH |

---

## Ipfs Hash

> QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG

---

## Item Brand

> DIFSR Brand

---

## Item Category

> Digital Asset / Certificate

---

## Cert Owner

> 0xAbCdEf1234567890AbCdEf1234567890AbCdEf12

---

## Currencies

> ETH / IDR

---

## System Reg

| Field | Value |
|-------|-------|
| **City** | Jakarta |
| **Country** | Indonesia |

---

## Latest Trx

| Field | Value |
|-------|-------|
| **Network** | Ethereum Mainnet |
| **Transaction Hash** | 0x1234...abcd |

---

## Valid Authenticator

> DIFSR Certification Authority — {ts}

---

## Summary

> Scan result: Valid | Crowd fund: Active | Owner: Verified | Mint trx: Success | BI Checking: Clear | Source origin: DIFSR

---
_Generated by DIFSR Document Agent Asist — {ts}_
",
            ts = ts,
        )
    }
}

fn izin_kerja_pdf_lines(use_form_content: bool) -> Vec<String> {
    let ts = chrono::Local::now().format("%Y-%m-%d").to_string();
    if use_form_content {
        vec![
            "# IZIN KERJA / CHANGE REQUEST".into(),
            "=".repeat(48),
            String::new(),
            format!("Nomor CR   : _______________"),
            format!("Tanggal    : {}", ts),
            "Prioritas  : [ ] Normal  [ ] High  [ ] Critical".into(),
            "Divisi     : IT Security / Operations Service".into(),
            String::new(),
            "## I. TUJUAN".into(),
            "-".repeat(40),
            "[Deskripsikan tujuan pekerjaan / change request ini]".into(),
            String::new(),
            "## II. RUANG LINGKUP".into(),
            "-".repeat(40),
            "[Sebutkan sistem, aplikasi, atau komponen dalam ruang lingkup]".into(),
            String::new(),
            "## III. INVESTIGASI".into(),
            "-".repeat(40),
            String::new(),
            "a. Resource".into(),
            "   Tim/Personil : _______________".into(),
            "   Jabatan      : _______________".into(),
            String::new(),
            "b. Jadwal Pelaksanaan".into(),
            "   Tanggal      : _______________".into(),
            "   Waktu        : ___ s/d ___".into(),
            "   Kegiatan     : _______________".into(),
            "   Lokasi       : _______________".into(),
            String::new(),
            "c. Impact".into(),
            "   Aplikasi, server & service yang terdampak:".into(),
            "   [Isi dampak di sini]".into(),
            String::new(),
            "   Referensi Detail:".into(),
            "   https://docs.google.com/spreadsheets/d/_______________".into(),
            String::new(),
            "## IV. RENCANA PEKERJAAN".into(),
            "-".repeat(40),
            "No.  Kegiatan                          Waktu     PIC".into(),
            "-".repeat(52),
            "1.   ____________________________      ______    ___".into(),
            "2.   ____________________________      ______    ___".into(),
            "3.   ____________________________      ______    ___".into(),
            "4.   ____________________________      ______    ___".into(),
            String::new(),
            "## V. PERSETUJUAN".into(),
            "-".repeat(40),
            "Bagian ini untuk persetujuan formal dari rekomendasi yang telah".into(),
            "diberikan termasuk perubahan yang dilakukan, jadwal dan dampak".into(),
            "dari permintaan perubahan.".into(),
            String::new(),
            "Dibuat Oleh                         Diperiksa Oleh".into(),
            "Officer Security Operations Svc     Kepala Dept Security Ops Svc".into(),
            "Nama   : _______________            Nama   : _______________".into(),
            "NIK    : _______________            NIK    : _______________".into(),
            format!("Tanggal: {}               Tanggal: _______________", ts),
            String::new(),
            "       Menyetujui dan Mengetahui".into(),
            "       Kepala Divisi IT Security".into(),
            "       Nama   : _______________".into(),
            "       NIK    : _______________".into(),
            "       Tanggal: _______________".into(),
            String::new(),
            "-".repeat(48),
            format!("Generated by DIFSR Document Agent — {}", ts),
        ]
    } else {
        vec![
            "# IZIN KERJA / CHANGE REQUEST".into(),
            "=".repeat(48),
            String::new(),
            format!("Nomor CR   : CR-2026-{}", ts),
            format!("Tanggal    : {}", ts),
            "Prioritas  : [X] Normal  [ ] High  [ ] Critical".into(),
            "Divisi     : IT Security / Operations Service".into(),
            String::new(),
            "## I. TUJUAN".into(),
            "-".repeat(40),
            "Melakukan upgrade dan konfigurasi ulang sistem firewall".into(),
            "Checkpoint di ServerFarm DC Jakarta untuk meningkatkan".into(),
            "keamanan jaringan dan memperbarui aturan akses sesuai".into(),
            "kebutuhan operasional terbaru.".into(),
            String::new(),
            "## II. RUANG LINGKUP".into(),
            "-".repeat(40),
            "- Firewall Checkpoint di ServerFarm DC Jakarta".into(),
            "- Sistem aplikasi yang terhubung ke firewall".into(),
            "- Service yang berjalan di area serverfarm dan multisite".into(),
            "- Konfigurasi rule akses dan policy security".into(),
            String::new(),
            "## III. INVESTIGASI".into(),
            "-".repeat(40),
            String::new(),
            "a. Resource".into(),
            "   Pelaksana   : Budi Santoso".into(),
            "   Jabatan     : Officer Security Operations Service".into(),
            "   PIC         : Ahmad Rizki".into(),
            "   Jabatan     : Kepala Departemen Security Ops Service".into(),
            String::new(),
            "b. Jadwal Pelaksanaan".into(),
            "   Tanggal     : {}".into(),
            "   Waktu Mulai : 22:00 WIB".into(),
            "   Waktu Selesai: 02:00 WIB".into(),
            "   Kegiatan    : Upgrade Firewall ServerFarm DC Jakarta".into(),
            "   Lokasi      : ServerFarm DC Jakarta".into(),
            String::new(),
            "c. Impact".into(),
            "   Impact Seluruh aplikasi, server maupun service yang".into(),
            "   berada di area serverfarm dan multisite. Selama proses".into(),
            "   upgrade, akan terjadi downtime sementara untuk service".into(),
            "   yang terhubung langsung ke firewall. Service lain yang".into(),
            "   tidak terdampak akan tetap berjalan normal.".into(),
            String::new(),
            "   Referensi Detail:".into(),
            "   https://docs.google.com/spreadsheets/d/1ZUSa8dLIC2B9HwIjBh2_".into(),
            "   xutx-m4g5rzPg9mR2IN2oQc/edit?gid=1616899155#gid=1616899155".into(),
            String::new(),
            "## IV. RENCANA PEKERJAAN".into(),
            "-".repeat(40),
            "No.  Kegiatan                          Waktu     PIC".into(),
            "-".repeat(52),
            "1.   Backup konfigurasi firewall saat ini 22:00-22:30 Budi Santoso".into(),
            "2.   Download dan install update firmware 22:30-23:30 Budi Santoso".into(),
            "3.   Konfigurasi ulang rule akses dan policy 23:30-01:00 Ahmad Rizki".into(),
            "4.   Testing konektivitas dan service terkait 01:00-02:00 Budi Santoso".into(),
            String::new(),
            "## V. PERSETUJUAN".into(),
            "-".repeat(40),
            "Bagian ini untuk persetujuan formal dari rekomendasi yang telah".into(),
            "diberikan termasuk perubahan yang dilakukan, jadwal dan dampak".into(),
            "dari permintaan perubahan.".into(),
            String::new(),
            "Dibuat Oleh                         Diperiksa Oleh".into(),
            "Officer Security Operations Svc     Kepala Dept Security Ops Svc".into(),
            "Nama   : Budi Santoso               Nama   : Ahmad Rizki".into(),
            "NIK    : 12345678                   NIK    : 87654321".into(),
            format!("Tanggal: {}                       Tanggal: {}", ts, ts),
            String::new(),
            "       Menyetujui dan Mengetahui".into(),
            "       Kepala Divisi IT Security".into(),
            "       Nama   : Diana Putri".into(),
            "       NIK    : 11223344".into(),
            format!("       Tanggal: {}", ts),
            String::new(),
            "-".repeat(48),
            format!("Generated by DIFSR Document Agent — {}", ts),
        ]
    }
}

fn certificate_pdf_lines(use_form_content: bool) -> Vec<String> {
    let ts = chrono::Local::now().format("%Y-%m-%d").to_string();
    if use_form_content {
        vec![
            "# CERTIFICATE".into(),
            "=".repeat(48),
            String::new(),
            format!("Nomor CR   : _______________"),
            format!("Tanggal    : {}", ts),
            "Token       : ".into(),
            "Mint        : ".into(),
            "Valuable Fee: ".into(),//Royalty
            String::new(),
            "Ipfs Hash".into(),
            "-".repeat(40),
            "Item Brand".into(),
            "".into(),
            "Item Category".into(),
            "".into(),
            "Cert Owner".into(),
            "".into(),
            String::new(),
            "Currencies".into(),
            "-".repeat(40),
            "".into(),
            String::new(),
            "System Reg".into(),
            "-".repeat(40),
            String::new(),
            "City,Country".into(),
            "".into(),
            "".into(),
            String::new(),
            "Latest Trx".into(),
            "".into(),
            "Network".into(),
            "".into(),
            "".into(),
            "Valid Authenticator".into(),//whos create and valid
            "".into(),
            String::new(),
            "".into(),//space for qr
            String::new(),
            format!("Generated by DIFSR Document Agent Asist — {}", ts),
            String::new(),
            "Summary".into(),//summary: scan result, crowd fund, owner, mint trx success, bi checking, source origin
            "".into(),
        ]
    } else {
        vec![
            "# CERTIFICATE".into(),
            "=".repeat(48),
            String::new(),
            format!("Nomor CR   : CR-2026-{}", ts),
            format!("Tanggal    : {}", ts),
            "Token       : TKN-0001".into(),
            "Mint        : 2026-01-01".into(),
            "Valuable Fee: 0.005 ETH".into(),//Royalty
            String::new(),
            "Ipfs Hash".into(),
            "-".repeat(40),
            "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".into(),
            String::new(),
            "Item Brand".into(),
            "-".repeat(40),
            "DIFSR Brand".into(),
            String::new(),
            "Item Category".into(),
            "-".repeat(40),
            "Digital Asset / Certificate".into(),
            String::new(),
            "Cert Owner".into(),
            "-".repeat(40),
            "0xAbCdEf1234567890AbCdEf1234567890AbCdEf12".into(),
            String::new(),
            "Currencies".into(),
            "-".repeat(40),
            "ETH / IDR".into(),
            String::new(),
            "System Reg".into(),
            "-".repeat(40),
            "City    : Jakarta".into(),
            "Country : Indonesia".into(),
            String::new(),
            "Latest Trx".into(),
            "-".repeat(40),
            "Network : Ethereum Mainnet".into(),
            "Trx Hash: 0x1234...abcd".into(),
            String::new(),
            "Valid Authenticator".into(),//whos create and valid
            "-".repeat(40),
            format!("DIFSR Certification Authority — {}", ts),
            String::new(),
            "".into(),//space for qr
            String::new(),
            "Summary".into(),//summary: scan result, crowd fund, owner, mint trx success, bi checking, source origin
            "-".repeat(40),
            "Scan: Valid | Fund: Active | Owner: Verified | Mint: Success".into(),
            "BI Checking: Clear | Source origin: DIFSR".into(),
            String::new(),
            "-".repeat(48),
            format!("Generated by DIFSR Document Agent Asist — {}", ts),
        ]
    }
}

// Multi-page PDF writer used by run_template.
// lines: content split into Vec<String>; headings prefixed with "# " or "## ".
fn write_template_pdf(lines: &[String], output: &str) -> std::io::Result<()> {
    const LPP: usize = 50;
    let pages: Vec<&[String]> = lines.chunks(LPP).collect();
    let np = pages.len().max(1);

    // Object layout (1-indexed):
    //  1          Catalog
    //  2          Pages
    //  3..3+np-1  Page descriptors   (np objects)
    //  3+np..     Content streams    (np objects)
    //  3+2*np     Font F1 Helvetica
    //  3+2*np+1   Font F2 Helvetica-Bold
    let font1 = 3 + 2 * np;
    let font2 = font1 + 1;
    let total = font2;
    let mut offsets = vec![0usize; total + 1];
    let mut pdf: Vec<u8> = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    offsets[1] = pdf.len();
    pdf.extend_from_slice(b"1 0 obj\n<</Type /Catalog /Pages 2 0 R>>\nendobj\n");

    let kids: String = (3..3 + np).map(|i| format!("{} 0 R", i)).collect::<Vec<_>>().join(" ");
    offsets[2] = pdf.len();
    pdf.extend_from_slice(
        format!("2 0 obj\n<</Type /Pages /Kids [{}] /Count {}>>\nendobj\n", kids, np).as_bytes(),
    );

    for i in 0..np {
        let pid = 3 + i;
        let cid = 3 + np + i;
        offsets[pid] = pdf.len();
        pdf.extend_from_slice(format!(
            "{} 0 obj\n<</Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n\
  /Contents {} 0 R /Resources <</Font <</F1 {} 0 R /F2 {} 0 R>>>>>>\nendobj\n",
            pid, cid, font1, font2
        ).as_bytes());
    }

    for (i, page_lines) in pages.iter().enumerate() {
        let cid = 3 + np + i;
        let mut s = String::from("BT\n/F1 10 Tf\n50 742 Td\n12 TL\n");
        for line in *page_lines {
            let (is_h1, is_h2, text) =
                if let Some(t) = line.strip_prefix("# ")  { (true,  false, t) }
                else if let Some(t) = line.strip_prefix("## ") { (false, true,  t) }
                else                                           { (false, false, line.as_str()) };
            let safe = text
                .replace('\\', "\\\\")
                .replace('(', "\\(")
                .replace(')', "\\)");
            if is_h1 {
                s.push_str(&format!("/F2 14 Tf ({}) Tj T*\n/F1 10 Tf\n", safe));
            } else if is_h2 {
                s.push_str(&format!("/F2 12 Tf ({}) Tj T*\n/F1 10 Tf\n", safe));
            } else {
                s.push_str(&format!("({}) Tj T*\n", safe));
            }
        }
        s.push_str("ET\n");
        let slen = s.len();
        offsets[cid] = pdf.len();
        pdf.extend_from_slice(
            format!("{} 0 obj\n<</Length {}>>\nstream\n", cid, slen).as_bytes(),
        );
        pdf.extend_from_slice(s.as_bytes());
        pdf.extend_from_slice(b"\nendstream\nendobj\n");
    }

    offsets[font1] = pdf.len();
    pdf.extend_from_slice(
        format!("{} 0 obj\n<</Type /Font /Subtype /Type1 /BaseFont /Helvetica>>\nendobj\n", font1).as_bytes(),
    );
    offsets[font2] = pdf.len();
    pdf.extend_from_slice(
        format!("{} 0 obj\n<</Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold>>\nendobj\n", font2).as_bytes(),
    );

    let xref_pos = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", total + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f\r\n");
    for i in 1..=total {
        pdf.extend_from_slice(format!("{:010} 00000 n\r\n", offsets[i]).as_bytes());
    }
    pdf.extend_from_slice(
        format!("trailer\n<</Size {} /Root 1 0 R>>\nstartxref\n{}\n%%EOF\n", total + 1, xref_pos).as_bytes(),
    );
    fs::write(output, &pdf)
}

// ── MD2PDF ──────────────────────────────────────────────────────────────────────

fn run_md2pdf(input: &str, output: &str) {
    println!("Converting Markdown to PDF...");
    println!("  Input : {}", if input.is_empty() { "(none)" } else { input });
    println!("  Output: {}", output);
    ensure_dir(output);
    let src = read_input(input);
    if src.is_empty() {
        eprintln!("Error: input file is empty or not readable");
        return;
    }
    let lines = markdown_to_pdf_lines(&src);
    match write_template_pdf(&lines, output) {
        Ok(_)  => println!("PDF written: {}", output),
        Err(e) => eprintln!("PDF error : {}", e),
    }
}

fn markdown_to_pdf_lines(md: &str) -> Vec<String> {
    let parser = Parser::new(md);
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut in_code_block = false;
    let mut in_list = false;
    let mut list_depth = 0;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push(String::new()); // blank line between paragraphs
            }
            Event::Start(Tag::Heading { level, .. }) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                let hashes = "#".repeat(level as usize);
                current_line.push_str(&hashes);
                current_line.push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push(String::new());
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push("--- Code Block ---".into());
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                lines.push(String::new());
            }
            Event::Start(Tag::List(_)) => {
                in_list = true;
            }
            Event::End(TagEnd::List(_)) => {
                in_list = false;
                list_depth = 0;
                lines.push(String::new());
            }
            Event::Start(Tag::Item) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                current_line.push_str(&"  ".repeat(list_depth));
                current_line.push_str("- ");
            }
            Event::End(TagEnd::Item) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push(">".into());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push(String::new());
            }
            Event::Text(text) => {
                let t = text.to_string();
                if in_code_block {
                    lines.push(format!("  {}", t));
                } else {
                    current_line.push_str(&t);
                }
            }
            Event::Code(text) => {
                current_line.push('`');
                current_line.push_str(&text.to_string());
                current_line.push('`');
            }
            Event::SoftBreak | Event::HardBreak => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            Event::Rule => {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                lines.push("---".into());
                lines.push(String::new());
            }
            _ => {}
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
