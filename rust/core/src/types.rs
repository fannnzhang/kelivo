pub type ByteBuf = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPath(pub String);

impl NormalizedPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
