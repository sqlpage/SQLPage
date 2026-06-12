use super::*;

/// Returns the value of an environment variable.
pub(super) async fn environment_variable(name: Cow<'_, str>) -> anyhow::Result<Option<Cow<'_, str>>> {
    match std::env::var(&*name) {
        Ok(value) => Ok(Some(Cow::Owned(value))),
        Err(std::env::VarError::NotPresent) if name.contains(['=', '\0']) => anyhow::bail!(
            "Invalid environment variable name: {name:?}. Environment variable names cannot contain an equals sign or a null character."
        ),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("unable to read the environment variable {name:?}"))
        }
    }
}
