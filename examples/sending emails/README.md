# Sending Emails with SQLPage

This example sends plain-text email with [`sqlpage.send_mail`](https://sql-page.com/functions.sql?function=send_mail). The included Docker Compose setup uses [Mailpit](https://mailpit.axllent.org/) as a local SMTP server, so no email leaves your computer.

Run the example:

```sh
docker compose up --build
```

This builds SQLPage from the current repository checkout before starting the example.

Open http://localhost:8080 and choose one of two flows, then inspect the message in the Mailpit inbox at http://localhost:8025:

- **Simple email** sends to one recipient with the `SMTP_FROM` sender configured in Docker Compose.
- **Advanced email** demonstrates multiple recipients, Cc, Reply-To, a per-message sender override, an HTML alternative body, and an uploaded attachment.

The SMTP server is configured in [`docker-compose.yml`](./docker-compose.yml) with `SMTP_HOST=mailpit`, `SMTP_PORT=1025`, `SMTP_TLS_MODE=none`, and a default `SMTP_FROM`. Plaintext mode is intended only for trusted local SMTP servers such as Mailpit.

For a remote SMTP relay, keep the default `SMTP_TLS_MODE=starttls`, or set it to `tls` when the relay requires implicit TLS. Configure `SMTP_USERNAME` and `SMTP_PASSWORD` when authentication is required; SQLPage rejects credentials in plaintext mode.

The simple handler omits `from`, so SQLPage uses the configured `SMTP_FROM`:

```sql
set message = json_object(
    'to', :recipient,
    'subject', :subject,
    'body', :body
);
set result = sqlpage.send_mail($message);
```

The advanced handler turns the temporary uploaded file into the data URL expected by an attachment:

```sql
set attachment_path = sqlpage.uploaded_file_path('attachment');
set attachment_data_url = sqlpage.read_file_as_data_url($attachment_path);
```

`sqlpage.send_mail` returns `{"status":"accepted"}` after the SMTP relay accepts the message. If validation, connection, authentication, or submission fails, it returns `{"status":"error","error_code":"...","error":"..."}` instead of stopping the request. Check `status` before reporting success. Passing SQL `NULL` returns SQL `NULL` without sending or logging a warning.

Do not expose unrestricted forms like these publicly. In production, authenticate users, restrict recipients, validate input and uploaded files, and add rate limiting to prevent abuse.
