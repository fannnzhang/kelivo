use flutter_rust_bridge::frb;
use kelivo_core::markdown as core;

#[frb]
pub fn replace_inline_base64_images(markdown: String) -> Result<String, String> {
    core::replace_inline_base64_images(&markdown).map_err(|err| err.to_string())
}

#[frb]
pub fn inline_local_images_to_base64(markdown: String) -> Result<String, String> {
    core::inline_local_images_to_base64(&markdown).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::encode as encode_b64;
    use std::env;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn set_test_images_dir(dir: &Path) {
        env::set_var(
            kelivo_core::config::sanitizer_env_var_name(),
            dir.to_string_lossy().to_string(),
        );
    }

    fn unset_test_images_dir() {
        env::remove_var(kelivo_core::config::sanitizer_env_var_name());
    }

    #[test]
    fn replaces_inline_base64_images_and_writes_files() {
        let temp = tempdir().expect("temp dir");
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
        let absolute = std::path::PathBuf::from(replaced_path);
        let file_bytes = fs::read(&absolute).expect("read written file");
        assert_eq!(file_bytes, payload_bytes);
    }

    #[test]
    fn inlines_local_images_to_base64() {
        let temp = tempdir().expect("temp dir");
        let file_path = temp.path().join("sample.png");
        fs::write(&file_path, b"png-bytes").expect("write sample file");

        let markdown = format!("![alt]({})", file_path.to_string_lossy());
        let result = inline_local_images_to_base64(markdown.clone()).expect("ok result");

        assert!(result.contains("data:image/png;base64"));
    }
}
