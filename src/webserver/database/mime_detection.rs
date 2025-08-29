/// Detects MIME type based on file signatures (magic bytes).
/// Returns the most appropriate MIME type for common file formats.
pub fn detect_mime_type(bytes: &[u8]) -> &'static str {
    if bytes.is_empty() {
        return "application/octet-stream";
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return "image/png";
    }
    // JPEG: FF D8
    if bytes.starts_with(b"\xFF\xD8") {
        return "image/jpeg";
    }
    // GIF87a/89a: GIF87a or GIF89a
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return "image/gif";
    }
    // BMP: 42 4D
    if bytes.starts_with(b"BM") {
        return "image/bmp";
    }
    // WebP: RIFF....WEBP
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        return "image/webp";
    }
    // PDF: %PDF
    if bytes.starts_with(b"%PDF") {
        return "application/pdf";
    }
    // ZIP: 50 4B 03 04
    if bytes.starts_with(b"PK\x03\x04") {
        // Check for Office document types in ZIP central directory
        if bytes.len() >= 50 {
            let central_dir = &bytes[30..bytes.len().min(50)];
            if central_dir.windows(6).any(|w| w == b"word/") {
                return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
            }
            if central_dir.windows(3).any(|w| w == b"xl/") {
                return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
            }
            if central_dir.windows(4).any(|w| w == b"ppt/") {
                return "application/vnd.openxmlformats-officedocument.presentationml.presentation";
            }
        }
        return "application/zip";
    }

    // Text-based formats - check first few bytes for ASCII patterns
    if bytes.len() >= 1 {
        match bytes[0] {
            b'<' => {
                if bytes.len() >= 4 && bytes.starts_with(b"<svg") {
                    return "image/svg+xml";
                }
                if bytes.len() >= 5 && bytes.starts_with(b"<?xml") {
                    return "application/xml";
                }
                return "application/xml";
            }
            b'{' | b'[' => return "application/json",
            _ => {}
        }
    }

    "application/octet-stream"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mime_type() {
        // Test empty data
        assert_eq!(detect_mime_type(&[]), "application/octet-stream");

        // Test PNG
        assert_eq!(detect_mime_type(b"\x89PNG\r\n\x1a\n"), "image/png");

        // Test JPEG
        assert_eq!(detect_mime_type(b"\xFF\xD8\xFF\xE0"), "image/jpeg");

        // Test GIF87a
        assert_eq!(detect_mime_type(b"GIF87a"), "image/gif");

        // Test GIF89a
        assert_eq!(detect_mime_type(b"GIF89a"), "image/gif");

        // Test BMP
        assert_eq!(detect_mime_type(b"BM\x00\x00"), "image/bmp");

        // Test PDF
        assert_eq!(detect_mime_type(b"%PDF-"), "application/pdf");

        // Test SVG
        assert_eq!(detect_mime_type(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"), "image/svg+xml");

        // Test XML (non-SVG)
        assert_eq!(detect_mime_type(b"<?xml version=\"1.0\"?><root><data>test</data></root>"), "application/xml");

        // Test JSON
        assert_eq!(detect_mime_type(b"{\"key\": \"value\"}"), "application/json");

        // Test ZIP
        assert_eq!(detect_mime_type(b"PK\x03\x04"), "application/zip");

        // Test unknown data
        assert_eq!(detect_mime_type(&[0x00, 0x01, 0x02, 0x03]), "application/octet-stream");
    }
}
