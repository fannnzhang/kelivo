use std::io::{Cursor, Read, Write};
use std::path::Path;

use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::KelivoResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupZipEntryInput {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupZipEntry {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDavEntry {
    pub href: String,
    pub display_name: String,
    pub size: u64,
    pub last_modified_rfc3339: Option<String>,
    pub is_directory: bool,
}

pub fn create_backup_zip(entries: &[BackupZipEntryInput]) -> KelivoResult<Vec<u8>> {
    if entries.is_empty() {
        return Err("no entries provided".into());
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

pub fn extract_backup_zip(bytes: &[u8]) -> KelivoResult<Vec<BackupZipEntry>> {
    if bytes.is_empty() {
        return Err("zip data is empty".into());
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
                path: strip_leading_slash(file.mangled_name().as_ref()),
                data,
                is_dir: true,
            });
        } else {
            file.read_to_end(&mut data)
                .map_err(|err| format!("failed to read zip file contents: {err}"))?;
            results.push(BackupZipEntry {
                path: strip_leading_slash(file.mangled_name().as_ref()),
                data,
                is_dir: false,
            });
        }
    }

    Ok(results)
}

pub fn parse_webdav_propfind(xml: &str, base_href: &str) -> KelivoResult<Vec<WebDavEntry>> {
    if xml.trim().is_empty() {
        return Err("webdav response is empty".into());
    }

    let mut reader = Reader::from_str(xml);
    reader.trim_text(false);
    let mut buf = Vec::new();

    let mut entries = Vec::new();
    let mut current_href = None;
    let mut current_display = None;
    let mut current_size: Option<u64> = None;
    let mut current_last_modified: Option<String> = None;
    let mut is_directory = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"d:href" | b"href" => {
                    current_href = reader
                        .read_text(e.name())
                        .ok()
                        .map(|text| text.trim().to_string());
                }
                b"d:displayname" | b"displayname" => {
                    current_display = reader
                        .read_text(e.name())
                        .ok()
                        .map(|text| text.trim().to_string());
                }
                b"d:resourcetype" | b"resourcetype" => {
                    is_directory = false;
                }
                b"d:collection" | b"collection" => {
                    is_directory = true;
                }
                b"d:getcontentlength" | b"getcontentlength" => {
                    current_size = reader
                        .read_text(e.name())
                        .ok()
                        .and_then(|value| value.trim().parse::<u64>().ok());
                }
                b"d:getlastmodified" | b"getlastmodified" => {
                    current_last_modified = reader
                        .read_text(e.name())
                        .ok()
                        .map(|value| value.trim().to_string());
                }
                _ => {}
            },
            Ok(Event::End(ref e))
                if e.name().as_ref() == b"d:response" || e.name().as_ref() == b"response" =>
            {
                if let Some(entry) = build_webdav_entry(
                    base_href,
                    current_href.take(),
                    current_display.take(),
                    current_size.take(),
                    current_last_modified.take(),
                    is_directory,
                ) {
                    entries.push(entry);
                }

                is_directory = false;
            }
            Ok(_) => {}
            Err(err) => return Err(format!("failed to parse WebDAV XML: {err}").into()),
        }
        buf.clear();
    }

    Ok(entries)
}

fn build_webdav_entry(
    base_href: &str,
    raw_href: Option<String>,
    display_name: Option<String>,
    size: Option<u64>,
    last_modified: Option<String>,
    is_directory: bool,
) -> Option<WebDavEntry> {
    let href = raw_href?;
    let normalized = normalize_href(base_href, &href);

    if normalized.trim_end_matches('/') == base_href.trim_end_matches('/') {
        return None;
    }

    let display = display_name.unwrap_or_else(|| {
        normalized
            .split('/')
            .filter(|segment| !segment.is_empty())
            .next_back()
            .unwrap_or_default()
            .to_string()
    });

    let last_modified_rfc3339 = last_modified.and_then(|value| {
        DateTime::parse_from_rfc2822(value.trim())
            .ok()
            .map(|dt| DateTime::<Utc>::from(dt).to_rfc3339())
    });

    Some(WebDavEntry {
        href: normalized,
        display_name: display,
        size: size.unwrap_or(0),
        last_modified_rfc3339,
        is_directory,
    })
}

fn sanitize_zip_path(path: &str) -> KelivoResult<String> {
    let cleaned = path.trim();
    if cleaned.is_empty() {
        return Err("zip entry path cannot be empty".into());
    }

    let normalized = cleaned.replace('\\', "/");
    if normalized.contains("../") || normalized.starts_with('/') {
        return Err(format!("invalid zip entry path: {normalized}").into());
    }

    Ok(if normalized.ends_with('/') {
        normalized
    } else {
        normalized.to_string()
    })
}

fn strip_leading_slash(path: &Path) -> String {
    path.to_string_lossy().trim_start_matches('/').to_string()
}

fn normalize_href(base: &str, href: &str) -> String {
    if href.starts_with(base) {
        return href.trim().to_string();
    }

    if href.starts_with('/') {
        format!("{base}{}", href.trim_start_matches('/'))
    } else {
        let separator = if base.ends_with('/') { "" } else { "/" };
        format!("{base}{separator}{href}")
    }
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

        let bytes = create_backup_zip(&entries).expect("zip bytes");
        let extracted = extract_backup_zip(&bytes).expect("extract zip");

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
    fn rejects_unsafe_paths() {
        let entries = vec![BackupZipEntryInput {
            path: "../bad".into(),
            data: vec![],
            is_dir: false,
        }];

        assert!(create_backup_zip(&entries).is_err());
    }

    #[test]
    fn rejects_absolute_paths() {
        let entries = vec![BackupZipEntryInput {
            path: "/etc/passwd".into(),
            data: vec![],
            is_dir: false,
        }];

        assert!(create_backup_zip(&entries).is_err());
    }

    #[test]
    fn normalizes_windows_separators() {
        let entries = vec![BackupZipEntryInput {
            path: "folder\\file.txt".into(),
            data: b"abc".to_vec(),
            is_dir: false,
        }];

        let bytes = create_backup_zip(&entries).expect("zip");
        let extracted = extract_backup_zip(&bytes).expect("extract");
        assert_eq!(extracted[0].path, "folder/file.txt");
    }

    #[test]
    fn handles_empty_zip_bytes() {
        assert!(extract_backup_zip(&[]).is_err());
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
"#;

        let entries = parse_webdav_propfind(xml, "https://example.com/backups/").expect("parse");
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
}
