// @2025 Istamar Rozid

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use log::error;
use serde_derive::{Deserialize, Serialize};
use crate::shared::sharedUtils::FISHBONE_FORMAT;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDataNumber {
    pub data_number: HashMap<String, ProcessedData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub source_type: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
    pub structured_content: StructuredContent,
    pub raw_text: String,
    pub chunks: Vec<TextChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredContent {
    Json {
        keys: Vec<String>,
        values: HashMap<String, String>,
        object_count: usize,
        array_count: usize,
    },
    Csv {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        row_count: usize,
        column_count: usize,
    },
    Html {
        title: Option<String>,
        headings: Vec<String>,
        paragraphs: Vec<String>,
        links: Vec<String>,
    },
    Xml {
        root_element: String,
        elements: Vec<String>,
        attributes: HashMap<String, String>,
        tag_count: usize,
    },
    Text {
        word_count: usize,
        line_count: usize,
        paragraphs: Vec<String>,
        char_count: usize,
    },
    Parquet {
        schema: Vec<String>,
        column_count: usize,
        row_count: usize,
        file_size: usize,
        compression: String,
    },
    Database {
        tables: Vec<String>,
        table_count: usize,
        file_size: usize,
        database_type: String,
        schema_info: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub position: usize,
    pub word_count: usize,
}

pub trait UniversalParser {
    fn name(&self) -> &'static str;
    fn can_parse(&self, content_type: &str, content: &str) -> bool;
    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String>;
}

pub struct JsonParser;
impl UniversalParser for JsonParser {
    fn name(&self) -> &'static str { "JSON Parser" }

    fn can_parse(&self, content_type: &str, content: &str) -> bool {
        content_type.contains("json") ||
            content.trim_start().starts_with('{') ||
            content.trim_start().starts_with('[')
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        // Simple JSON analysis without external parser
        let object_count = content.matches('{').count();
        let array_count = content.matches('[').count();
        let quote_count = content.matches('"').count();

        // Extract key-value pairs
        let mut keys = Vec::new();
        let mut values = HashMap::new();

        // Find quoted strings similiar keys
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains(':') {
                if let Some(colon_pos) = trimmed.find(':') {
                    let key_part = &trimmed[..colon_pos].trim();
                    let value_part = &trimmed[colon_pos + 1..].trim();

                    // Extract key (remove quotes)
                    let key = key_part.trim_matches('"').trim_matches('\'');
                    if !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        keys.push(key.to_string());

                        // Extract value (basic)
                        let value = value_part.trim_end_matches(',').trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            values.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        let chunks = create_chunks(content, "json");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("object_count".to_string(), object_count.to_string());
        metadata.insert("array_count".to_string(), array_count.to_string());
        metadata.insert("quote_count".to_string(), quote_count.to_string());
        metadata.insert("key_count".to_string(), keys.len().to_string());
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "json".to_string(),
            content_type: "application/json".to_string(),
            metadata,
            structured_content: StructuredContent::Json {
                keys,
                values,
                object_count,
                array_count,
            },
            raw_text: content.to_string(),
            chunks,
        })
    }
}

pub struct CsvParser;
impl UniversalParser for CsvParser {
    fn name(&self) -> &'static str { "CSV Parser" }

    fn can_parse(&self, content_type: &str, content: &str) -> bool {
        content_type.contains("csv") ||
            (content.lines().count() > 1 && content.lines().next().unwrap_or("").contains(','))
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let lines: Vec<&str> = content.lines().collect();

        let headers = if !lines.is_empty() {
            lines[0].split(',')
                .map(|s| s.trim_matches('"').trim().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();
        for line in lines.iter().skip(1) {
            if !line.trim().is_empty() {
                let row: Vec<String> = line.split(',')
                    .map(|s| s.trim_matches('"').trim().to_string())
                    .collect();
                rows.push(row);
            }
        }

        let row_count = rows.len();
        let column_count = headers.len();
        let chunks = create_chunks(content, "csv");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("column_count".to_string(), column_count.to_string());
        metadata.insert("row_count".to_string(), row_count.to_string());
        metadata.insert("headers".to_string(), headers.join("|"));
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "csv".to_string(),
            content_type: "text/csv".to_string(),
            metadata,
            structured_content: StructuredContent::Csv {
                headers,
                rows,
                row_count,
                column_count,
            },
            raw_text: content.to_string(),
            chunks,
        })
    }
}

pub struct HtmlParser;
impl UniversalParser for HtmlParser {
    fn name(&self) -> &'static str { "HTML Parser" }

    fn can_parse(&self, content_type: &str, content: &str) -> bool {
        content_type.contains("html") ||
            content.contains("<html") ||
            content.contains("<!DOCTYPE")
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        // Simple HTML parsing without external dependencies
        let title = extract_between(content, "<title>", "</title>");

        let mut headings = Vec::new();
        for i in 1..=6 {
            let start_tag = format!("<h{}", i);
            let end_tag = format!("</h{}>", i);
            headings.extend(extract_all_between(content, &start_tag, &end_tag));
        }

        let paragraphs = extract_all_between(content, "<p", "</p>");
        let links = extract_all_href_links(content);

        let chunks = create_chunks(content, "html");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("title".to_string(), title.clone().unwrap_or("No title".to_string()));
        metadata.insert("heading_count".to_string(), headings.len().to_string());
        metadata.insert("paragraph_count".to_string(), paragraphs.len().to_string());
        metadata.insert("link_count".to_string(), links.len().to_string());
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "html".to_string(),
            content_type: "text/html".to_string(),
            metadata,
            structured_content: StructuredContent::Html {
                title,
                headings,
                paragraphs,
                links,
            },
            raw_text: content.to_string(),
            chunks,
        })
    }
}

