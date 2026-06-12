use super::*;

/// escapes a string for use in a URL using percent encoding
/// for example, spaces are replaced with %20, '/' with %2F, etc.
/// This is useful for constructing URLs in SQL queries.
/// If this function is passed a NULL value, it will return NULL (None in Rust),
/// rather than an empty string or an error.
pub(super) async fn url_encode(raw_text: Option<Cow<'_, str>>) -> Option<Cow<'_, str>> {
    Some(match raw_text? {
        Cow::Borrowed(inner) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            encoded.into()
        }
        Cow::Owned(inner) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            Cow::Owned(encoded.collect())
        }
    })
}
