use anyhow::{anyhow, Context, Result as AnyResult};
use base64::{decode as decode_b64, encode as encode_b64};
use directories::UserDirs;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

lazy_static! {
    static ref INLINE_BASE64_RE: Regex =
        Regex::new(r"!\[[^\]]*\]\((data:image/[a-zA-Z0-9.+\-]+;base64,[a-zA-Z0-9+/=\r\n]+)\)")
            .expect("invalid base64 image regex");
    static ref INLINE_IMAGE_RE: Regex =
        Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").expect("invalid image regex");
}

#[flutter_rust_bridge::frb]
pub fn replace_inline_base64_images(markdown: String) -> Result<String, String> {
    replace_inline_base64_images_impl(&markdown).map_err(|err| err.to_string())
}

#[flutter_rust_bridge::frb]
pub fn inline_local_images_to_base64(markdown: String) -> Result<String, String> {
    inline_local_images_to_base64_impl(&markdown).map_err(|err| err.to_string())
}

fn replace_inline_base64_images_impl(markdown: &str) -> AnyResult<String> {
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

        let path_str = to_slash_path(&file_path);
        let replaced_segment = matched.as_str().replacen(data_url, &path_str, 1);
        output.push_str(&replaced_segment);
        last_end = matched.end();
    }

    output.push_str(&markdown[last_end..]);
    Ok(output)
}

fn inline_local_images_to_base64_impl(markdown: &str) -> AnyResult<String> {
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

        let path = resolve_local_path(url);
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

fn resolve_images_dir() -> AnyResult<PathBuf> {
    if let Ok(explicit) = env::var("KELIVO_SANITIZER_IMAGE_DIR") {
        let path = PathBuf::from(explicit);
        fs::create_dir_all(&path).context("unable to create configured images directory")?;
        return Ok(path);
    }

    if let Some(user_dirs) = UserDirs::new() {
        if let Some(documents) = user_dirs.document_dir() {
            let images_dir = documents.join("images");
            fs::create_dir_all(&images_dir)
                .context("unable to create documents/images directory")?;
            return Ok(images_dir);
        }
    }

    let fallback = env::temp_dir().join("kelivo").join("images");
    fs::create_dir_all(&fallback).context("unable to create fallback images directory")?;
    Ok(fallback)
}

fn parse_data_url(data_url: &str) -> AnyResult<(String, String)> {
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

fn decode_base64(payload: &str) -> AnyResult<Vec<u8>> {
    decode_b64(payload).map_err(|err| anyhow!("failed to decode base64 payload: {err}"))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> AnyResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory for {}", path.display()))?;
    }

    let mut file =
        File::create(path).with_context(|| format!("failed to create file {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write file {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush file {}", path.display()))?;
    Ok(())
}

fn build_file_name(normalized_payload: &str, extension: &str) -> String {
    let digest = Uuid::new_v5(&Uuid::NAMESPACE_URL, normalized_payload.as_bytes());
    format!("img_{}.{}", digest, extension)
}

fn to_slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn mime_to_extension(mime: &str) -> &'static str {
    let lower = mime.to_ascii_lowercase();
    match lower.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/svg" | "image/svg+xml" => "svg",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        "image/avif" => "avif",
        "image/heic" | "image/heif" => "heic",
        _ => "png",
    }
}

fn is_local_image_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    let lower = path.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("asset:")
    {
        return false;
    }

    lower.starts_with("file://")
        || path.starts_with('/')
        || path.contains(':')
        || path.contains('\\')
}

fn resolve_local_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("file://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn set_test_images_dir(dir: &Path) {
        env::set_var(
            "KELIVO_SANITIZER_IMAGE_DIR",
            dir.to_string_lossy().to_string(),
        );
    }

    fn unset_test_images_dir() {
        env::remove_var("KELIVO_SANITIZER_IMAGE_DIR");
    }

    #[test]
    fn replaces_inline_base64_images_and_writes_files() {
        let temp = tempdir().expect("create temp dir");
        let images_dir = temp.path().join("images");
        set_test_images_dir(&images_dir);

        let payload_bytes = b"test-png".to_vec();
        let payload_b64 = encode_b64(&payload_bytes);
        let markdown = format!("# Title\n![sample](data:image/png;base64,{payload_b64})\n");

        let result = replace_inline_base64_images(markdown.clone()).expect("ok result");
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
        let temp = tempdir().expect("create temp dir");
        let images_dir = temp.path().join("images");
        set_test_images_dir(&images_dir);

        assert!(decode_b64("====").is_err());
        let markdown = "![bad](data:image/png;base64,====)".to_string();
        let result = replace_inline_base64_images(markdown.clone());
        unset_test_images_dir();
        assert!(result.is_err());
    }

    #[test]
    fn inlines_local_images_to_base64() {
        let temp = tempdir().expect("create temp dir");
        let file_path = temp.path().join("sample.png");
        fs::write(&file_path, b"png-bytes").expect("write sample file");

        let markdown = format!("![alt]({})", file_path.to_string_lossy());
        let result = inline_local_images_to_base64(markdown.clone()).expect("ok result");

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
        let result = inline_local_images_to_base64(markdown.clone()).expect("ok result");
        assert_eq!(result, markdown);
    }

    #[test]
    fn leaves_remote_urls_untouched() {
        let markdown = "![alt](https://example.com/image.png)".to_string();
        let result = inline_local_images_to_base64(markdown.clone()).expect("ok result");
        assert_eq!(result, markdown);
    }
}
