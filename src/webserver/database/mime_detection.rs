/// Detects MIME type based on file signatures (magic bytes).
/// Returns the most appropriate MIME type for common file formats.
#[must_use] pub fn detect_mime_type(bytes: &[u8]) -> &'static str {
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
    if !bytes.is_empty() {
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

/// Converts binary data to a data URL string.
/// This function is used by both SQL type conversion and file reading functions.
/// Automatically detects common file types based on magic bytes.
#[must_use] pub fn vec_to_data_uri(bytes: &[u8]) -> String {
    let mime_type = detect_mime_type(bytes);
    vec_to_data_uri_with_mime(bytes, mime_type)
}

/// Converts binary data to a data URL string with a specific MIME type.
/// This function is used by both SQL type conversion and file reading functions.
#[must_use] pub fn vec_to_data_uri_with_mime(bytes: &[u8], mime_type: &str) -> String {
    let mut data_url = format!("data:{mime_type};base64,");
    base64::Engine::encode_string(
        &base64::engine::general_purpose::STANDARD,
        bytes,
        &mut data_url,
    );
    data_url
}

/// Converts binary data to a data URL JSON value.
/// This is a convenience function for SQL type conversion.
#[must_use] pub fn vec_to_data_uri_value(bytes: &[u8]) -> serde_json::Value {
    serde_json::Value::String(vec_to_data_uri(bytes))
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
        assert_eq!(
            detect_mime_type(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"),
            "image/svg+xml"
        );

        // Test XML (non-SVG)
        assert_eq!(
            detect_mime_type(b"<?xml version=\"1.0\"?><root><data>test</data></root>"),
            "application/xml"
        );

        // Test JSON
        assert_eq!(
            detect_mime_type(b"{\"key\": \"value\"}"),
            "application/json"
        );

        // Test ZIP
        assert_eq!(detect_mime_type(b"PK\x03\x04"), "application/zip");

        // Test unknown data
        assert_eq!(
            detect_mime_type(&[0x00, 0x01, 0x02, 0x03]),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_vec_to_data_uri() {
        // Test with empty bytes
        let result = vec_to_data_uri(&[]);
        assert_eq!(result, "data:application/octet-stream;base64,");

        // Test with simple text
        let result = vec_to_data_uri(b"Hello World");
        assert_eq!(
            result,
            "data:application/octet-stream;base64,SGVsbG8gV29ybGQ="
        );

        // Test with binary data
        let binary_data = [0, 1, 2, 255, 254, 253];
        let result = vec_to_data_uri(&binary_data);
        assert_eq!(result, "data:application/octet-stream;base64,AAEC//79");
    }

    #[test]
    fn test_vec_to_data_uri_with_mime() {
        // Test with custom MIME type
        let result = vec_to_data_uri_with_mime(b"Hello", "text/plain");
        assert_eq!(result, "data:text/plain;base64,SGVsbG8=");

        // Test with image MIME type
        let result = vec_to_data_uri_with_mime(&[255, 216, 255], "image/jpeg");
        assert_eq!(result, "data:image/jpeg;base64,/9j/");

        // Test with empty bytes and custom MIME
        let result = vec_to_data_uri_with_mime(&[], "application/json");
        assert_eq!(result, "data:application/json;base64,");
    }

    #[test]
    fn test_vec_to_data_uri_value() {
        // Test that it returns a JSON string value
        let result = vec_to_data_uri_value(b"test");
        match result {
            serde_json::Value::String(s) => assert_eq!(s, "data:application/octet-stream;base64,dGVzdA=="),
            _ => panic!("Expected String value"),
        }
    }
}