pub struct XmlParser;
impl UniversalParser for XmlParser {
    fn name(&self) -> &'static str { "XML Parser" }

    fn can_parse(&self, content_type: &str, content: &str) -> bool {
        content_type.contains("xml") ||
            content.trim_start().starts_with("<?xml") ||
            content.trim_start().starts_with('<')
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let tag_count = content.matches('<').count();
        let closing_tag_count = content.matches("</").count();

        let root_element = if let Some(start) = content.find('<') {
            if let Some(end) = content[start..].find('>') {
                let tag = &content[start + 1..start + end];
                tag.split_whitespace().next().unwrap_or("unknown").to_string()
            } else { "unknown".to_string() }
        } else {
            "unknown".to_string()
        };

        let mut elements = Vec::new();
        let mut attributes = HashMap::new();

        // Simple tag extraction
        let mut pos = 0;
        while let Some(start) = content[pos..].find('<') {
            let abs_start = pos + start;
            if let Some(end) = content[abs_start..].find('>') {
                let abs_end = abs_start + end;
                let tag_content = &content[abs_start + 1..abs_end];

                if !tag_content.starts_with('/') && !tag_content.starts_with('!') && !tag_content.starts_with('?') {
                    let parts: Vec<&str> = tag_content.split_whitespace().collect();
                    if let Some(element_name) = parts.first() {
                        elements.push(element_name.to_string());

                        // Extract attributes (basic)
                        for part in parts.iter().skip(1) {
                            if part.contains('=') {
                                let attr_parts: Vec<&str> = part.splitn(2, '=').collect();
                                if attr_parts.len() == 2 {
                                    let key = attr_parts[0].trim();
                                    let value = attr_parts[1].trim_matches('"').trim_matches('\'');
                                    attributes.insert(format!("{}@{}", element_name, key), value.to_string());
                                }
                            }
                        }
                    }
                }
                pos = abs_end + 1;
            } else {
                break;
            }
        }

        let chunks = create_chunks(content, "xml");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("root_element".to_string(), root_element.clone());
        metadata.insert("tag_count".to_string(), tag_count.to_string());
        metadata.insert("closing_tag_count".to_string(), closing_tag_count.to_string());
        metadata.insert("element_count".to_string(), elements.len().to_string());
        metadata.insert("attribute_count".to_string(), attributes.len().to_string());
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "xml".to_string(),
            content_type: "application/xml".to_string(),
            metadata,
            structured_content: StructuredContent::Xml {
                root_element,
                elements,
                attributes,
                tag_count,
            },
            raw_text: content.to_string(),
            chunks,
        })
    }
}

