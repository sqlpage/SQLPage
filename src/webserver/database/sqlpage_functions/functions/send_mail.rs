use std::{borrow::Cow, fmt};

use anyhow::Context;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, CertificateStore, Tls, TlsParameters},
    },
};
use serde::Deserialize;

use crate::{
    app_config::{AppConfig, SmtpTlsMode},
    webserver::{
        database::blob_to_data_url::decode_data_uri_with_limit,
        http_client::native_certificate_der, http_request_info::RequestInfo,
    },
};

#[derive(Deserialize)]
#[serde(untagged)]
enum Recipients {
    One(String),
    Many(Vec<String>),
}

impl Recipients {
    fn parse(self, field: RecipientField) -> SendMailResult<Vec<Mailbox>> {
        let recipients = match self {
            Self::One(recipient) => vec![recipient],
            Self::Many(recipients) => recipients,
        };
        if recipients.is_empty() {
            return Err(field.invalid_address(None));
        }
        recipients
            .into_iter()
            .map(|recipient| {
                recipient
                    .parse::<Mailbox>()
                    .map_err(|_| field.invalid_address(Some(recipient)))
            })
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MailAttachment<'a> {
    #[serde(borrow)]
    filename: Cow<'a, str>,
    #[serde(borrow)]
    data_url: Cow<'a, str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MailRequest<'a> {
    to: Recipients,
    #[serde(default)]
    cc: Option<Recipients>,
    #[serde(borrow)]
    subject: Cow<'a, str>,
    #[serde(borrow, default)]
    body: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    body_html: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    body_md: Option<Cow<'a, str>>,
    #[serde(borrow, default, rename = "from")]
    from: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    reply_to: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    attachments: Vec<MailAttachment<'a>>,
}

#[derive(Debug, Clone, Copy)]
enum RecipientField {
    To,
    Cc,
}

impl RecipientField {
    fn invalid_address(self, address: Option<String>) -> SendMailError {
        match self {
            Self::To => SendMailError::InvalidEmailTo { address },
            Self::Cc => SendMailError::InvalidEmailCc { address },
        }
    }
}

#[derive(Debug)]
enum SendMailError {
    InvalidMessage(anyhow::Error),
    SmtpNotConfigured,
    MissingEmailFrom,
    InvalidEmailFrom { address: String },
    InvalidEmailTo { address: Option<String> },
    InvalidEmailCc { address: Option<String> },
    InvalidEmailReplyTo { address: String },
    InvalidAttachmentFilename { index: usize },
    InvalidAttachment {
        index: usize,
        reason: anyhow::Error,
    },
    SmtpTlsFailed(anyhow::Error),
    SmtpTimeout(anyhow::Error),
    SmtpRejected(anyhow::Error),
    SmtpConnectionFailed(anyhow::Error),
}

impl SendMailError {
    const fn code(&self) -> &'static str {
        match self {
            Self::InvalidMessage(_) => "INVALID_MESSAGE",
            Self::SmtpNotConfigured => "SMTP_NOT_CONFIGURED",
            Self::MissingEmailFrom => "MISSING_EMAIL_FROM",
            Self::InvalidEmailFrom { .. } => "INVALID_EMAIL_FROM",
            Self::InvalidEmailTo { .. } => "INVALID_EMAIL_TO",
            Self::InvalidEmailCc { .. } => "INVALID_EMAIL_CC",
            Self::InvalidEmailReplyTo { .. } => "INVALID_EMAIL_REPLY_TO",
            Self::InvalidAttachmentFilename { .. } | Self::InvalidAttachment { .. } => {
                "INVALID_ATTACHMENT"
            }
            Self::SmtpTlsFailed(_) => "SMTP_TLS_FAILED",
            Self::SmtpTimeout(_) => "SMTP_TIMEOUT",
            Self::SmtpRejected(_) => "SMTP_REJECTED",
            Self::SmtpConnectionFailed(_) => "SMTP_CONNECTION_FAILED",
        }
    }
}

impl fmt::Display for SendMailError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMessage(reason) => write_reason(formatter, "Invalid email message", reason),
            Self::SmtpNotConfigured => formatter.write_str(
                "sqlpage.send_mail() requires the smtp_host configuration option",
            ),
            Self::MissingEmailFrom => formatter.write_str(
                "Email has no from address; set its from property or configure smtp_from",
            ),
            Self::InvalidEmailFrom { address } => {
                write!(formatter, "'{address}' is not a valid from email address")
            }
            Self::InvalidEmailTo { address } => {
                write_invalid_recipient(formatter, "to", address.as_deref())
            }
            Self::InvalidEmailCc { address } => {
                write_invalid_recipient(formatter, "cc", address.as_deref())
            }
            Self::InvalidEmailReplyTo { address } => {
                write!(formatter, "'{address}' is not a valid reply_to email address")
            }
            Self::InvalidAttachmentFilename { index } => {
                write!(formatter, "Attachment filename at index {index} must not be empty")
            }
            Self::InvalidAttachment { index, reason } => {
                write_reason(formatter, &format!("Invalid attachment at index {index}"), reason)
            }
            Self::SmtpTlsFailed(reason) => {
                write_reason(formatter, "Unable to establish SMTP TLS", reason)
            }
            Self::SmtpTimeout(reason) => {
                write_reason(formatter, "SMTP operation timed out", reason)
            }
            Self::SmtpRejected(reason) => {
                write_reason(formatter, "SMTP server rejected the email", reason)
            }
            Self::SmtpConnectionFailed(reason) => {
                write_reason(formatter, "Unable to communicate with the SMTP server", reason)
            }
        }
    }
}

