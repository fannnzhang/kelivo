use std::io::{Cursor, Read, Write};

use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[flutter_rust_bridge::frb]
#[derive(Debug, Clone)]
pub struct BackupZipEntryInput {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[flutter_rust_bridge::frb]
#[derive(Debug, Clone)]
pub struct BackupZipEntry {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[flutter_rust_bridge::frb]
#[derive(Debug, Clone)]
pub struct WebDavEntry {
    pub href: String,
    pub display_name: String,
    pub size: u64,
    pub last_modified_rfc3339: Option<String>,
    pub is_directory: bool,
}

#[flutter_rust_bridge::frb]
pub fn create_backup_zip(entries: Vec<BackupZipEntryInput>) -> Result<Vec<u8>, String> {
    if entries.is_empty() {
        return Err("no entries provided".to_string());
    }

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut cursor);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        for entry in entries {
            let path = sanitize_zip_path(&entry.path)?;
            if entry.is_dir {
                writer
                    .add_directory(path, options)
                    .map_err(|err| format!("failed to add directory to zip: {err}"))?;
            } else {
                writer
                    .start_file(path, options)
                    .map_err(|err| format!("failed to add file to zip: {err}"))?;
                writer
                    .write_all(&entry.data)
                    .map_err(|err| format!("failed to write zip file content: {err}"))?;
            }
        }

        writer
            .finish()
            .map_err(|err| format!("failed to finish zip: {err}"))?;
    }

    Ok(cursor.into_inner())
}

#[flutter_rust_bridge::frb]
pub fn extract_backup_zip(bytes: Vec<u8>) -> Result<Vec<BackupZipEntry>, String> {
    if bytes.is_empty() {
        return Err("zip data is empty".to_string());
    }

    let cursor = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|err| format!("failed to read zip archive: {err}"))?;
    let mut results = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|err| format!("failed to read zip entry {i}: {err}"))?;
        let mut data = Vec::new();
        if file.is_dir() {
            results.push(BackupZipEntry {
                path: file
                    .mangled_name()
                    .to_string_lossy()
                    .trim_start_matches('/')
                    .to_string(),
                data,
                is_dir: true,
            });
        } else {
            file.read_to_end(&mut data)
                .map_err(|err| format!("failed to read zip file contents: {err}"))?;
            results.push(BackupZipEntry {
                path: file
                    .mangled_name()
                    .to_string_lossy()
                    .trim_start_matches('/')
                    .to_string(),
                data,
                is_dir: false,
            });
        }
    }

    Ok(results)
}

fn sanitize_zip_path(path: &str) -> Result<String, String> {
    let cleaned = path.trim();
    if cleaned.is_empty() {
        return Err("zip entry path cannot be empty".to_string());
    }

    let normalized = cleaned.replace('\\', "/");
    if normalized.contains("../") || normalized.starts_with('/') {
        return Err(format!("invalid zip entry path: {normalized}"));
    }

    Ok(if normalized.ends_with('/') {
        normalized
    } else {
        normalized.to_string()
    })
}

#[derive(Default, Debug)]
struct ParsedResponse {
    href: Option<String>,
    display_name: Option<String>,
    size: Option<u64>,
    last_modified: Option<String>,
    is_directory: bool,
}