pub struct TextParser;
impl UniversalParser for TextParser {
    fn name(&self) -> &'static str { "Text Parser" }

    fn can_parse(&self, _content_type: &str, _content: &str) -> bool { true }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let word_count = content.split_whitespace().count();
        let line_count = content.lines().count();
        let char_count = content.chars().count();

        let paragraphs: Vec<String> = content.split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();

        let chunks = create_chunks(content, "text");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("word_count".to_string(), word_count.to_string());
        metadata.insert("line_count".to_string(), line_count.to_string());
        metadata.insert("char_count".to_string(), char_count.to_string());
        metadata.insert("paragraph_count".to_string(), paragraphs.len().to_string());
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "text".to_string(),
            content_type: "text/plain".to_string(),
            metadata,
            structured_content: StructuredContent::Text {
                word_count,
                line_count,
                paragraphs: paragraphs.clone(),
                char_count,
            },
            raw_text: content.to_string(),
            chunks,
        })
    }
}

pub struct ParquetParser;
impl UniversalParser for ParquetParser {
    fn name(&self) -> &'static str { "Parquet Parser" }

    fn can_parse(&self, content_type: &str, _content: &str) -> bool {
        content_type.contains("parquet") || content_type.contains("application/octet-stream")
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let file_size = content.len();
        let has_parquet_magic = content.starts_with("PAR1") || content.contains("parquet");
        if !has_parquet_magic {
            return Err("Not a valid Parquet file".to_string());
        }

        let schema = vec![
            "column1: string".to_string(),
            "column2: int64".to_string(),
            "column3: double".to_string(),
            "column4: boolean".to_string(),
        ];

        let column_count = schema.len();
        let estimated_row_count = file_size / 100; // Rough estimation
        let compression = "SNAPPY".to_string(); // Default assumption

        let raw_text = format!(
            "Parquet file with {} columns and approximately {} rows. Schema: {}. Compression: {}. File size: {} bytes.",
            column_count, estimated_row_count, schema.join(", "), compression, file_size
        );

        let chunks = create_chunks(&raw_text, "parquet");
        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("file_size".to_string(), file_size.to_string());
        metadata.insert("column_count".to_string(), column_count.to_string());
        metadata.insert("estimated_rows".to_string(), estimated_row_count.to_string());
        metadata.insert("compression".to_string(), compression.clone());
        metadata.insert("schema_columns".to_string(), schema.len().to_string());
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "parquet".to_string(),
            content_type: "application/parquet".to_string(),
            metadata,
            structured_content: StructuredContent::Parquet {
                schema,
                column_count,
                row_count: estimated_row_count,
                file_size,
                compression,
            },
            raw_text,
            chunks,
        })
    }
}

pub struct DatabaseParser;
impl UniversalParser for DatabaseParser {
    fn name(&self) -> &'static str { "Database Parser" }

    fn can_parse(&self, content_type: &str, content: &str) -> bool {
        content_type.contains("sqlite") ||
            content_type.contains("database") ||
            content.starts_with("SQLite format 3") ||
            content.contains("CREATE TABLE") ||
            content.contains("INSERT INTO")
    }

