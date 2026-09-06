/// Detects MIME type based on file signatures (magic bytes).
/// Returns the most appropriate MIME type for common file formats.
#[must_use]
pub fn detect_mime_type(bytes: &[u8]) -> &'static str {
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
            if central_dir.windows(5).any(|w| w == b"word/") {
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

    if bytes.starts_with(b"<?xml") {
        return "application/xml";
    }
    if bytes.starts_with(b"<svg") || bytes.starts_with(b"<!DOCTYPE svg") {
        return "image/svg+xml";
    }
    if bytes.starts_with(b"{") || bytes.starts_with(b"[") {
        return "application/json";
    }

    "application/octet-stream"
}

/// Converts binary data to a data URL string.
/// This function is used by both SQL type conversion and file reading functions.
/// Automatically detects common file types based on magic bytes.
#[must_use]
pub fn vec_to_data_uri(bytes: &[u8]) -> String {
    let mime_type = detect_mime_type(bytes);
    vec_to_data_uri_with_mime(bytes, mime_type)
}

/// Converts binary data to a data URL string with a specific MIME type.
/// This function is used by both SQL type conversion and file reading functions.
#[must_use]
pub fn vec_to_data_uri_with_mime(bytes: &[u8], mime_type: &str) -> String {
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
#[must_use]
pub fn vec_to_data_uri_value(bytes: &[u8]) -> serde_json::Value {
    serde_json::Value::String(vec_to_data_uri(bytes))
}

/// Decodes a data URL into its declared media type and raw bytes.
pub fn decode_data_uri(data_url: &str) -> anyhow::Result<(&str, Vec<u8>)> {
    decode_data_uri_with_limit(data_url, usize::MAX)
}

/// Decodes a data URL while limiting the size of the decoded bytes.
pub fn decode_data_uri_with_limit(
    data_url: &str,
    max_decoded_size: usize,
) -> anyhow::Result<(&str, Vec<u8>)> {
    use anyhow::Context as _;

    let rest = data_url
        .strip_prefix("data:")
        .context("Invalid data URL: missing 'data:' prefix")?;
    let (mut media_type, data) = rest
        .split_once(',')
        .context("Invalid data URL: missing comma")?;
    let is_base64 = media_type.ends_with(";base64");
    let max_percent_decoded_size = if is_base64 {
        max_decoded_size
            .saturating_add(2)
            .saturating_div(3)
            .saturating_mul(4)
    } else {
        max_decoded_size
    };
    let percent_decoded = percent_encoding::percent_decode(data.as_bytes())
        .take(max_percent_decoded_size.saturating_add(1))
        .collect::<Vec<_>>();
    anyhow::ensure!(
        percent_decoded.len() <= max_percent_decoded_size,
        "Decoded data exceeds the limit of {max_decoded_size} bytes"
    );
    let bytes = if let Some(stripped) = media_type.strip_suffix(";base64") {
        media_type = stripped;
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, percent_decoded)
            .context("Invalid base64 data in data URL")?
    } else {
        percent_decoded
    };
    anyhow::ensure!(
        bytes.len() <= max_decoded_size,
        "Decoded data exceeds the limit of {max_decoded_size} bytes"
    );
    Ok((media_type, bytes))
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

    fn zip_starting_with(first_entry_name: &[u8]) -> Vec<u8> {
        let mut blob = b"PK\x03\x04".to_vec();
        blob.resize(30, 0);
        blob.extend_from_slice(first_entry_name);
        blob.resize(50, 0);
        blob
    }

    #[test]
    fn test_detect_office_documents() {
        assert_eq!(
            detect_mime_type(&zip_starting_with(b"word/document.xml")),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
        assert_eq!(
            detect_mime_type(&zip_starting_with(b"xl/workbook.xml")),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
        assert_eq!(
            detect_mime_type(&zip_starting_with(b"ppt/presentation.xml")),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );
        assert_eq!(
            detect_mime_type(&zip_starting_with(b"other/thing.txt")),
            "application/zip"
        );
    }

    #[test]
    fn decodes_base64_and_percent_encoded_data_urls() {
        assert_eq!(
            decode_data_uri("data:text/plain;base64,aGVsbG8=").unwrap(),
            ("text/plain", b"hello".to_vec())
        );
        assert_eq!(
            decode_data_uri("data:text/plain,hello%20world").unwrap(),
            ("text/plain", b"hello world".to_vec())
        );
    }

    #[test]
    fn limits_decoded_data_url_size_before_decoding() {
        assert_eq!(
            decode_data_uri_with_limit("data:text/plain;base64,aGVsbG8=", 5).unwrap(),
            ("text/plain", b"hello".to_vec())
        );
        assert!(decode_data_uri_with_limit("data:text/plain;base64,aGVsbG8=", 4).is_err());
        assert!(decode_data_uri_with_limit("data:text/plain,hello%20world", 10).is_err());
    }

    #[test]
    fn rejects_invalid_data_urls() {
        assert!(decode_data_uri("text/plain,hello").is_err());
        assert!(decode_data_uri("data:text/plain").is_err());
        assert!(decode_data_uri("data:;base64,not base64").is_err());
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
            serde_json::Value::String(s) => {
                assert_eq!(s, "data:application/octet-stream;base64,dGVzdA==");
            }
            _ => panic!("Expected String value"),
        }
    }
}
