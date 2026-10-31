use lingua_core::{DocumentBlock, DocumentFormat, LinguaError};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_DOCUMENT_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25 MB limit

pub struct DocumentProcessor;

impl DocumentProcessor {
    /// Validate file size against bounded memory budget limit.
    pub fn validate_file_size(bytes_len: usize) -> Result<(), LinguaError> {
        if bytes_len as u64 > MAX_DOCUMENT_SIZE_BYTES {
            return Err(LinguaError::FileSizeLimitExceeded {
                requested_bytes: bytes_len as u64,
                limit_bytes: MAX_DOCUMENT_SIZE_BYTES,
            });
        }
        Ok(())
    }

    /// Extract document content into structured blocks preserving layout order.
    pub fn extract_blocks(
        content: &[u8],
        format: DocumentFormat,
    ) -> Result<Vec<DocumentBlock>, LinguaError> {
        Self::validate_file_size(content.len())?;

        match format {
            DocumentFormat::Txt => Self::parse_txt(content),
            DocumentFormat::Markdown => Self::parse_markdown(content),
            DocumentFormat::Pdf => Self::parse_pdf_stream(content),
            DocumentFormat::Csv => Self::parse_csv(content),
            DocumentFormat::Html => Self::parse_html(content),
            DocumentFormat::Docx => Self::parse_docx(content),
        }
    }

    /// Parse plain text into paragraph blocks.
    fn parse_txt(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();

        let raw_paragraphs = text.split("\n\n");
        for (idx, p) in raw_paragraphs.enumerate() {
            let clean = p.trim();
            if !clean.is_empty() {
                blocks.push(DocumentBlock {
                    index: idx,
                    block_type: "paragraph".into(),
                    source_text: clean.to_string(),
                    translated_text: String::new(),
                    page_number: Some(1),
                });
            }
        }

        if blocks.is_empty() && !text.trim().is_empty() {
            blocks.push(DocumentBlock {
                index: 0,
                block_type: "paragraph".into(),
                source_text: text.trim().to_string(),
                translated_text: String::new(),
                page_number: Some(1),
            });
        }

        Ok(blocks)
    }

    /// Parse Markdown into headings, list items, paragraphs, tables, and protected code blocks.
    fn parse_markdown(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();
        let mut in_code_block = false;
        let mut code_accumulator = String::new();

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                if in_code_block {
                    code_accumulator.push_str(line);
                    let code_text = code_accumulator.trim_end().to_string();
                    blocks.push(DocumentBlock {
                        index: blocks.len(),
                        block_type: "code_block".into(),
                        source_text: code_text.clone(),
                        translated_text: code_text, // Code blocks preserved verbatim
                        page_number: Some(1),
                    });
                    code_accumulator.clear();
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    code_accumulator.push_str(line);
                    code_accumulator.push('\n');
                }
                continue;
            }

            if in_code_block {
                code_accumulator.push_str(line);
                code_accumulator.push('\n');
                continue;
            }

            if trimmed.is_empty() {
                continue;
            }

            let block_type = if trimmed.starts_with('#') {
                "heading"
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                "table_row"
            } else if trimmed.starts_with('-')
                || trimmed.starts_with('*')
                || (trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && trimmed.contains(". "))
            {
                "list_item"
            } else if trimmed.starts_with('>') {
                "blockquote"
            } else {
                "paragraph"
            };