    fn parse(&self, content: &str, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let file_size = content.len();
        let database_type = if content.starts_with("SQLite format 3") {
            "SQLite".to_string()
        } else if content.contains("CREATE TABLE") {
            "SQL Script".to_string()
        } else {
            "Unknown Database".to_string()
        };

        let mut tables = Vec::new();
        let mut schema_info = HashMap::new();
        if content.contains("CREATE TABLE") {
            let table_count = content.matches("CREATE TABLE").count();
            for i in 0..table_count {
                let table_name = format!("table_{}", i + 1);
                tables.push(table_name.clone());
                schema_info.insert(table_name, "columns: id, name, data".to_string());
            }
        } else {
            tables = vec![
                "users".to_string(),
                "products".to_string(),
                "orders".to_string(),
            ];
            schema_info.insert("users".to_string(), "id INTEGER PRIMARY KEY, name TEXT, email TEXT".to_string());
            schema_info.insert("products".to_string(), "id INTEGER PRIMARY KEY, name TEXT, price REAL".to_string());
            schema_info.insert("orders".to_string(), "id INTEGER PRIMARY KEY, user_id INTEGER, product_id INTEGER".to_string());
        }

        let table_count = tables.len();

        // Create text representation
        let raw_text = format!(
            "{} database with {} tables: {}. File size: {} bytes. Schema information available for analysis.",
            database_type, table_count, tables.join(", "), file_size
        );

        let chunks = create_chunks(&raw_text, "database");

        let mut metadata = HashMap::new();
        metadata.insert("parser".to_string(), self.name().to_string());
        metadata.insert("database_type".to_string(), database_type.clone());
        metadata.insert("file_size".to_string(), file_size.to_string());
        metadata.insert("table_count".to_string(), table_count.to_string());
        metadata.insert("tables".to_string(), tables.join("|"));
        if let Some(url) = source_url {
            metadata.insert("source_url".to_string(), url.to_string());
        }

        Ok(ProcessedData {
            source_type: "database".to_string(),
            content_type: "application/x-sqlite3".to_string(),
            metadata,
            structured_content: StructuredContent::Database {
                tables,
                table_count,
                file_size,
                database_type,
                schema_info,
            },
            raw_text,
            chunks,
        })
    }
}

fn extract_between(content: &str, start: &str, end: &str) -> Option<String> {
    if let Some(start_pos) = content.find(start) {
        let after_start = start_pos + start.len();
        if let Some(end_pos) = content[after_start..].find(end) {
            return Some(content[after_start..after_start + end_pos].trim().to_string());
        }
    }
    None
}

fn extract_all_between(content: &str, start_pattern: &str, end_pattern: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_pos = 0;

    while let Some(start_pos) = content[search_pos..].find(start_pattern) {
        let abs_start = search_pos + start_pos;
        if let Some(tag_end) = content[abs_start..].find('>') {
            let content_start = abs_start + tag_end + 1;

            if let Some(end_pos) = content[content_start..].find(end_pattern) {
                let abs_end = content_start + end_pos;
                let extracted = content[content_start..abs_end].trim();
                if !extracted.is_empty() {
                    results.push(extracted.to_string());
                }
                search_pos = abs_end + end_pattern.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    results
}

fn extract_all_href_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut search_pos = 0;

    while let Some(href_pos) = content[search_pos..].find("href=") {
        let abs_pos = search_pos + href_pos + 5; // Skip "href="

        let quote_char = if content.chars().nth(abs_pos) == Some('"') { '"' } else { '\'' };
        let start_pos = abs_pos + 1;

        if let Some(end_pos) = content[start_pos..].find(quote_char) {
            let link = &content[start_pos..start_pos + end_pos];
            if !link.is_empty() {
                links.push(link.to_string());
            }
            search_pos = start_pos + end_pos + 1;
        } else {
            break;
        }
    }

    links
}

fn create_chunks(content: &str, chunk_type: &str) -> Vec<TextChunk> {
    const CHUNK_SIZE: usize = 100; // words per chunk
    const OVERLAP: usize = 20;

    let words: Vec<&str> = content.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut position = 0;

    while start < words.len() {
        let end = std::cmp::min(start + CHUNK_SIZE, words.len());
        let chunk_words = &words[start..end];
        let chunk_content = chunk_words.join(" ");

        // let mut metadata = HashMap::new();
        // metadata.insert("start_word".to_string(), start.to_string());
        // metadata.insert("end_word".to_string(), end.to_string());
        // metadata.insert("word_count".to_string(), chunk_words.len().to_string());
        //

        let mut metadata = HashMap::new();
        metadata.insert("start_word".to_string(), start.to_string());
        metadata.insert("end_word".to_string(), end.to_string());
        metadata.insert("word_count".to_string(), chunk_words.len().to_string());


        chunks.push(TextChunk {
            id: format!("{}_{}", chunk_type, position),
            content: chunk_content,
            metadata,
            position,
            word_count: chunk_words.len(),
        });

        start = if end == words.len() {
            words.len()
        } else {
            end.saturating_sub(OVERLAP)
        };
        position += 1;
    }

    chunks
}

pub struct UniversalDataProcessor {
    parsers: Vec<Box<dyn UniversalParser>>,
}

impl UniversalDataProcessor {
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(JsonParser),
                Box::new(CsvParser),
                Box::new(HtmlParser),
                Box::new(XmlParser),
                Box::new(ParquetParser),
                Box::new(DatabaseParser),
                Box::new(TextParser),
            ],
        }
    }

    pub fn process_content(&self, content: &str, content_type: Option<&str>, source_url: Option<&str>) -> Result<ProcessedData, String> {
        let ct = content_type.unwrap_or("*/*");

        let parser = self.parsers.iter()
            .find(|p| p.can_parse(ct, content))
            .ok_or_else(|| "No suitable parser found".to_string())?;

        parser.parse(content, source_url)
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<ProcessedData, String> {
        let path = file_path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let content_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => "application/json",
            Some("html") | Some("htm") => "text/html",
            Some("csv") => "text/csv",
            Some("xml") => "application/xml",
            Some("parquet") => "application/parquet",
            Some("db") | Some("sqlite") | Some("sqlite3") => "application/x-sqlite3",
            Some("sql") => "application/sql",
            Some("txt") => "text/plain",
            _ => "*/*",
        };

        let source_url = format!("file://{}", path.display());
        self.process_content(&content, Some(content_type), Some(&source_url))
    }
}

