use std::path::PathBuf;

use flutter_rust_bridge::frb;
use kelivo_core::document as core;

#[frb]
pub fn extract_text_from_pdf(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    core::extract_text_from_pdf(&path).map_err(|err| err.to_string())
}

#[frb]
pub fn extract_text_from_docx(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    core::extract_text_from_docx(&path).map_err(|err| err.to_string())
}

#[frb]
pub fn read_text_fallback(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    core::read_text_fallback(&path).map_err(|err| err.to_string())
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
        assert!(text.contains("Hello from PDF"));
    }

    #[test]
    fn extracts_docx_text() {
        let dir = tempdir().expect("temp dir");
        let docx_path = dir.path().join("sample.docx");
        create_sample_docx(&docx_path, &["Hello", "World"]).expect("create docx");

        let text = extract_text_from_docx(docx_path.to_string_lossy().to_string())
            .expect("extract text from docx");
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn read_text_fallback_handles_invalid_utf8() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("sample.txt");
        std::fs::write(&path, b"Hello\xffWorld").expect("write sample file");

        let text =
            read_text_fallback(path.to_string_lossy().to_string()).expect("read text fallback");
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

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
