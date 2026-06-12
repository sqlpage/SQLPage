use super::*;

/// Computes the HMAC (Hash-based Message Authentication Code) of the input data
/// using the specified key and hashing algorithm.
pub(super) async fn hmac<'a>(
    data: Cow<'a, str>,
    key: Cow<'a, str>,
    algorithm: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    use ::hmac::{Hmac, KeyInit, Mac};
    use sha2::{Sha256, Sha512};

    let algorithm = algorithm.as_deref().unwrap_or("sha256");

    // Parse algorithm and output format (e.g., "sha256" or "sha256-base64")
    let (hash_algo, output_format) = if let Some((algo, format)) = algorithm.split_once('-') {
        (algo, format)
    } else {
        (algorithm, "hex")
    };

    let result = match hash_algo.to_lowercase().as_str() {
        "sha256" => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                .map_err(|e| anyhow!("Invalid HMAC key: {e}"))?;
            mac.update(data.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        "sha512" => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                .map_err(|e| anyhow!("Invalid HMAC key: {e}"))?;
            mac.update(data.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        _ => {
            anyhow::bail!(
                "Unsupported HMAC algorithm: {hash_algo}. Supported algorithms: sha256, sha512"
            )
        }
    };

    // Convert to requested output format
    let output = match output_format.to_lowercase().as_str() {
        "hex" => result.into_iter().fold(String::new(), |mut acc, byte| {
            write!(&mut acc, "{byte:02x}").unwrap();
            acc
        }),
        "base64" => base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result),
        _ => {
            anyhow::bail!(
                "Unsupported output format: {output_format}. Supported formats: hex, base64"
            )
        }
    };

    Ok(Some(output))
}

#[tokio::test]
pub(super) async fn test_hmac() {
    // Test vector from RFC 4231 - HMAC-SHA256
    let result = hmac(
        Cow::Borrowed("The quick brown fox jumps over the lazy dog"),
        Cow::Borrowed("key"),
        Some(Cow::Borrowed("sha256")),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        result,
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
}