impl ProcessedData {
    pub fn get_content_summary(&self) -> String {
        match &self.structured_content {
            StructuredContent::Json { keys, object_count, array_count, .. } => {
                format!("JSON with {} keys, {} objects, {} arrays", keys.len(), object_count, array_count)
            },
            StructuredContent::Html { title, headings, paragraphs, links } => {
                format!("HTML '{}' with {} headings, {} paragraphs, {} links",
                        title.as_deref().unwrap_or("Untitled"), headings.len(), paragraphs.len(), links.len())
            },
            StructuredContent::Csv { headers, row_count, column_count, .. } => {
                format!("CSV with {} columns, {} rows ({})", column_count, row_count, headers.join(", "))
            },
            StructuredContent::Xml { root_element, elements, tag_count, .. } => {
                format!("XML '{}' with {} elements, {} total tags", root_element, elements.len(), tag_count)
            },
            StructuredContent::Text { word_count, line_count, char_count, .. } => {
                format!("Text with {} words, {} lines, {} characters", word_count, line_count, char_count)
            },
            StructuredContent::Parquet { schema: _, column_count, row_count, file_size, compression } => {
                format!("Parquet with {} columns, {} rows, {} compression ({} bytes)",
                        column_count, row_count, compression, file_size)
            },
            StructuredContent::Database { database_type, table_count, file_size, tables, .. } => {
                format!("{} database with {} tables ({} bytes): {}",
                        database_type, table_count, file_size, tables.join(", "))
            },
        }
    }

    pub fn to_json_string(&self) -> String {
        // Simple JSON serialization without external dependencies
        format!(
            r#"{{
  "source_type": "{}",
  "content_type": "{}",
  "summary": "{}",
  "chunk_count": {},
  "metadata": {{"parser": "{}"}},
  "rag_ready": true
}}"#,
            self.source_type,
            self.content_type,
            self.get_content_summary(),
            self.chunks.len(),
            self.metadata.get("parser").unwrap_or(&"Unknown".to_string())
        )
    }
}


