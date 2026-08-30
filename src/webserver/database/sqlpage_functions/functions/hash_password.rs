use anyhow::anyhow;

pub(super) async fn hash_password(password: Option<String>) -> anyhow::Result<Option<String>> {
    let Some(password) = password else {
        return Ok(None);
    };
    actix_web::rt::task::spawn_blocking(move || {
        use argon2::password_hash::PasswordHasher;

        // Hashes a password using Argon2. This is a CPU-intensive blocking operation.
        let phf = argon2::Argon2::default();
        let password_hash = phf
            .hash_password(password.as_bytes())
            .map_err(|e| anyhow!("Unable to hash password: {e}"))?;
        Ok(password_hash.to_string())
    })
    .await?
    .map(Some)
}

#[tokio::test]
pub(super) async fn test_hash_password() {
    let s = hash_password(Some("password".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert!(s.starts_with("$argon2"));
}
