use std::fs::File;
use std::io::Read;
use std::path::Path;

use pdf_extract;
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

#[flutter_rust_bridge::frb]
pub fn extract_text_from_pdf(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    pdf_extract::extract_text(path).map_err(|err| format!("failed to extract PDF text: {err}"))
}

#[flutter_rust_bridge::frb]
pub fn extract_text_from_docx(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    let file = File::open(path)
        .map_err(|err| format!("failed to open DOCX file {}: {err}", path.display()))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("failed to read DOCX archive {}: {err}", path.display()))?;

    let mut document_xml = archive
        .by_name("word/document.xml")
        .map_err(|err| format!("DOCX missing word/document.xml: {err}"))?;

    let mut xml = String::new();
    document_xml
        .read_to_string(&mut xml)
        .map_err(|err| format!("failed to read DOCX XML: {err}"))?;

    parse_docx_xml(&xml)
}

#[flutter_rust_bridge::frb]
pub fn read_text_fallback(path: String) -> Result<String, String> {
    let bytes =
        std::fs::read(&path).map_err(|err| format!("failed to read file {}: {err}", path))?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn parse_docx_xml(xml: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(false);

    let mut buf = Vec::new();
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_text = false;
    let mut preserve_space = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = true;
                preserve_space = e.attributes().with_checks(false).flatten().any(|attr| {
                    attr.key.as_ref() == b"xml:space" && attr.value.as_ref() == b"preserve"
                });
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = false;
                preserve_space = false;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                paragraphs.push(current.clone());
                current.clear();
            }
            Ok(Event::Text(e)) if in_text => {
                let text = e
                    .unescape()
                    .map_err(|err| format!("failed to decode DOCX text: {err}"))?;
                if preserve_space {
                    current.push_str(&text);
                } else {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        // Skip purely whitespace runs when not preserving space.
                    } else {
                        let needs_space = current
                            .chars()
                            .rev()
                            .find(|c| !c.is_control())
                            .map(|c| !c.is_whitespace())
                            .unwrap_or(false);
                        if needs_space {
                            current.push(' ');
                        }
                        current.push_str(trimmed);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(format!("failed to parse DOCX XML: {err}")),
        }
        buf.clear();
    }

    if !current.is_empty() {
        paragraphs.push(current);
    }

    Ok(paragraphs.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::Path;
    use tempfile::tempdir;
    use zip::write::FileOptions;
    use zip::CompressionMethod;

    #[test]
    fn extracts_pdf_text() {
        let dir = tempdir().expect("temp dir");
        let pdf_path = dir.path().join("sample.pdf");
        create_sample_pdf(&pdf_path, "Hello from PDF").expect("create sample pdf");

        let text = extract_text_from_pdf(pdf_path.to_string_lossy().to_string())
            .expect("extract text from pdf");
        assert!(
            text.contains("Hello from PDF"),
            "expected extracted text to contain 'Hello from PDF', got: {text}"
        );
    }

    #[test]
    fn extracts_docx_text() {
        let dir = tempdir().expect("temp dir");
        let docx_path = dir.path().join("sample.docx");
        create_sample_docx(&docx_path, &["Hello DOCX", "Second paragraph"])
            .expect("create sample docx");

        let text = extract_text_from_docx(docx_path.to_string_lossy().to_string())
            .expect("extract text from docx");
        let mut lines = text.lines();
        assert_eq!(lines.next(), Some("Hello DOCX"));
        assert_eq!(lines.next(), Some("Second paragraph"));
    }

    #[test]
    fn reads_text_fallback_with_lossy_utf8() {
        let dir = tempdir().expect("temp dir");
        let txt_path = dir.path().join("sample.txt");
        std::fs::write(&txt_path, b"Hello \xF0\x9F\x92\xA9 World\xC3").expect("write sample text");

        let text =
            read_text_fallback(txt_path.to_string_lossy().to_string()).expect("read text fallback");
        assert!(
            text.contains("Hello"),
            "expected fallback text to contain 'Hello', got: {text}"
        );
        assert!(
            text.contains("World"),
            "expected fallback text to contain 'World', got: {text}"
        );
    }

    // #[test]
    // #[ignore]
    // fn print_sample_artifacts() {
    //     use base64::{engine::general_purpose::STANDARD, Engine};
    //
    //     let dir = tempdir().expect("temp dir");
    //     let pdf_path = dir.path().join("sample.pdf");
    //     create_sample_pdf(&pdf_path, "Sample PDF from Rust").expect("create pdf");
    //     let docx_path = dir.path().join("sample.docx");
    //     create_sample_docx(&docx_path, &["Sample DOCX from Rust"]).expect("create docx");
    //
    //     let pdf_b64 = STANDARD.encode(std::fs::read(&pdf_path).expect("read pdf"));
    //     let docx_b64 = STANDARD.encode(std::fs::read(&docx_path).expect("read docx"));
    //
    //     println!("PDF_BASE64={pdf_b64}");
    //     println!("DOCX_BASE64={docx_b64}");
    // }

    fn create_sample_pdf(path: &Path, text: &str) -> Result<(), Box<dyn Error>> {
        let catalog_id = Ref::new(1);
        let pages_id = Ref::new(2);
        let page_id = Ref::new(3);
        let font_id = Ref::new(4);
        let content_id = Ref::new(5);
        let font_name = Name(b"F1");

        let mut pdf = Pdf::new();
        pdf.catalog(catalog_id).pages(pages_id);
        pdf.pages(pages_id).kids([page_id]).count(1);
        pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

        {
            let mut page = pdf.page(page_id);
            page.parent(pages_id);
            page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
            page.contents(content_id);
            page.resources().fonts().pair(font_name, font_id);
            page.finish();
        }

        let mut content = Content::new();
        content.begin_text();
        content.set_font(font_name, 18.0);
        content.next_line(50.0, 750.0);
        content.show(Str(text.as_bytes()));
        content.end_text();
        pdf.stream(content_id, &content.finish());

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&pdf.finish())?;
        writer.flush()?;
        Ok(())
    }

    fn create_sample_docx(path: &Path, paragraphs: &[&str]) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(CompressionMethod::Stored);

        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )?;

        zip.start_file("_rels/.rels", options)?;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )?;

        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
        )?;

        zip.start_file("word/document.xml", options)?;
        let mut body = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
"#,
        );

        for paragraph in paragraphs {
            body.push_str("    <w:p><w:r><w:t>");
            body.push_str(&escape_xml(paragraph));
            body.push_str("</w:t></w:r></w:p>\n");
        }

        body.push_str("  </w:body>\n</w:document>");
        zip.write_all(body.as_bytes())?;
        zip.finish();
        Ok(())
    }

    fn escape_xml(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&apos;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
}