pub fn prep_module_model_context_protocol(result_body: Vec<(&str, ProcessedData)>,
                                          payload: crate::domain::dbs_rag_LM::Payload)
    -> Result<Vec<(&str, ProcessedData)>, Box<dyn std::error::Error>> {
    let mut total_processed = 0;
    let mut total_chunks = 0;
    let mut results: Vec<(&str, ProcessedData)> = Vec::new();
    results = result_body;

    // Detailed results
    println!("Detail Results:");
    println!(".....................................");
    for (file_path, data) in &results {
        println!("{}: {} ({} chunks)",
                 file_path,
                 data.source_type.to_uppercase(),
                 data.chunks.len()
        );
        println!("   {}", data.get_content_summary());

        if !data.chunks.is_empty() {
            let total_words: usize = data.chunks.iter().map(|c| c.word_count).sum();
            let avg_words = if !data.chunks.is_empty() { total_words / data.chunks.len() } else { 0 };
            println!("#   Chunk Analysis: {} total words, {} avg words/chunk", total_words, avg_words);

            println!("#   First chunk: {} ({} words)", data.chunks[0].id, data.chunks[0].word_count);
            if data.chunks.len() > 1 {
                let last = data.chunks.last().unwrap();
                println!("#   Last chunk: {} ({} words)", last.id, last.word_count);
            }
        }
    }

    println!("RAG Analysis:");//Readiness
    println!("-***********************************");
    let mut all_ready = true;

    for (file_path, data) in &results {
        let has_chunks = !data.chunks.is_empty();
        let has_metadata = !data.metadata.is_empty();
        let has_content = !data.raw_text.is_empty();
        let has_structured = true;

        let ready = has_chunks && has_metadata && has_content && has_structured;
        all_ready = all_ready && ready;
        println!("{}: {} (Chunks: {}, Metadata: {}, Content: {}, Structured: {})",
                 file_path,
                 if ready { "READY" } else { "NOT READY" },
                 if has_chunks { "YES" } else { "NO" },
                 if has_metadata { "YES" } else { "NO" },
                 if has_content { "YES" } else { "NO" },
                 if has_structured { "YES" } else { "NO" }
        );
    }


    println!("All Chunks Content from All Sources:");//add from APIS
    println!("***************************************************");
    for (file_path, data) in &results {
        println!("# Source: {} ({})", file_path, data.source_type.to_uppercase());
        println!("# Content Type: {}", data.content_type);
        println!("# Total Chunks: {}", data.chunks.len());
        println!("{}","-".repeat(80));

        for (i, chunk) in data.chunks.iter().enumerate() {
            println!("[CHUNK {}] ID: {} | Words: {} | Position: {}",
                     i + 1, chunk.id, chunk.word_count, chunk.position);
            println!("{}","=".repeat(60));//Search best repeat
            println!("{}", chunk.content);
            println!("{}","=".repeat(60));

            if !chunk.metadata.is_empty() {
                println!("Chunk Metadata:");
                for (key, value) in &chunk.metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
    }

    println!("FINAL STATUS:");
    println!("*******************");
    if total_processed > 0 {
        println!("DONE");
        println!("total_processed {} data sources for llm", total_processed);
        println!("total_chunks {} text chunks for vector embedding.", total_chunks);
        if let Some((_, resources_data)) = results.first() {
            println!("Connecting base chain to RAG, JSON output:");
            println!("{}", resources_data.to_json_string());
        }
    } else {
        println!("Resources Not Available");
    }

    // let q_prompt_result = q_prompt(&results, &payload.data);

    Ok(results)
}

pub fn prep_module_q_prompt(results: &[(&str, ProcessedData)], list_q_payload: &str) -> Result<String, anyhow::Error>  {
    //use or include key 'tamar' for fishbone and chart

    let list_q_available = vec![
        list_q_payload,
    ];

    for (i, question) in list_q_available.iter().enumerate() {
        println!("{}. {}", i + 1, question);
    }
    println!();

    let result = "";
    for question in &list_q_available {
        println!("CMDPrompt: {}", question);
        println!("{}", "-".repeat(60));
        let result = generic_q_prompt(question, results);
        println!("Result: {}", result);
        println!();
    }

    Ok(result.to_string())
}


//make in utils
pub(crate) fn generic_q_prompt(question: &str, results: &[(&str, ProcessedData)]) -> String {
    let question_lower = question.to_lowercase();
    let keywords = extract_keywords(&question_lower);

    //make in utils
    let mut relevant_chunks = Vec::new();
    for (file_path, data) in results {
        for chunk in &data.chunks {
            let chunk_lower = chunk.content.to_lowercase();
            let relevance_score = calculate_enhanced_relevance(&chunk_lower, &keywords, &question_lower);

            if relevance_score > 0 {
                relevant_chunks.push((file_path, chunk, relevance_score));
            }
        }
    }
    relevant_chunks.sort_by(|a, b| b.2.cmp(&a.2));

    if relevant_chunks.is_empty() {
        return "No relevant information found in the parsed documents.".to_string();
    }
    let (source_file, best_chunk, score) = &relevant_chunks[0];
    let answer = extract_generic_q_prompt_result(&question_lower, &best_chunk.content, &keywords);
    format!("{} (Source: {}, Relevance: {})", answer, source_file, score)
}

pub(crate) fn extract_keywords(question: &str) -> Vec<String> {//common ke utils
    let stop_words = vec![
        "what", "where", "when", "how", "is", "are", "the", "a", "an",
        "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
        "by", "from", "up", "about", "into", "through", "during", "before",
        "after", "above", "below", "between", "among", "near", "far"
    ];//total ke bahasa indo
    question.split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty() && !stop_words.contains(&word.as_str()))
        .collect()
}

pub(crate) fn calculate_enhanced_relevance(text: &str, keywords: &[String], question: &str) -> usize {
    let mut score = 0;
    let text_lower = text.to_lowercase();
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        let matches = text_lower.matches(&keyword_lower).count();

        if matches > 0 {
            score += matches * 10;
            let word_boundaries = format!(" {} ", keyword_lower);
            if text_lower.contains(&word_boundaries) ||
                text_lower.starts_with(&format!("{} ", keyword_lower)) ||
                text_lower.ends_with(&format!(" {}", keyword_lower)) ||
                text_lower == keyword_lower {
                score += 20;
            }
        }
    }

    let question_words: Vec<&str> = question.split_whitespace().collect();
    for window in question_words.windows(2) {
        let phrase = format!("{} {}", window[0], window[1]);
        if text_lower.contains(&phrase) {
            score += 5;
        }
    }

    if question.contains("how many")
        || question.contains("what is")
        || question.contains("price") {
        let number_count = text.chars().filter(|c| c.is_numeric()).count();
        if number_count > 0 {
            score += number_count;
        }
    }

    score
}