type SendMailResult<T> = Result<T, SendMailError>;

fn write_invalid_recipient(
    formatter: &mut fmt::Formatter<'_>,
    field: &str,
    address: Option<&str>,
) -> fmt::Result {
    match address {
        Some(address) => write!(formatter, "'{address}' is not a valid {field} email address"),
        None => write!(formatter, "{field} must contain at least one email address"),
    }
}

fn write_reason(
    formatter: &mut fmt::Formatter<'_>,
    context: &str,
    reason: &anyhow::Error,
) -> fmt::Result {
    write!(formatter, "{context}: ")?;
    if formatter.alternate() {
        write!(formatter, "{reason:#}")
    } else {
        fmt::Display::fmt(reason, formatter)
    }
}

/// Sends an email through the configured SMTP relay.
pub(super) async fn send_mail(
    request: &RequestInfo,
    mail_request: Option<Cow<'_, str>>,
) -> Option<String> {
    let mail_request = mail_request?;
    Some(send_mail_result_json(
        send_mail_with_config(&request.app_state.config, &mail_request).await,
    ))
}

fn send_mail_result_json(result: SendMailResult<()>) -> String {
    match result {
        Ok(()) => serde_json::json!({ "status": "accepted" }).to_string(),
        Err(error) => {
            let message = format!("{error:#}");
            log::warn!(
                "sqlpage.send_mail failed with {}: {message}",
                error.code()
            );
            serde_json::json!({
                "status": "error",
                "error_code": error.code(),
                "error": message,
            })
            .to_string()
        }
    }
}

async fn send_mail_with_config(config: &AppConfig, mail_request: &str) -> SendMailResult<()> {
    let host = config
        .smtp_host
        .as_deref()
        .ok_or(SendMailError::SmtpNotConfigured)?;
    let parsed: MailRequest<'_> = serde_json::from_str(mail_request)
        .context("sqlpage.send_mail() expects a valid JSON object")
        .map_err(SendMailError::InvalidMessage)?;

    let email = build_email(config, parsed)?;

    let tls = smtp_tls(config, host)?;
    let port = config
        .smtp_port
        .unwrap_or_else(|| config.smtp_tls_mode.default_port());
    let mut mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
        .port(port)
        .tls(tls);
    if let (Some(username), Some(password)) = (&config.smtp_username, &config.smtp_password) {
        mailer = mailer.credentials(Credentials::new(username.clone(), password.clone()));
    }

    let response = mailer.build().send(email).await.map_err(|error| {
        let is_timeout = error.is_timeout();
        let is_tls = error.is_tls();
        let is_rejected = error.is_transient() || error.is_permanent();
        let reason = anyhow::Error::new(error)
            .context(format!("Unable to send email through {host}:{port}"));
        if is_timeout {
            SendMailError::SmtpTimeout(reason)
        } else if is_tls {
            SendMailError::SmtpTlsFailed(reason)
        } else if is_rejected {
            SendMailError::SmtpRejected(reason)
        } else {
            SendMailError::SmtpConnectionFailed(reason)
        }
    })?;
    log::debug!("SMTP relay accepted email: {response:?}");
    Ok(())
}