fn parse_webdav_propfind_impl(xml: String, base_url: String) -> Result<Vec<WebDavEntry>, String> {
    let xml_bytes = xml.into_bytes();
    let mut cursor = Cursor::new(xml_bytes);
    let mut reader = Reader::from_reader(&mut cursor);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut results = Vec::new();
    let mut current: Option<ParsedResponse> = None;
    let mut current_field: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                if name == "response" {
                    current = Some(ParsedResponse::default());
                } else if let Some(resp) = current.as_mut() {
                    match name.as_str() {
                        "href" | "displayname" | "getcontentlength" | "getlastmodified" => {
                            current_field = Some(name.to_string());
                        }
                        "collection" => {
                            resp.is_directory = true;
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if let (Some(field), Some(resp)) = (current_field.as_deref(), current.as_mut()) {
                    let txt = e
                        .unescape()
                        .map_err(|err| format!("failed to decode XML text: {err}"))?
                        .trim()
                        .to_string();
                    match field {
                        "href" => resp.href = Some(txt),
                        "displayname" => resp.display_name = Some(txt),
                        "getcontentlength" => {
                            resp.size = txt.parse::<u64>().ok();
                        }
                        "getlastmodified" => resp.last_modified = Some(txt),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref());
                if name == "response" {
                    if let Some(resp) = current.take() {
                        if let Some(entry) = finalize_response(resp, &base_url) {
                            results.push(entry);
                        }
                    }
                } else if current_field.as_deref() == Some(name.as_str()) {
                    current_field = None;
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => {
                return Err(format!("failed to parse WebDAV response XML: {err}"));
            }
        }
        buf.clear();
    }

    Ok(results)
}

#[flutter_rust_bridge::frb]
pub fn parse_webdav_propfind(xml: String, base_url: String) -> Result<Vec<WebDavEntry>, String> {
    parse_webdav_propfind_impl(xml, base_url)
}

fn finalize_response(resp: ParsedResponse, base_url: &str) -> Option<WebDavEntry> {
    let href = resp.href?;
    let href_trimmed = href.trim();
    if href_trimmed.is_empty() {
        return None;
    }

    let base = base_url.trim_end_matches('/');
    let resolved = if href_trimmed.starts_with("http://") || href_trimmed.starts_with("https://") {
        href_trimmed.to_string()
    } else if href_trimmed.starts_with('/') {
        format!("{base}{href_trimmed}")
    } else {
        format!("{base}/{}", href_trimmed.trim_start_matches('/'))
    };

    if resolved.trim_end_matches('/') == base {
        return None;
    }

    let is_directory = resp.is_directory || resolved.ends_with('/') || href_trimmed.ends_with('/');

    let display_name = resp
        .display_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| {
            resolved
                .split('/')
                .filter(|segment| !segment.is_empty())
                .next_back()
                .unwrap_or_default()
                .to_string()
        });

    let size = resp.size.unwrap_or(0);
    let last_modified_rfc3339 = resp.last_modified.and_then(|value| {
        DateTime::parse_from_rfc2822(value.trim())
            .ok()
            .map(|dt| DateTime::<Utc>::from(dt).to_rfc3339())
    });

    Some(WebDavEntry {
        href: resolved,
        display_name,
        size,
        last_modified_rfc3339,
        is_directory,
    })
}

fn local_name(name: &[u8]) -> String {
    let full = std::str::from_utf8(name).unwrap_or_default();
    full.rsplit(':').next().unwrap_or(full).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn creates_and_extracts_zip() {
        let entries = vec![
            BackupZipEntryInput {
                path: "settings.json".into(),
                data: br#"{"foo":"bar"}"#.to_vec(),
                is_dir: false,
            },
            BackupZipEntryInput {
                path: "upload/".into(),
                data: vec![],
                is_dir: true,
            },
            BackupZipEntryInput {
                path: "upload/file.txt".into(),
                data: b"hello world".to_vec(),
                is_dir: false,
            },
        ];

        let bytes = create_backup_zip(entries).expect("zip bytes");
        let extracted = extract_backup_zip(bytes).expect("extract zip");

        let mut map = HashMap::new();
        for entry in extracted {
            map.insert(entry.path.clone(), (entry.is_dir, entry.data));
        }

        assert!(map.contains_key("settings.json"));
        assert!(!map.get("settings.json").unwrap().0);
        assert_eq!(
            String::from_utf8(map.get("settings.json").unwrap().1.clone()).unwrap(),
            r#"{"foo":"bar"}"#
        );
        assert!(map.contains_key("upload"));
        assert!(map.get("upload").unwrap().0);
        assert!(map.contains_key("upload/file.txt"));
    }

    #[test]
    fn parses_webdav_response() {
        let xml = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>https://example.com/backups/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>backups</d:displayname>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>https://example.com/backups/kelivo_backup.zip</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>kelivo_backup.zip</d:displayname>
        <d:getcontentlength>1234</d:getcontentlength>
        <d:getlastmodified>Tue, 15 Oct 2024 12:34:56 GMT</d:getlastmodified>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>
"#
        .to_string();

        let entries =
            parse_webdav_propfind(xml, "https://example.com/backups/".to_string()).expect("parse");
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.display_name, "kelivo_backup.zip");
        assert_eq!(entry.size, 1234);
        assert!(!entry.is_directory);
        assert!(entry
            .last_modified_rfc3339
            .as_ref()
            .unwrap()
            .starts_with("2024-10-15T12:34:56"));
    }

    #[test]
    fn rejects_unsafe_paths() {
        let entries = vec![BackupZipEntryInput {
            path: "../bad".into(),
            data: vec![],
            is_dir: false,
        }];

        assert!(create_backup_zip(entries).is_err());
    }

    #[test]
    fn handles_empty_zip_bytes() {
        assert!(extract_backup_zip(Vec::new()).is_err());
    }
}