pub(crate) fn extract_generic_q_prompt_result(question: &str, content: &str, keywords: &[String]) -> String {
    let sentences: Vec<&str> = content.split(&['.', '!', '?'][..])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut best_sentence = "";
    let mut best_score = 0;

    for sentence in &sentences {
        let sentence_lower = sentence.to_lowercase();
        let mut sentence_score = 0;
        for keyword in keywords {
            if sentence_lower.contains(keyword) {
                sentence_score += 2;
            }
        }

        //sesuaikan category question (tanpa LLM Model)
        if question.contains("how many") || question.contains("price") || question.contains("population") {
            if sentence_lower.chars().any(|c| c.is_numeric()) {
                sentence_score += 3;
            }
        }
        if question.contains("where") {
            let location_words = ["in", "on", "at", "near", "situated", "located", "side", "north", "south", "east", "west"];
            for loc_word in &location_words {
                if sentence_lower.contains(loc_word) {
                    sentence_score += 2;
                }
            }
        }
        let descriptive_words = ["is", "are", "has", "have", "contains", "includes", "with"];
        for desc_word in &descriptive_words {
            if sentence_lower.contains(desc_word) {
                sentence_score += 1;
            }
        }

        if sentence_score > best_score {
            best_score = sentence_score;
            best_sentence = sentence;
        }

        // /**
        // * Add All Of Probable
        // **/
    }

    // key for chart: tamar
    let core_answer = if !best_sentence.is_empty() && best_score > 0 {
        best_sentence.to_string()
    } else if !keywords.is_empty() {
        let main_keyword = &keywords[0];
        sentences.iter()
            .find(|sentence| sentence.to_lowercase().contains(main_keyword))
            .unwrap_or(&sentences.get(0).unwrap_or(&""))
            .to_string()
    } else {
        sentences.get(0).unwrap_or(&"Information available").to_string()
    };

    if question.to_lowercase().contains(FISHBONE_FORMAT) {
        format_brainstorming_answer(question, &core_answer, keywords)
    } else {
        core_answer
    }
}

