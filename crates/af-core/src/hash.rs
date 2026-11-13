//! SHA-256 hashing for concurrency checks and result metadata.

use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_empty_string() {
        let h = sha256_hex(b"");
        assert_eq!(h.len(), 64);
    }
}

