INSERT INTO sqlpage_functions (
        "name",
        "return_type",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'send_mail',
        'JSON',
        '0.45.0',
        'mail',
        'Sends a plain-text email using the outgoing mail server configured in SQLPage.

### Quick start

You need an [SMTP server](https://en.wikipedia.org/wiki/Simple_Mail_Transfer_Protocol), which is the outgoing mail server provided by an email account or email delivery service.

Add its connection details to `sqlpage/sqlpage.json`:

```json
{
  "smtp_host": "smtp.example.com",
  "smtp_username": "your-smtp-user",
  "smtp_password": "your-smtp-password",
  "smtp_from": "My application <notifications@example.com>"
}
```

The default connection uses STARTTLS on port 587, which is the most common setup. Restart SQLPage after changing its configuration.

**Important:** SQLPage can sign in with an SMTP username and password, but it does not support OAuth. If the provider instructions only offer OAuth or "Modern Auth", use a different SMTP relay.

You can now send an email from any SQL file:

```sql
set result = sqlpage.send_mail(json_object(
    ''to'', ''alice@example.com'',
    ''subject'', ''Hello from SQLPage'',
    ''body'', ''Your first email is working!''
));
```

The sender comes from `smtp_from`. The result is a JSON object:

```json
{"status":"accepted"}
```

If the message cannot be sent, the function returns the reason instead of stopping the request:

```json
{"status":"error","error_code":"INVALID_EMAIL_TO","error":"''xxx'' is not a valid to email address"}
```

For every non-`NULL` call, `status` is either `accepted` or `error`. Always check it before showing a success message or continuing work that depends on the email:

```sql
select ''alert'' as component,
    case when json_extract($result, ''$.status'') = ''accepted'' then ''success'' else ''danger'' end as color,
    case when json_extract($result, ''$.status'') = ''accepted'' then ''Email sent'' else ''Email could not be sent'' end as title,
    json_extract($result, ''$.error'') as description;
```

### Where to find the SMTP settings

Search the help pages or administration panel of the service that sends email for you. Look for **SMTP**, **outgoing mail server**, **SMTP submission**, **SMTP relay**, or **send from an app or device**.

Provider documentation may use different names for the same settings:

| Provider documentation | SQLPage setting |
| --- | --- |
| SMTP server, outgoing server, relay, or smart host | `smtp_host` |
| Port | `smtp_port` |
| STARTTLS, SSL/TLS, or connection security | `smtp_tls_mode` |
| SMTP username | `smtp_username` |
| SMTP password, app password, token, or API key | `smtp_password` |
| Sender or From address | `smtp_from` |

The SMTP password is often a separate app password, token, or SMTP credential rather than the password used to open webmail. Use exactly what the provider instructions specify.

For an existing mailbox, these official guides explain the available options:

- [Personal Google Account app passwords](https://support.google.com/accounts/answer/185833), for eligible accounts
- [Google Workspace: send email from a printer, scanner, or app](https://knowledge.workspace.google.com/admin/gmail/send-email-from-a-printer-scanner-or-app)
- [Microsoft 365: send email from a device or application](https://learn.microsoft.com/en-us/exchange/mail-flow-best-practices/how-to-set-up-a-multifunction-device-or-application-to-send-email-using-microsoft-365-or-office-365)

Personal Outlook.com SMTP requires OAuth and is therefore not currently compatible. Microsoft 365 administrators can use the relay options described in the linked organization guide.

Dedicated email delivery services also provide SMTP settings. Here are examples in alphabetical order:

- [Amazon SES SMTP credentials](https://docs.aws.amazon.com/ses/latest/dg/smtp-credentials.html)
- [Mailgun SMTP](https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-smtp)
- [Postmark SMTP](https://postmarkapp.com/developer/user-guide/send-email-with-smtp)
- [Resend SMTP](https://resend.com/docs/send-with-smtp)
- [Twilio SendGrid SMTP](https://www.twilio.com/docs/sendgrid/for-developers/sending-email/integrating-with-the-smtp-api)

These links are examples, not endorsements. SQLPage is not affiliated with any of these services. Compare their requirements, limits, and pricing for your own use case.

For local development, [Mailpit](https://mailpit.axllent.org/) accepts messages and displays them in a browser without delivering them to real recipients. The [SQLPage email example](https://github.com/sqlpage/SQLPage/tree/main/examples/sending%20emails) includes a ready-to-run Mailpit setup.

All SMTP options can also be set with uppercase environment variables such as `SMTP_HOST` and `SMTP_PASSWORD`. See the complete [SQLPage configuration reference](https://github.com/sqlpage/SQLPage/blob/main/configuration.md). Do not commit SMTP credentials to source control.

### Message fields

The function takes one JSON object with three required fields:

- `to`: the recipient email address;
- `subject`: the email subject;
- `body`: the plain-text email body.

It also accepts:

- `from`: overrides `smtp_from` for this message;
- `reply_to`: the address that receives replies;
- `cc`: a recipient who receives a visible copy;
- `body_html`: an HTML version of the body;
- `attachments`: files to include with the message.

`to` and `cc` can each be either one address or an array of addresses. Addresses can include a display name, for example `"Jane Doe <jane@example.com>"`.

Most SMTP servers only allow approved sender addresses. Prefer a fixed `smtp_from`. Override `from` only when the SMTP provider allows the address.

### Multiple recipients and attachments

Each attachment has a file name and a [data URL](https://developer.mozilla.org/en-US/docs/Web/URI/Schemes/data) containing its data. This example attaches a file from the SQLPage server:

```sql
set result = sqlpage.send_mail(json_object(
    ''to'', json_array(''alice@example.com'', ''bob@example.com''),
    ''cc'', ''team@example.com'',
    ''subject'', ''Monthly report'',
    ''body'', ''The report is attached.'',
    ''attachments'', json_array(json_object(
        ''filename'', ''report.pdf'',
        ''data_url'', sqlpage.read_file_as_data_url(''report.pdf'')
    ))
));
```

[`sqlpage.read_file_as_data_url`](/functions.sql?function=read_file_as_data_url) is one way to create attachment data. Data URLs can also come from an uploaded file, a database value, an HTTP response, or SQL.

The combined decoded size of all attachments is limited by `max_email_attachment_size`, which defaults to 10 MiB. This is separate from `max_uploaded_file_size` because attachments do not have to come from form uploads.

### HTML email

Set `body_html` to send an HTML version of the message alongside the plain-text `body`. The message is sent as a `multipart/alternative`: mail clients that prefer HTML show the HTML body, and clients that prefer text show the plain-text body.

```sql
set result = sqlpage.send_mail(json_object(
    ''to'', ''alice@example.com'',
    ''subject'', ''Welcome'',
    ''body'', ''Welcome to our service.'',
    ''body_html'', ''<p>Welcome to <strong>our service</strong>.</p>''
));
```

`body` is always required. Include a meaningful plain-text alternative for deliverability and accessibility. SQLPage does not sanitize `body_html`: the SQL author is responsible for the HTML content. Email clients ignore scripts, and styles are often stripped or sandboxed.

### Contact form

```sql
select ''form'' as component, ''post'' as method;
select ''email'' as name, ''email'' as type, true as required;
select ''message'' as name, ''textarea'' as type, true as required;

set mail = json_object(
    ''to'', ''admin@example.com'',
    ''reply_to'', :email,
    ''subject'', ''Website contact form'',
    ''body'', :message
);
set result = (
    select sqlpage.send_mail($mail)
    where :message is not null
);

select ''alert'' as component,
    case when json_extract($result, ''$.status'') = ''accepted'' then ''success'' else ''danger'' end as color,
    case when json_extract($result, ''$.status'') = ''accepted'' then ''Message sent'' else ''Message could not be sent'' end as title,
    json_extract($result, ''$.error'') as description
where :message is not null;
```

On the initial page load, `:message` is `NULL`, so the query returns no row and no email is sent. After submission, `:email` and `:message` contain the form fields.

For a public form, keep `to` fixed in SQL so visitors cannot use your server to email arbitrary recipients. Validate inputs and add suitable rate limiting and anti-abuse controls.

### Before using this in production

- A `status` of `accepted` is not proof of delivery. A message can still bounce or be filtered later. Check the provider logs or delivery webhooks when delivery status matters.
- The provider may require sender or domain verification and DNS records such as SPF, DKIM, or DMARC. Configure these with the provider and DNS host.
- The function waits for the SMTP server during the web request. It opens a new connection for each call and does not retry automatically or save failed messages in a queue.
- SMTP commands use a fixed 60-second timeout. SQLPage does not currently provide a setting to change it.
- A selected call runs once for every row returned by its query. If the query returns no rows, it sends no email. Avoid using it over many rows; use a background queue or provider bulk API for bulk sending.

### Connection options

`smtp_host` must contain only a host name or IP address. Do not include `smtp://`, `https://`, a path, or a port.

Choose `smtp_tls_mode` according to the provider instructions:

- `starttls` (default) requires a STARTTLS upgrade before authentication or message submission. Its default port is 587. SQLPage fails rather than continuing without encryption when STARTTLS is unavailable.
- `tls` encrypts the connection from the beginning. Its default port is 465. Providers may call this implicit TLS, SSL/TLS, or SMTPS.
- `none` sends the message without encryption. Its default port is 25. It cannot be used with a username and password and is intended only for a trusted local server such as Mailpit.

Set `smtp_port` when the provider specifies another port, such as 2525. `smtp_username` and `smtp_password` must either both be configured or both be omitted.

SQLPage normally validates TLS certificates using public web PKI roots. Enable `system_root_ca_certificates`, or set `SSL_CERT_FILE` or `SSL_CERT_DIR`, to use roots installed by the system administrator, including private roots.

### Supported and unsupported features

SQLPage supports unauthenticated SMTP servers and username/password authentication using the SMTP `PLAIN` and `LOGIN` mechanisms. Credentials are allowed only over an encrypted connection.

SQLPage does not currently support:

- OAuth or XOAUTH2 authentication. If a provider only allows OAuth, it is not compatible with this function;
- CRAM-MD5, DIGEST-MD5, client-certificate authentication, or a per-server custom CA file;
- opportunistic STARTTLS, direct delivery to recipient mail servers, or receiving email;
- BCC, multiple reply-to addresses, or custom email headers;
- provider-specific headers for templates, tags, tracking, scheduling, idempotency, or metadata;
- DKIM signing inside SQLPage, S/MIME, or end-to-end encryption. The SMTP provider may add DKIM signatures;
- connection pooling, automatic retries, a persistent queue, scheduled sending, or a bulk-send API;
- a configurable SMTP command timeout or EHLO client identity;
- delivery receipts, bounce processing, suppression lists, open or click tracking, or webhooks.

### `NULL`, empty values, and invalid input

- SQL `NULL` is passed through: `sqlpage.send_mail(NULL)` returns SQL `NULL`, sends nothing, and does not log a warning.
- A JSON value other than an object and unknown or invalid message fields produce a JSON result with `status`, `error_code`, and `error` fields.
- `to`, `subject`, and `body` are required and cannot be JSON `null`.
- `from`, `reply_to`, `cc`, and `body_html` treat JSON `null` like an omitted field. If `from` is omitted, `smtp_from` must be configured. When `body_html` is omitted, the message is plain text only.
- `attachments` can be omitted or an empty array. JSON `null` is not accepted for `attachments`.
- Empty recipient arrays, JSON `null` inside recipient arrays, invalid addresses, an empty attachment file name, invalid data URLs, and unknown attachment fields produce an error result with `status`, `error_code`, and `error`.
- Empty strings are allowed for `subject` and `body`, although an SMTP server may reject them.

### Error codes

`error_code` is a stable, machine-readable value. `error` is the corresponding human-readable detail.

| `error_code` | Meaning |
| --- | --- |
| `INVALID_MESSAGE` | The argument is not a valid message object, a required field is missing, or the message cannot be constructed. |
| `SMTP_NOT_CONFIGURED` | `smtp_host` is not configured. |
| `MISSING_EMAIL_FROM` | Neither the message nor `smtp_from` provides a sender. |
| `INVALID_EMAIL_FROM` | `from` is not a valid email address. |
| `INVALID_EMAIL_TO` | `to` is empty or contains an invalid email address. |
| `INVALID_EMAIL_CC` | `cc` is empty or contains an invalid email address. |
| `INVALID_EMAIL_REPLY_TO` | `reply_to` is not a valid email address. |
| `INVALID_ATTACHMENT` | An attachment has an invalid name, data URL, media type, or exceeds the configured size limit. |
| `SMTP_TLS_FAILED` | TLS certificates could not be configured or the encrypted connection failed. |
| `SMTP_TIMEOUT` | The SMTP operation timed out. |
| `SMTP_REJECTED` | The SMTP server returned a temporary or permanent rejection, including authentication failures. |
| `SMTP_CONNECTION_FAILED` | SQLPage could not connect to or communicate with the SMTP server. |
'
    );

INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'send_mail',
        1,
        'message',
        'A JSON object containing the email to send. Required properties are `to` (an address or non-empty address array), `subject`, and `body`. Optional properties are `from` (required unless `smtp_from` or `SMTP_FROM` is configured), `reply_to`, `cc` (an address or non-empty address array), `body_html` (an HTML alternative body sent as `multipart/alternative` alongside `body`), and `attachments` (an array of `{ "filename": "...", "data_url": "data:..." }` objects). Invalid JSON and invalid or unknown properties return `{ "status": "error", "error_code": "...", "error": "..." }`. SQL `NULL` returns SQL `NULL` without sending.',
        'JSON'
    );