/// Resolves the plain-text and optional HTML body alternatives from the request fields.
///
/// - At least one of `body` or `body_md` must be provided.
/// - `body_md` cannot be combined with `body_html`.
/// - When `body_md` is provided, it is rendered to HTML. The plain-text alternative is
///   `body` if present, otherwise the raw markdown source.
fn resolve_bodies(
    config: &AppConfig,
    body: Option<Cow<'_, str>>,
    body_html: Option<&Cow<'_, str>>,
    body_md: Option<&Cow<'_, str>>,
) -> SendMailResult<(String, Option<String>)> {
    if body.is_none() && body_md.is_none() {
        return Err(SendMailError::InvalidMessage(anyhow::anyhow!(
            "sqlpage.send_mail() requires either 'body' or 'body_md'"
        )));
    }
    if body_md.is_some() && body_html.is_some() {
        return Err(SendMailError::InvalidMessage(anyhow::anyhow!(
            "sqlpage.send_mail() cannot combine 'body_md' with 'body_html'"
        )));
    }

    let html_body = if let Some(markdown_src) = body_md {
        Some(
            crate::template_helpers::render_markdown_to_html(config, markdown_src)
                .map_err(|reason| {
                    SendMailError::InvalidMessage(anyhow::anyhow!(
                        "Failed to render body_md as HTML: {reason}"
                    ))
                })?,
        )
    } else {
        body_html.map(std::string::ToString::to_string)
    };

    // If body is provided it takes precedence; otherwise the raw markdown is used.
    let text_body = match body {
        Some(body) => body.into_owned(),
        None => body_md
            .map(std::string::ToString::to_string)
            .expect("body_md is present when body is None"),
    };
    Ok((text_body, html_body))
}

