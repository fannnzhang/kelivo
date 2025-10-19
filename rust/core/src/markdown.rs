use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use base64::{decode as decode_b64, encode as encode_b64};
use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

use crate::config;
use crate::fs::{normalize_slash_path, resolve_absolute, write_bytes};
use crate::KelivoResult;

lazy_static! {
    static ref INLINE_BASE64_RE: Regex =
        Regex::new(r"!\[[^\]]*\]\((data:image/[a-zA-Z0-9.+\-]+;base64,[a-zA-Z0-9+/=\r\n]+)\)",)
            .expect("invalid base64 image regex");
    static ref INLINE_IMAGE_RE: Regex =
        Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").expect("invalid image regex");
}

pub fn replace_inline_base64_images(markdown: &str) -> KelivoResult<String> {
    replace_inline_base64_images_impl(markdown).map_err(Into::into)
}

pub fn inline_local_images_to_base64(markdown: &str) -> KelivoResult<String> {
    inline_local_images_to_base64_impl(markdown).map_err(Into::into)
}

fn replace_inline_base64_images_impl(markdown: &str) -> anyhow::Result<String> {
    if !markdown.contains("data:image") {
        return Ok(markdown.to_string());
    }

    let images_dir = resolve_images_dir()?;
    let mut output = String::with_capacity(markdown.len());
    let mut last_end = 0usize;

    for caps in INLINE_BASE64_RE.captures_iter(markdown) {
        let matched = caps
            .get(0)
            .ok_or_else(|| anyhow!("missing regex match for inline base64"))?;
        output.push_str(&markdown[last_end..matched.start()]);

        let data_url = caps
            .get(1)
            .ok_or_else(|| anyhow!("missing capture group for data uri"))?
            .as_str();

        let (mime, payload) = parse_data_url(data_url)?;
        let normalized = normalize_base64(&payload);
        let bytes = decode_base64(&normalized)?;
        let extension = mime_to_extension(&mime);
        let file_name = build_file_name(&normalized, extension);
        let file_path = images_dir.join(file_name);

        if !file_path.exists() {
            write_bytes(&file_path, &bytes)?;
        }

        let path_str = normalize_slash_path(&file_path);
        let replaced_segment = matched.as_str().replacen(data_url, &path_str, 1);
        output.push_str(&replaced_segment);
        last_end = matched.end();
    }

    output.push_str(&markdown[last_end..]);
    Ok(output)
}

fn inline_local_images_to_base64_impl(markdown: &str) -> anyhow::Result<String> {
    if !markdown.contains('!') || !markdown.contains("](") {
        return Ok(markdown.to_string());
    }

    let mut output = String::with_capacity(markdown.len());
    let mut last_end = 0usize;

    for caps in INLINE_IMAGE_RE.captures_iter(markdown) {
        let matched = caps
            .get(0)
            .ok_or_else(|| anyhow!("missing regex match for inline image"))?;
        output.push_str(&markdown[last_end..matched.start()]);

        let url = caps
            .get(1)
            .ok_or_else(|| anyhow!("missing capture group for image src"))?
            .as_str()
            .trim();

        if !is_local_image_path(url) {
            output.push_str(matched.as_str());
            last_end = matched.end();
            continue;
        }

        let path = resolve_absolute(url);
        match fs::read(&path) {
            Ok(bytes) => {
                let mime = guess_mime_from_path(&path);
                let encoded = encode_b64(bytes);
                let data_url = format!("data:{};base64,{}", mime, encoded);
                let replaced_segment = matched.as_str().replacen(url, &data_url, 1);
                output.push_str(&replaced_segment);
            }
            Err(_) => {
                output.push_str(matched.as_str());
            }
        }

        last_end = matched.end();
    }

    output.push_str(&markdown[last_end..]);
    Ok(output)
}

fn resolve_images_dir() -> anyhow::Result<PathBuf> {
    let dir = config::sanitizer_image_dir();
    fs::create_dir_all(&dir).with_context(|| {
        format!(
            "unable to create sanitizer image directory {}",
            dir.display()
        )
    })?;
    Ok(dir)
}

fn parse_data_url(data_url: &str) -> anyhow::Result<(String, String)> {
    if !data_url.starts_with("data:") {
        return Err(anyhow!("data url missing data: prefix"));
    }

    let semicolon_index = data_url
        .find(';')
        .ok_or_else(|| anyhow!("data url missing ';' separator"))?;
    let comma_index = data_url
        .find(',')
        .ok_or_else(|| anyhow!("data url missing ',' separator"))?;

    let mime = data_url[5..semicolon_index].to_string();
    let encoding = &data_url[semicolon_index + 1..comma_index];
    if !encoding.to_ascii_lowercase().starts_with("base64") {
        return Err(anyhow!("data url is not base64 encoded"));
    }

    let payload = data_url[comma_index + 1..].to_string();
    Ok((mime, payload))
}

fn normalize_base64(data: &str) -> String {
    data.chars()
        .filter(|c| !matches!(c, '\n' | '\r' | ' '))
        .collect::<String>()
}

fn decode_base64(payload: &str) -> anyhow::Result<Vec<u8>> {
    decode_b64(payload).map_err(|err| anyhow!("failed to decode base64 payload: {err}"))
}