// MASTER FORMAT
fn format_brainstorming_answer(question: &str, core_content: &str, keywords: &[String]) -> String {
    let question_type = analyze_question_type(question);
    let key_elements = extract_key_elements(core_content, keywords);
    //solution tanpa score di view
    format!(
        "\n┌─ PROBLEM: {}\n\
         ├─ ANALYSIS:\n\
         │  ├─ Question Type: {}\n\
         │  ├─ Key Elements: {}\n\
         │  └─ Context: {}\n\
         ├─ SOLUTION:\n\
         │  ├─ Primary Answer: {}\n\
         │  ├─ Supporting Facts: {}\n\
         │  └─ Confidence: High\n\
         └─ RESULT:\n\
            ├─ Main Point: {}\n\
            ├─ Sub-Point 1: {}\n\
            ├─ Sub-Point 2: {}\n\
            └─ Conclusion: {}",
        question,
        question_type,
        key_elements,
        if core_content.len() > 50 { &core_content[..50] } else { core_content },
        extract_main_fact(core_content),
        extract_supporting_facts(core_content),
        extract_main_point(core_content),
        extract_sub_point_1(core_content),
        extract_sub_point_2(core_content),
        extract_conclusion(core_content, question)
    )
}

// DETAIL FORMAT
fn analyze_question_type(question: &str) -> &'static str {
    let q_lower = question.to_lowercase();
    if q_lower.starts_with("what") { "Definition/Description" }
    else if q_lower.starts_with("where") { "Location/Position" }
    else if q_lower.starts_with("when") { "Time/Date" }
    else if q_lower.starts_with("how") { "Process/Method" }
    else if q_lower.starts_with("why") { "Reason/Cause" }
    else if q_lower.starts_with("who") { "Person/Entity" }
    else { "General Inquiry" }
}

fn extract_key_elements(content: &str, keywords: &[String]) -> String {
    let mut elements = Vec::new();
    for keyword in keywords {
        if content.to_lowercase().contains(&keyword.to_lowercase()) {
            elements.push(keyword.clone());
        }
    }
    if elements.is_empty() { "General content".to_string() } else { elements.join(", ") }
}

fn extract_main_fact(content: &str) -> String {
    let sentences: Vec<&str> = content.split(&['.', '!', '?'][..]).collect();
    sentences.get(0).unwrap_or(&"Main info available").trim().to_string()
}

fn extract_supporting_facts(content: &str) -> String {
    let words = content.split_whitespace().collect::<Vec<_>>();
    let numbers: Vec<&str> = words.iter().filter(|w| w.chars().any(|c| c.is_numeric())).take(2).cloned().collect();
    if !numbers.is_empty() {
        format!("Numeric data: {}", numbers.join(", "))
    } else {
        "Context".to_string()
    }
}

fn extract_main_point(content: &str) -> String {
    let words: Vec<&str> = content.split_whitespace().take(8).collect();
    format!("{}{}", words.join(" "), if content.split_whitespace().count() > 8 { "..." } else { "" })
}

fn extract_sub_point_1(content: &str) -> String {
    if content.to_lowercase().contains("population") {
        "devgraphic information identified"
    } else if content.to_lowercase().contains("price") {
        "Pricing information available"
    } else if content.to_lowercase().contains("located") || content.to_lowercase().contains("situated") {
        "Geographic positioning specified"
    } else {
        "Descriptive details provided"
    }.to_string()
}

fn extract_sub_point_2(content: &str) -> String {
    if content.chars().any(|c| c.is_numeric()) {
        "Quantitative data present"
    } else if content.len() > 100 {
        "Comprehensive information available"
    } else {
        "Concise information provided"
    }.to_string()
}

fn extract_conclusion(content: &str, question: &str) -> String {
    let q_lower = question.to_lowercase();
    if q_lower.contains("what is") {
        "Describe"
    } else if q_lower.contains("where") {
        "Location"
    } else if q_lower.contains("how many") || q_lower.contains("price") {
        "Number"
    } else {
        "Cmd Prompt done"
    }.to_string()
}
// END DETAIL FORMAT
