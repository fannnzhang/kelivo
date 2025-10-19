use flutter_rust_bridge::frb;
use kelivo_core::zip_webdav as core;

#[frb]
#[derive(Debug, Clone)]
pub struct BackupZipEntryInput {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[frb]
#[derive(Debug, Clone)]
pub struct BackupZipEntry {
    pub path: String,
    pub data: Vec<u8>,
    pub is_dir: bool,
}

#[frb]
#[derive(Debug, Clone)]
pub struct WebDavEntry {
    pub href: String,
    pub display_name: String,
    pub size: u64,
    pub last_modified_rfc3339: Option<String>,
    pub is_directory: bool,
}

impl From<BackupZipEntryInput> for core::BackupZipEntryInput {
    fn from(value: BackupZipEntryInput) -> Self {
        Self {
            path: value.path,
            data: value.data,
            is_dir: value.is_dir,
        }
    }
}

impl From<core::BackupZipEntry> for BackupZipEntry {
    fn from(value: core::BackupZipEntry) -> Self {
        Self {
            path: value.path,
            data: value.data,
            is_dir: value.is_dir,
        }
    }
}

impl From<core::WebDavEntry> for WebDavEntry {
    fn from(value: core::WebDavEntry) -> Self {
        Self {
            href: value.href,
            display_name: value.display_name,
            size: value.size,
            last_modified_rfc3339: value.last_modified_rfc3339,
            is_directory: value.is_directory,
        }
    }
}

#[frb]
pub fn create_backup_zip(entries: Vec<BackupZipEntryInput>) -> Result<Vec<u8>, String> {
    let converted: Vec<core::BackupZipEntryInput> = entries.into_iter().map(Into::into).collect();
    core::create_backup_zip(&converted).map_err(|err| err.to_string())
}

#[frb]
pub fn extract_backup_zip(bytes: Vec<u8>) -> Result<Vec<BackupZipEntry>, String> {
    core::extract_backup_zip(&bytes)
        .map(|entries| entries.into_iter().map(Into::into).collect())
        .map_err(|err| err.to_string())
}

#[frb]
pub fn parse_webdav_propfind(xml: String, base_href: String) -> Result<Vec<WebDavEntry>, String> {
    core::parse_webdav_propfind(&xml, &base_href)
        .map(|entries| entries.into_iter().map(Into::into).collect())
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn round_trip_backup_zip() {
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
        assert!(map.contains_key("upload"));
        assert!(map.get("upload").unwrap().0);
        assert!(map.contains_key("upload/file.txt"));
    }

    #[test]
    fn parses_webdav_documents() {
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
        assert_eq!(entries[0].display_name, "kelivo_backup.zip");
    }

    #[test]
    fn rejects_invalid_zip_path() {
        let entries = vec![BackupZipEntryInput {
            path: "../bad".into(),
            data: vec![],
            is_dir: false,
        }];

        assert!(create_backup_zip(entries).is_err());
    }
}