fn build_email(config: &AppConfig, request: MailRequest<'_>) -> SendMailResult<Message> {
    let MailRequest {
        to,
        cc,
        subject,
        body,
        body_html,
        body_md,
        from,
        reply_to,
        attachments,
    } = request;

    let (text_body, html_body) = resolve_bodies(config, body, body_html.as_ref(), body_md.as_ref())?;

    let sender = from
        .as_deref()
        .or(config.smtp_from.as_deref())
        .ok_or(SendMailError::MissingEmailFrom)?;
    let sender = sender
        .parse::<Mailbox>()
        .map_err(|_| SendMailError::InvalidEmailFrom {
            address: sender.to_string(),
        })?;
    let mut email = Message::builder()
        .from(sender)
        .subject(subject.as_ref());
    for recipient in to.parse(RecipientField::To)? {
        email = email.to(recipient);
    }
    if let Some(cc) = cc {
        for recipient in cc.parse(RecipientField::Cc)? {
            email = email.cc(recipient);
        }
    }
    if let Some(reply_to) = reply_to {
        let parsed_reply_to = reply_to.parse::<Mailbox>().map_err(|_| {
            SendMailError::InvalidEmailReplyTo {
                address: reply_to.to_string(),
            }
        })?;
        email = email.reply_to(parsed_reply_to);
    }
    if attachments.is_empty() {
        if let Some(html) = html_body {
            return email
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(text_body))
                        .singlepart(SinglePart::html(html)),
                )
                .context("Unable to build email message")
                .map_err(SendMailError::InvalidMessage);
        }
        return email
            .header(ContentType::TEXT_PLAIN)
            .body(text_body)
            .context("Unable to build email message")
            .map_err(SendMailError::InvalidMessage);
    }

    let mut remaining_attachment_size = config.max_email_attachment_size;
    let mut multipart = if let Some(html) = html_body {
        MultiPart::mixed().multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body))
                .singlepart(SinglePart::html(html)),
        )
    } else {
        MultiPart::mixed().singlepart(SinglePart::plain(text_body))
    };
    for (index, attachment) in attachments.into_iter().enumerate() {
        if attachment.filename.is_empty() {
            return Err(SendMailError::InvalidAttachmentFilename { index });
        }
        let (media_type, bytes) =
            decode_data_uri_with_limit(&attachment.data_url, remaining_attachment_size)
                .with_context(|| format!("Invalid attachment data_url at index {index}"))
                .map_err(|reason| SendMailError::InvalidAttachment { index, reason })?;
        remaining_attachment_size = remaining_attachment_size
            .checked_sub(bytes.len())
            .context("Attachments exceed max_email_attachment_size")
            .map_err(|reason| SendMailError::InvalidAttachment { index, reason })?;
        let media_type = if media_type.is_empty() {
            "application/octet-stream"
        } else {
            media_type
        };
        let content_type = ContentType::parse(media_type)
            .with_context(|| format!("Invalid attachment media type at index {index}"))
            .map_err(|reason| SendMailError::InvalidAttachment { index, reason })?;
        multipart = multipart.singlepart(Attachment::new(attachment.filename.into_owned()).body(
            bytes,
            content_type,
        ));
    }
    email
        .multipart(multipart)
        .context("Unable to build email message")
        .map_err(SendMailError::InvalidMessage)
}

