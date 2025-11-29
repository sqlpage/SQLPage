use super::RequestInfo;
use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use std::borrow::Cow;
use std::time::Duration;

pub(super) async fn upload_to_s3<'a>(
    request: &'a RequestInfo,
    bucket: Option<Cow<'a, str>>,
    data: Cow<'a, str>,
    key: Cow<'a, str>,
) -> anyhow::Result<String> {
    let config = &request.app_state.config;
    let bucket = bucket
        .as_deref()
        .or(config.s3_bucket.as_deref())
        .ok_or_else(|| anyhow::anyhow!("S3 bucket not configured"))?;

    let client = get_s3_client(config).await;

    let body_bytes = if let Some(stripped) = data.strip_prefix("file://") {
        let file_path = std::path::Path::new(stripped);
        // Security check: ensure the file is within the web root or allowed paths
        let web_root = &config.web_root;
        let full_path = web_root.join(file_path);
        if !full_path.starts_with(web_root) {
            anyhow::bail!("Security violation: Access denied to file outside web root");
        }
        tokio::fs::read(&full_path)
            .await
            .map_err(|e| {
                log::error!("Failed to read file {}: {}", full_path.display(), e);
                e
            })
            .with_context(|| format!("Unable to read file {}", full_path.display()))?
    } else {
        // Assume base64
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(data.as_bytes())
            .map_err(|e| {
                log::error!("Base64 decode failed: {e}");
                e
            })
            .context("Invalid base64 data")?
    };

    client
        .put_object()
        .bucket(bucket)
        .key(key.as_ref())
        .body(body_bytes.into())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload to S3: {e}"))?;

    Ok(format!("s3://{bucket}/{key}"))
}

pub(super) async fn get_from_s3<'a>(
    request: &'a RequestInfo,
    bucket: Option<Cow<'a, str>>,
    key: Cow<'a, str>,
) -> anyhow::Result<String> {
    let config = &request.app_state.config;
    let bucket = bucket
        .as_deref()
        .or(config.s3_bucket.as_deref())
        .ok_or_else(|| anyhow::anyhow!("S3 bucket not configured"))?;

    let client = get_s3_client(config).await;

    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(3600))?;

    let presigned_request = client
        .get_object()
        .bucket(bucket)
        .key(key.as_ref())
        .presigned(presigning_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate presigned URL: {e}"))?;

    Ok(presigned_request.uri().to_string())
}

async fn get_s3_client(config: &crate::app_config::AppConfig) -> aws_sdk_s3::Client {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());

    if let Some(endpoint) = &config.s3_endpoint {
        loader = loader.endpoint_url(endpoint);
    }
    if let Some(region) = &config.s3_region {
        loader = loader.region(aws_config::Region::new(region.clone()));
    }
    if let (Some(access_key), Some(secret_key)) = (&config.s3_access_key, &config.s3_secret_key) {
        let creds = aws_sdk_s3::config::Credentials::new(
            access_key.clone(),
            secret_key.clone(),
            None,
            None,
            "sqlpage-config",
        );
        loader = loader.credentials_provider(creds);
    }

    let sdk_config = loader.load().await;
    aws_sdk_s3::Client::new(&sdk_config)
}
