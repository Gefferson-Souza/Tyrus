use sha2::{Digest, Sha256};

pub(crate) type Sha256Bytes = [u8; 32];

pub(crate) fn sha256(bytes: &[u8]) -> Sha256Bytes {
    Sha256::digest(bytes).into()
}

pub(crate) fn to_hex(digest: &Sha256Bytes) -> String {
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        for nibble in [byte >> 4, byte & 0x0f] {
            out.extend(char::from_digit(u32::from(nibble), 16));
        }
    }
    out
}