fn smtp_tls(config: &AppConfig, host: &str) -> SendMailResult<Tls> {
    if config.smtp_tls_mode == SmtpTlsMode::None {
        return Ok(Tls::None);
    }

    let mut parameters = TlsParameters::builder(host.to_string());
    if config.system_root_ca_certificates {
        parameters = parameters.certificate_store(CertificateStore::None);
        for certificate in native_certificate_der().map_err(SendMailError::SmtpTlsFailed)? {
            parameters = parameters.add_root_certificate(
                Certificate::from_der(certificate.as_ref().to_vec())
                    .context("Unable to configure an SMTP root certificate")
                    .map_err(SendMailError::SmtpTlsFailed)?,
            );
        }
    } else {
        parameters = parameters.certificate_store(CertificateStore::WebpkiRoots);
    }
    let parameters = parameters
        .build_rustls()
        .context("Unable to configure SMTP TLS")
        .map_err(SendMailError::SmtpTlsFailed)?;
    Ok(match config.smtp_tls_mode {
        SmtpTlsMode::Starttls => Tls::Required(parameters),
        SmtpTlsMode::Tls => Tls::Wrapper(parameters),
        SmtpTlsMode::None => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    use super::{
        SendMailError, SmtpTlsMode, send_mail_result_json, send_mail_with_config,
    };
    use crate::app_config::tests::test_config;

    #[tokio::test]
    async fn sends_plain_text_email_to_configured_relay() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r#"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "SMTP test",
                "body": "hello smtp"
            }"#,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Subject: SMTP test"));
        assert!(data.contains("hello smtp"));
    }

    #[tokio::test]
    async fn sends_html_alternative_to_configured_relay() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r#"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "HTML test",
                "body": "hello plain",
                "body_html": "<p>hello <strong>html</strong></p>"
            }"#,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Subject: HTML test"));
        assert!(data.contains("Content-Type: multipart/alternative"));
        assert!(data.contains("Content-Type: text/plain"));
        assert!(data.contains("hello plain"));
        assert!(data.contains("Content-Type: text/html"));
        assert!(data.contains("<p>hello <strong>html</strong></p>"));
    }

    #[tokio::test]
    async fn sends_html_alternative_with_attachments() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r#"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "HTML and attachment test",
                "body": "hello plain",
                "body_html": "<p>hello <strong>html</strong></p>",
                "attachments": [
                    {"filename": "note.txt", "data_url": "data:text/plain;base64,aGk="}
                ]
            }"#,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Content-Type: multipart/mixed"));
        assert!(data.contains("Content-Type: multipart/alternative"));
        assert!(data.contains("Content-Type: text/plain"));
        assert!(data.contains("hello plain"));
        assert!(data.contains("Content-Type: text/html"));
        assert!(data.contains("<p>hello <strong>html</strong></p>"));
        assert!(data.contains("note.txt"));
    }

    #[tokio::test]
    async fn treats_null_body_html_as_omitted() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r#"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "null html test",
                "body": "hello plain",
                "body_html": null
            }"#,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Content-Type: text/plain"));
        assert!(!data.contains("text/html"));
    }

    #[tokio::test]
    async fn sends_markdown_alternative_to_configured_relay() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r##"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "Markdown test",
                "body_md": "# Hello\n\nThis is **bold**."
            }"##,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Subject: Markdown test"));
        assert!(data.contains("Content-Type: multipart/alternative"));
        assert!(data.contains("Content-Type: text/plain"));
        // The raw markdown is used as the plain-text alternative.
        assert!(data.contains("# Hello"));
        assert!(data.contains("This is **bold**."));
        assert!(data.contains("Content-Type: text/html"));
        assert!(data.contains("<h1>Hello</h1>"));
        assert!(data.contains("<strong>bold</strong>"));
    }

    #[tokio::test]
    async fn sends_markdown_alternative_with_attachments() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r##"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "Markdown and attachment test",
                "body_md": "# Hello",
                "attachments": [
                    {"filename": "note.txt", "data_url": "data:text/plain;base64,aGk="}
                ]
            }"##,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Content-Type: multipart/mixed"));
        assert!(data.contains("Content-Type: multipart/alternative"));
        assert!(data.contains("Content-Type: text/plain"));
        assert!(data.contains("# Hello"));
        assert!(data.contains("Content-Type: text/html"));
        assert!(data.contains("<h1>Hello</h1>"));
        assert!(data.contains("note.txt"));
    }

    #[tokio::test]
    async fn sends_markdown_with_explicit_body_as_text() {
        let (host, port, received) = start_smtp_server();
        let mut config = test_config();
        config.smtp_host = Some(host);
        config.smtp_port = Some(port);
        config.smtp_tls_mode = SmtpTlsMode::None;

        send_mail_with_config(
            &config,
            r##"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "Markdown with body test",
                "body": "plain text override",
                "body_md": "# Hello"
            }"##,
        )
        .await
        .unwrap();

        let data = received.recv().unwrap();
        assert!(data.contains("Content-Type: multipart/alternative"));
        assert!(data.contains("Content-Type: text/plain"));
        assert!(data.contains("plain text override"));
        assert!(!data.contains("# Hello"));
        assert!(data.contains("Content-Type: text/html"));
        assert!(data.contains("<h1>Hello</h1>"));
    }

    #[tokio::test]
    async fn rejects_body_md_combined_with_body_html() {
        let mut config = test_config();
        config.smtp_host = Some("localhost".to_string());
        let error = send_mail_with_config(
            &config,
            r##"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "conflict test",
                "body_md": "# Hello",
                "body_html": "<h1>Hello</h1>"
            }"##,
        )
        .await
        .unwrap_err();
        assert!(matches!(&error, SendMailError::InvalidMessage(_)));
        assert!(error.to_string().contains("cannot combine 'body_md' with 'body_html'"));
    }

    #[tokio::test]
    async fn rejects_missing_body_and_body_md() {
        let mut config = test_config();
        config.smtp_host = Some("localhost".to_string());
        let error = send_mail_with_config(
            &config,
            r#"{
                "to": "admin@example.com",
                "from": "contact@example.com",
                "subject": "missing body test"
            }"#,
        )
        .await
        .unwrap_err();
        assert!(matches!(&error, SendMailError::InvalidMessage(_)));
        assert!(error.to_string().contains("requires either 'body' or 'body_md'"));
    }

    #[tokio::test]
    async fn rejects_unknown_message_fields() {
        let mut config = test_config();
        config.smtp_host = Some("localhost".to_string());
        let error = send_mail_with_config(
            &config,
            r#"{"recipient":"admin@example.com","subject":"test","body":"hello"}"#,
        )
        .await
        .unwrap_err();
        assert!(matches!(&error, SendMailError::InvalidMessage(_)));
        assert!(error.to_string().contains("expects a valid JSON object"));
    }

    #[tokio::test]
    async fn reports_the_invalid_address_field() {
        let mut config = test_config();
        config.smtp_host = Some("localhost".to_string());
        let error = send_mail_with_config(
            &config,
            r#"{
                "to":"xxx",
                "from":"contact@example.com",
                "subject":"test",
                "body":"hello"
            }"#,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            &error,
            SendMailError::InvalidEmailTo {
                address: Some(address)
            } if address == "xxx"
        ));
        assert_eq!(error.to_string(), "'xxx' is not a valid to email address");
    }

    #[tokio::test]
    async fn rejects_combined_attachments_over_decoded_size_limit() {
        let mut config = test_config();
        config.smtp_host = Some("localhost".to_string());
        config.max_email_attachment_size = 4;
        let error = send_mail_with_config(
            &config,
            r#"{
                "to":"admin@example.com",
                "from":"contact@example.com",
                "subject":"test",
                "body":"hello",
                "attachments":[
                    {"filename":"one.txt","data_url":"data:text/plain;base64,YWJj"},
                    {"filename":"two.txt","data_url":"data:text/plain;base64,ZGVm"}
                ]
            }"#,
        )
        .await
        .unwrap_err();
        assert!(matches!(
            &error,
            SendMailError::InvalidAttachment { .. }
        ));
        assert!(format!("{error:#}").contains("Decoded data exceeds the limit of 1 bytes"));
    }

    #[test]
    fn serializes_success_and_errors_as_json() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&send_mail_result_json(Ok(()))).unwrap(),
            serde_json::json!({ "status": "accepted" })
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&send_mail_result_json(Err(
                SendMailError::SmtpConnectionFailed(anyhow::anyhow!("SMTP failed"))
            )))
            .unwrap(),
            serde_json::json!({
                "status": "error",
                "error_code": "SMTP_CONNECTION_FAILED",
                "error": "Unable to communicate with the SMTP server: SMTP failed",
            })
        );
    }

    fn start_smtp_server() -> (String, u16, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            sender.send(handle_smtp_connection(stream)).unwrap();
        });
        (address.ip().to_string(), address.port(), receiver)
    }

    fn handle_smtp_connection(mut stream: TcpStream) -> String {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        write_response(&mut stream, "220 localhost ESMTP test server");
        let mut data = String::new();
        loop {
            let line = read_line(&mut reader);
            let command = line.trim_end_matches(['\r', '\n']);
            if command.starts_with("EHLO") || command.starts_with("HELO") {
                write_response(&mut stream, "250 localhost");
            } else if command == "DATA" {
                write_response(&mut stream, "354 End data with <CR><LF>.<CR><LF>");
                loop {
                    let line = read_line(&mut reader);
                    if line == ".\r\n" || line == ".\n" {
                        break;
                    }
                    data.push_str(&line);
                }
                write_response(&mut stream, "250 Message accepted");
            } else if command == "QUIT" {
                write_response(&mut stream, "221 Bye");
                break;
            } else {
                write_response(&mut stream, "250 OK");
            }
        }
        data
    }

    fn read_line(reader: &mut BufReader<TcpStream>) -> String {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        line
    }

    fn write_response(stream: &mut TcpStream, response: &str) {
        write!(stream, "{response}\r\n").unwrap();
    }
}
