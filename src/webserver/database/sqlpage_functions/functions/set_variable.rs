use super::*;

pub(super) async fn set_variable<'a>(
    context: &'a ExecutionContext,
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    let mut params = URLParameters::new();

    for (k, v) in &context.url_params {
        if k == &name {
            continue;
        }
        params.push_single_or_vec(k, v.clone());
    }

    if let Some(value) = value {
        params.push_single_or_vec(&name, SingleOrVec::Single(value.into_owned()));
    }

    Ok(params.with_empty_path())
}
