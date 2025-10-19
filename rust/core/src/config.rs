use std::env;
use std::path::PathBuf;

const SANITIZER_ENV: &str = "KELIVO_SANITIZER_IMAGE_DIR";

pub fn sanitizer_env_var_name() -> &'static str {
    SANITIZER_ENV
}

pub fn sanitizer_image_dir() -> PathBuf {
    if let Ok(explicit) = env::var(SANITIZER_ENV) {
        return PathBuf::from(explicit);
    }

    if let Some(user_dirs) = directories::UserDirs::new() {
        if let Some(documents) = user_dirs.document_dir() {
            return documents.join("images");
        }
    }

    env::temp_dir().join("kelivo").join("images")
}
