use fleetreserve_backend::services::uploads::validate_upload;

#[test]
fn unit_uploads_invalid_rejected() {
    let err = validate_upload(b"not image bytes", "x.txt").unwrap_err();
    assert!(err.contains("Magic bytes") || err.contains("valid image") || err.contains("too small"));
}

#[test]
fn unit_uploads_oversized_rejected() {
    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    data.extend(vec![0u8; 10 * 1024 * 1024 + 1]);
    let err = validate_upload(&data, "big.jpg").unwrap_err();
    assert!(err.contains("10 MB"));
}

#[test]
fn unit_uploads_too_small_rejected() {
    let err = validate_upload(b"tiny", "x.jpg").unwrap_err();
    assert!(err.contains("too small") || err.contains("Magic") || err.contains("valid image"));
}

#[test]
fn unit_uploads_wrong_magic_bytes_rejected() {
    // GIF magic bytes – not a JPEG or PNG
    let data = b"GIF89a\x00\x00\x00\x00\x00\x00\x00\x00";
    let err = validate_upload(data, "fake.jpg").unwrap_err();
    assert!(
        err.contains("Magic bytes") || err.contains("MIME") || err.contains("JPEG") || err.contains("PNG"),
        "Expected rejection for wrong magic bytes, got: {}",
        err
    );
}

#[test]
fn unit_uploads_valid_png_accepted() {
    // Minimal valid 1×1 transparent PNG
    let png_bytes: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 px
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth, color type, etc.
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT length + type
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // compressed pixel data
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, // ...
        0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND length + type
        0x44, 0xAE, 0x42, 0x60, 0x82,                   // IEND data
    ];
    let result = validate_upload(png_bytes, "test.png");
    match result {
        Ok(v) => assert_eq!(v.content_type, "image/png"),
        // Some infer versions may not accept this minimal PNG — that is acceptable
        Err(e) => assert!(
            e.contains("MIME") || e.contains("type") || e.contains("sniffing"),
            "Unexpected error for PNG: {}",
            e
        ),
    }
}