fn build_file_name(normalized_payload: &str, extension: &str) -> String {
    let digest = Uuid::new_v5(&Uuid::NAMESPACE_URL, normalized_payload.as_bytes());
    format!("img_{}.{}", digest, extension)
}

fn mime_to_extension(mime: &str) -> &'static str {
    let lower = mime.to_ascii_lowercase();
    match lower.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/svg" | "image/svg+xml" => "svg",
        "image/x-icon" => "ico",
        "image/avif" => "avif",
        "image/heic" | "image/heif" => "heic",
        _ => "png",
    }
}

fn guess_mime_from_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "webp" => "image/webp",
        Some(ext) if ext == "gif" => "image/gif",
        Some(ext) if ext == "bmp" => "image/bmp",
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "ico" => "image/x-icon",
        Some(ext) if ext == "avif" => "image/avif",
        Some(ext) if ext == "heic" || ext == "heif" => "image/heic",
        _ => "image/png",
    }
}

fn is_local_image_path(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return false;
    }

    lower.starts_with("file://") || url.starts_with('/') || url.contains(':') || url.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    fn set_test_images_dir(dir: &Path) {
        env::set_var(
            config::sanitizer_env_var_name(),
            dir.to_string_lossy().to_string(),
        );
    }

    fn unset_test_images_dir() {
        env::remove_var(config::sanitizer_env_var_name());
    }

    #[test]
    fn replaces_inline_base64_images_and_writes_files() {
        let temp = tempdir().expect("temp dir");
        let images_dir = temp.path().join("images");
        set_test_images_dir(&images_dir);

        let payload_bytes = b"test-png".to_vec();
        let payload_b64 = encode_b64(&payload_bytes);
        let markdown = format!("# Title\n![sample](data:image/png;base64,{payload_b64})\n");

        let result = replace_inline_base64_images(&markdown).expect("ok result");
        unset_test_images_dir();

        assert!(result.contains("/images/"));
        let link_start = result
            .find("](")
            .map(|idx| idx + 2)
            .expect("image link start");
        let link_end = result[link_start..]
            .find(')')
            .map(|delta| link_start + delta)
            .expect("image link end");
        let replaced_path = &result[link_start..link_end];
        let absolute = PathBuf::from(replaced_path);
        let file_bytes = fs::read(&absolute).expect("read written file");
        assert_eq!(file_bytes, payload_bytes);
    }

    #[test]
    fn returns_error_on_invalid_base64_payload() {
        let temp = tempdir().expect("temp dir");
        let images_dir = temp.path().join("images");
        set_test_images_dir(&images_dir);

        assert!(decode_b64("====").is_err());
        let markdown = "![bad](data:image/png;base64,====)".to_string();
        let result = replace_inline_base64_images(&markdown);
        unset_test_images_dir();
        assert!(result.is_err());
    }

    #[test]
    fn reuses_existing_files_for_identical_payloads() {
        let temp = tempdir().expect("temp dir");
        let images_dir = temp.path().join("images");
        set_test_images_dir(&images_dir);

        let payload = encode_b64(b"duplicate");
        let markdown = format!(
            "![one](data:image/png;base64,{payload})\n![two](data:image/png;base64,{payload})"
        );

        let result = replace_inline_base64_images(&markdown).expect("ok result");
        unset_test_images_dir();

        let matches: Vec<_> = INLINE_IMAGE_RE
            .captures_iter(&result)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], matches[1]);
    }

    #[test]
    fn supports_extended_mime_types() {
        let temp = tempdir().expect("temp dir");
        set_test_images_dir(temp.path());

        for (mime, expected_ext) in [
            ("image/svg+xml", "svg"),
            ("image/avif", "avif"),
            ("image/heic", "heic"),
        ] {
            let payload = encode_b64(b"img");
            let markdown = format!("![x](data:{mime};base64,{payload})");
            let result = replace_inline_base64_images(&markdown).expect("ok result");
            assert!(result.contains(expected_ext));
        }

        unset_test_images_dir();
    }

    #[test]
    fn inlines_local_images_to_base64() {
        let temp = tempdir().expect("temp dir");
        let file_path = temp.path().join("sample.png");
        fs::write(&file_path, b"png-bytes").expect("write sample file");

        let markdown = format!("![alt]({})", file_path.to_string_lossy());
        let result = inline_local_images_to_base64(&markdown).expect("ok result");

        assert!(result.contains("data:image/png;base64"));

        let start = result.find("data:image/png;base64,").unwrap();
        let data_sub = &result[start + "data:image/png;base64,".len()..];
        let end = data_sub.find(')').unwrap();
        let encoded = &data_sub[..end];
        let decoded = decode_b64(encoded).expect("decode base64 from result");
        assert_eq!(decoded, b"png-bytes");
    }

    #[test]
    fn skips_nonexistent_local_files() {
        let markdown = "![alt](/tmp/does_not_exist.png)".to_string();
        let result = inline_local_images_to_base64(&markdown).expect("ok result");
        assert_eq!(result, markdown);
    }

    #[test]
    fn leaves_remote_urls_untouched() {
        let markdown = "![alt](https://example.com/image.png)".to_string();
        let result = inline_local_images_to_base64(&markdown).expect("ok result");
        assert_eq!(result, markdown);
    }
}