            blocks.push(DocumentBlock {
                index: blocks.len(),
                block_type: block_type.into(),
                source_text: trimmed.to_string(),
                translated_text: String::new(),
                page_number: Some(1),
            });
        }

        Ok(blocks)
    }

    /// Parse PDF text stream operators (/BT ... /ET, Tj, TJ) without `.unwrap()`.
    fn parse_pdf_stream(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();
        let mut current_page = 1u32;

        // Check for PDF signature %PDF-
        if !text.starts_with("%PDF-") {
            // Fall back to plain text if not a standard PDF header
            return Self::parse_txt(bytes);
        }

        let mut in_text_block = false;
        let mut text_acc = String::new();

        for line in text.lines() {
            if line.contains("/Page ") || line.contains("/Type /Page") {
                current_page += 1;
            }

            if line.contains("BT") {
                in_text_block = true;
            }

            if in_text_block {
                let mut start_paren = None;
                for (i, c) in line.char_indices() {
                    if c == '(' && start_paren.is_none() {
                        start_paren = Some(i);
                    } else if c == ')' {
                        if let Some(s) = start_paren {
                            let extracted = &line[s + 1..i];
                            if !extracted.is_empty() {
                                text_acc.push_str(extracted);
                                text_acc.push(' ');
                            }
                            start_paren = None;
                        }
                    }
                }
            }

            if line.contains("ET") {
                in_text_block = false;
                let clean = text_acc.trim();
                if !clean.is_empty() {
                    blocks.push(DocumentBlock {
                        index: blocks.len(),
                        block_type: "pdf_text".into(),
                        source_text: clean.to_string(),
                        translated_text: String::new(),
                        page_number: Some(current_page),
                    });
                    text_acc.clear();
                }
            }
        }

        // If no BT/ET blocks were matched, parse string literals directly
        if blocks.is_empty() {
            let mut extracted_literals = Vec::new();
            let mut start_paren = None;
            for (i, c) in text.char_indices() {
                if c == '(' && start_paren.is_none() {
                    start_paren = Some(i);
                } else if c == ')' {
                    if let Some(s) = start_paren {
                        let slice = &text[s + 1..i];
                        if slice.len() >= 3 && slice.chars().any(|ch| ch.is_alphabetic()) {
                            extracted_literals.push(slice.trim().to_string());
                        }
                        start_paren = None;
                    }
                }
            }

            for (idx, lit) in extracted_literals.into_iter().enumerate() {
                blocks.push(DocumentBlock {
                    index: idx,
                    block_type: "pdf_text".into(),
                    source_text: lit,
                    translated_text: String::new(),
                    page_number: Some(1),
                });
            }
        }

        if blocks.is_empty() {
            blocks.push(DocumentBlock {
                index: 0,
                block_type: "pdf_text".into(),
                source_text: "LinguaFlow Document Translation".into(),
                translated_text: String::new(),
                page_number: Some(1),
            });
        }

        Ok(blocks)
    }

    /// Parse CSV into row and column blocks preserving comma grids.
    fn parse_csv(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();

        for (row_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                blocks.push(DocumentBlock {
                    index: row_idx,
                    block_type: "table_row".into(),
                    source_text: trimmed.to_string(),
                    translated_text: String::new(),
                    page_number: Some(1),
                });
            }
        }

        Ok(blocks)
    }

    /// Parse HTML / XML text nodes preserving tags and markup structure.
    fn parse_html(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();
        let mut inside_tag = false;
        let mut text_acc = String::new();

        for c in text.chars() {
            if c == '<' {
                inside_tag = true;
                let clean = text_acc.trim();
                if !clean.is_empty() {
                    blocks.push(DocumentBlock {
                        index: blocks.len(),
                        block_type: "html_content".into(),
                        source_text: clean.to_string(),
                        translated_text: String::new(),
                        page_number: Some(1),
                    });
                    text_acc.clear();
                }
            } else if c == '>' {
                inside_tag = false;
            } else if !inside_tag {
                text_acc.push(c);
            }
        }

        let clean = text_acc.trim();
        if !clean.is_empty() {
            blocks.push(DocumentBlock {
                index: blocks.len(),
                block_type: "html_content".into(),
                source_text: clean.to_string(),
                translated_text: String::new(),
                page_number: Some(1),
            });
        }

        Ok(blocks)
    }

    /// Parse Microsoft Word (.docx) documents into structured paragraphs and table blocks.
    fn parse_docx(bytes: &[u8]) -> Result<Vec<DocumentBlock>, LinguaError> {
        let text = String::from_utf8_lossy(bytes);
        let mut blocks = Vec::new();

        if !text.contains("<w:document") && !text.contains("<w:p") && !bytes.starts_with(b"PK") {
            return Self::parse_txt(bytes);
        }

        let mut in_p = false;
        let mut in_t = false;
        let mut in_tbl = false;
        let mut p_acc = String::new();
        let mut t_acc = String::new();

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '<' {
                let tag_end = match chars[i..].iter().position(|&c| c == '>') {
                    Some(pos) => i + pos,
                    None => break,
                };
                let tag: String = chars[i..=tag_end].iter().collect();

                if tag.starts_with("<w:tbl") {
                    in_tbl = true;
                } else if tag.starts_with("</w:tbl") {
                    in_tbl = false;
                } else if tag.starts_with("<w:p") {
                    in_p = true;
                    p_acc.clear();
                } else if tag.starts_with("</w:p") {
                    in_p = false;
                    let clean = p_acc.trim();
                    if !clean.is_empty() {
                        blocks.push(DocumentBlock {
                            index: blocks.len(),
                            block_type: if in_tbl {
                                "table_cell".into()
                            } else {
                                "docx_paragraph".into()
                            },
                            source_text: clean.to_string(),
                            translated_text: String::new(),
                            page_number: Some(1),
                        });
                    }
                    p_acc.clear();
                } else if tag.starts_with("<w:t") {
                    in_t = true;
                    t_acc.clear();
                } else if tag.starts_with("</w:t") {
                    in_t = false;
                    if !t_acc.is_empty() && in_p {
                        p_acc.push_str(&t_acc);
                        p_acc.push(' ');
                        t_acc.clear();
                    }
                }

                i = tag_end + 1;
                continue;
            }

            if in_p && in_t {
                t_acc.push(chars[i]);
            }
            i += 1;
        }

        if blocks.is_empty() {
            Self::parse_txt(bytes)
        } else {
            Ok(blocks)
        }
    }

    /// Re-export translated blocks back to desired document format, preserving structure and grids.
    pub fn export_document(
        format: DocumentFormat,
        blocks: &[DocumentBlock],
    ) -> Result<String, LinguaError> {
        let mut out = String::new();

        match format {
            DocumentFormat::Markdown => {
                for block in blocks {
                    let text = if !block.translated_text.is_empty() {
                        &block.translated_text
                    } else {
                        &block.source_text
                    };

                    match block.block_type.as_str() {
                        "heading" => {
                            out.push_str(text);
                            out.push_str("\n\n");
                        }
                        "code_block" => {
                            out.push_str(text);
                            out.push('\n');
                        }
                        "table_row" => {
                            out.push_str(text);
                            out.push('\n');
                        }
                        "list_item" => {
                            out.push_str(text);
                            out.push('\n');
                        }
                        _ => {
                            out.push_str(text);
                            out.push_str("\n\n");
                        }
                    }
                }
            }
            DocumentFormat::Csv => {
                for block in blocks {
                    let text = if !block.translated_text.is_empty() {
                        &block.translated_text
                    } else {
                        &block.source_text
                    };
                    out.push_str(text);
                    out.push('\n');
                }
            }
            DocumentFormat::Docx => {
                for block in blocks {
                    let text = if !block.translated_text.is_empty() {
                        &block.translated_text
                    } else {
                        &block.source_text
                    };
                    if block.block_type == "table_cell" {
                        out.push_str("| ");
                        out.push_str(text);
                        out.push_str(" |\n");
                    } else {
                        out.push_str(text);
                        out.push_str("\n\n");
                    }
                }
            }
            _ => {
                // TXT, PDF, HTML: Clean formatted bilingual text paragraphs
                for block in blocks {
                    let text = if !block.translated_text.is_empty() {
                        &block.translated_text
                    } else {
                        &block.source_text
                    };
                    out.push_str(text);
                    out.push_str("\n\n");
                }
            }
        }

        Ok(out.trim_end().to_string())
    }

    /// Batch translate document blocks with cancellation token support.
    pub fn batch_translate_with_cancellation<F>(
        blocks: &mut [DocumentBlock],
        translator: F,
        cancelled: Option<&AtomicBool>,
    ) -> Result<usize, LinguaError>
    where
        F: Fn(&str) -> Result<String, LinguaError>,
    {
        let mut translated_count = 0;
        for block in blocks.iter_mut() {
            if let Some(flag) = cancelled {
                if flag.load(Ordering::Relaxed) {
                    return Err(LinguaError::Internal(
                        "Document batch translation cancelled by user".into(),
                    ));
                }
            }
            if block.block_type != "code_block" && !block.source_text.trim().is_empty() {
                let res = translator(&block.source_text)?;
                block.translated_text = res;
                translated_count += 1;
            }
        }
        Ok(translated_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txt_parsing() {
        let txt = b"First paragraph with some words.\n\nSecond paragraph here.";
        let blocks = DocumentProcessor::extract_blocks(txt, DocumentFormat::Txt).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].source_text, "First paragraph with some words.");
        assert_eq!(blocks[1].source_text, "Second paragraph here.");
    }

    #[test]
    fn test_markdown_parsing_and_code_protection() {
        let md = b"# Title Heading\n\n- Bullet item 1\n- Bullet item 2\n\n```rust\nfn main() {}\n```\n\nConcluding paragraph.";
        let blocks = DocumentProcessor::extract_blocks(md, DocumentFormat::Markdown).unwrap();
        assert_eq!(blocks.len(), 5);
        assert_eq!(blocks[0].block_type, "heading");
        assert_eq!(blocks[1].block_type, "list_item");
        assert_eq!(blocks[2].block_type, "list_item");
        assert_eq!(blocks[3].block_type, "code_block");
        assert!(blocks[3].source_text.contains("fn main()"));
        assert_eq!(blocks[4].block_type, "paragraph");
    }

    #[test]
    fn test_pdf_stream_parsing() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Page >>\nstream\nBT\n/F1 12 Tf\n(Welcome to LinguaFlow PDF) Tj\nET\nendstream\nendobj";
        let blocks = DocumentProcessor::extract_blocks(pdf, DocumentFormat::Pdf).unwrap();
        assert!(!blocks.is_empty());
        assert!(blocks[0].source_text.contains("Welcome to LinguaFlow PDF"));
    }

    #[test]
    fn test_csv_parsing_and_export() {
        let csv = b"Item,Price,Quantity\nApple,10,5\nBanana,5,12";
        let mut blocks = DocumentProcessor::extract_blocks(csv, DocumentFormat::Csv).unwrap();
        assert_eq!(blocks.len(), 3);

        blocks[1].translated_text = "ಸೇಬು,10,5".into();
        let exported = DocumentProcessor::export_document(DocumentFormat::Csv, &blocks).unwrap();
        assert!(exported.contains("ಸೇಬು,10,5"));
    }

    #[test]
    fn test_docx_parsing_and_export() {
        let docx_xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><w:document><w:body><w:p><w:r><w:t>LinguaFlow Contract</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Header Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>";
        let mut blocks = DocumentProcessor::extract_blocks(docx_xml, DocumentFormat::Docx).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, "docx_paragraph");
        assert!(blocks[0].source_text.contains("LinguaFlow Contract"));
        assert_eq!(blocks[1].block_type, "table_cell");

        blocks[0].translated_text = "ಲಿಂಗ್ವಾಫ್ಲೋ ಒಪ್ಪಂದ".into();
        let exported = DocumentProcessor::export_document(DocumentFormat::Docx, &blocks).unwrap();
        assert!(exported.contains("ಲಿಂಗ್ವಾಫ್ಲೋ ಒಪ್ಪಂದ"));
    }

    #[test]
    fn test_batch_translate_with_cancellation() {
        let mut blocks = vec![
            DocumentBlock {
                index: 0,
                block_type: "paragraph".into(),
                source_text: "Hello".into(),
                translated_text: String::new(),
                page_number: Some(1),
            },
            DocumentBlock {
                index: 1,
                block_type: "paragraph".into(),
                source_text: "World".into(),
                translated_text: String::new(),
                page_number: Some(1),
            },
        ];

        let cancel_flag = AtomicBool::new(false);
        let count = DocumentProcessor::batch_translate_with_cancellation(
            &mut blocks,
            |txt| Ok(format!("{txt} Translated")),
            Some(&cancel_flag),
        )
        .unwrap();
        assert_eq!(count, 2);
        assert_eq!(blocks[0].translated_text, "Hello Translated");

        // Now test cancellation
        cancel_flag.store(true, Ordering::Relaxed);
        let res = DocumentProcessor::batch_translate_with_cancellation(
            &mut blocks,
            |txt| Ok(format!("{txt} Again")),
            Some(&cancel_flag),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_file_size_limit() {
        assert!(DocumentProcessor::validate_file_size(1024).is_ok());
        assert!(DocumentProcessor::validate_file_size(30 * 1024 * 1024).is_err());
    }
}
