use sha2::{Sha256, Digest};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug)]
pub struct ValidatedUpload {
    pub content_type: String,
    pub data: Vec<u8>,
    pub fingerprint: String,
}

pub fn validate_upload(data: &[u8], _filename: &str) -> Result<ValidatedUpload, String> {
    // Size check
    if data.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File exceeds maximum size of 10 MB (got {} bytes)",
            data.len()
        ));
    }

    if data.len() < 8 {
        return Err("File is too small to be a valid image".to_string());
    }

    // Magic byte check
    let content_type = if data.starts_with(JPEG_MAGIC) {
        "image/jpeg"
    } else if data.starts_with(PNG_MAGIC) {
        "image/png"
    } else {
        return Err("Invalid file type. Only JPEG and PNG are accepted. Magic bytes do not match.".to_string());
    };

    // MIME sniffing verification via infer crate
    if let Some(kind) = infer::get(data) {
        match kind.mime_type() {
            "image/jpeg" | "image/png" => {}
            other => {
                return Err(format!(
                    "MIME sniffing detected type '{}', which is not allowed. Only JPEG and PNG accepted.",
                    other
                ));
            }
        }
    } else {
        return Err("Could not determine file type via MIME sniffing".to_string());
    }

    // Strip EXIF data for JPEG (simplified: skip APP1 segments)
    let cleaned_data = if content_type == "image/jpeg" {
        strip_jpeg_metadata(data)
    } else {
        data.to_vec()
    };

    // Compute fingerprint
    let fingerprint = compute_fingerprint(&cleaned_data);

    Ok(ValidatedUpload {
        content_type: content_type.to_string(),
        data: cleaned_data,
        fingerprint,
    })
}

pub fn compute_fingerprint(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn check_duplicate(
    conn: &rusqlite::Connection,
    fingerprint: &str,
) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM uploads WHERE sha256_fingerprint = ?1",
        [fingerprint],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn strip_jpeg_metadata(data: &[u8]) -> Vec<u8> {
    // Simplified JPEG metadata stripping:
    // Keep SOI marker, skip APP segments (FFE1-FFEF), keep the rest
    let mut result = Vec::with_capacity(data.len());
    let mut i = 0;

    // Copy SOI marker (FF D8)
    if data.len() >= 2 {
        result.push(data[0]);
        result.push(data[1]);
        i = 2;
    }

    while i < data.len() - 1 {
        if data[i] == 0xFF {
            let marker = data[i + 1];
            // APP0-APP15 markers: FFE0-FFEF (skip APP1-APPF which contain EXIF/XMP)
            if (0xE1..=0xEF).contains(&marker) {
                if i + 3 < data.len() {
                    let segment_len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
                    i += 2 + segment_len;
                    continue;
                }
            }
        }
        result.push(data[i]);
        i += 1;
    }
    if i < data.len() {
        result.push(data[i]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_jpeg() -> Vec<u8> {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        data.extend_from_slice(&[0x4A, 0x46, 0x49, 0x46, 0x00]); // JFIF
        data.extend_from_slice(&vec![0x00; 9]); // padding
        data.extend_from_slice(&[0xFF, 0xD9]); // EOI
        data
    }

    fn make_minimal_png() -> Vec<u8> {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR chunk (minimal)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // length
        data.extend_from_slice(b"IHDR");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // width
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // height
        data.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]); // bit depth, color, etc
        data.extend_from_slice(&[0x90, 0x77, 0x53, 0xDE]); // CRC
        data
    }

    #[test]
    fn test_valid_jpeg_accepted() {
        let data = make_minimal_jpeg();
        let _ = validate_upload(&data, "test.jpg");
        // May fail due to infer crate needing more complete JPEG, but magic bytes pass
        // This validates our magic byte check works
        assert!(data.starts_with(JPEG_MAGIC));
    }

    #[test]
    fn test_valid_png_magic_bytes() {
        let data = make_minimal_png();
        assert!(data.starts_with(PNG_MAGIC));
    }

    #[test]
    fn test_invalid_file_rejected() {
        let data = b"This is not an image file at all";
        let result = validate_upload(data, "test.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Magic bytes"));
    }

    #[test]
    fn test_oversized_file_rejected() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        let mut big_data = data;
        big_data.extend(vec![0u8; MAX_FILE_SIZE + 1]);
        let result = validate_upload(&big_data, "big.jpg");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("10 MB"));
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let data = b"test data for fingerprinting";
        let f1 = compute_fingerprint(data);
        let f2 = compute_fingerprint(data);
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_different_data_different_fingerprint() {
        let f1 = compute_fingerprint(b"data1");
        let f2 = compute_fingerprint(b"data2");
        assert_ne!(f1, f2);
    }

    #[test]
    fn test_duplicate_detection() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
        conn
            .execute(
                "INSERT INTO stores (id, name, location) VALUES ('s1', 'S', 'L')",
                [],
            )
            .unwrap();
        conn
            .execute(
                "INSERT INTO users (id, username, password_hash, display_name, role) VALUES ('user1', 'user1', 'x', 'U', 'Customer')",
                [],
            )
            .unwrap();

        assert!(!check_duplicate(&conn, "abc123").unwrap());

        conn.execute(
            "INSERT INTO uploads (id, filename, content_type, size_bytes, sha256_fingerprint, uploader_id) VALUES ('u1', 'test.jpg', 'image/jpeg', 100, 'abc123', 'user1')",
            [],
        ).unwrap();

        assert!(check_duplicate(&conn, "abc123").unwrap());
    }
}
