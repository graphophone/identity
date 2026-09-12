pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let password_hash = bcrypt::hash_bytes(password, bcrypt::DEFAULT_COST)?;
    let password_hash = str::from_utf8(&password_hash)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, password_hash)
}