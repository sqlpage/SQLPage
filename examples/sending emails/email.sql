-- Configure the email request

-- Obtain the authorization by encoding "api:YOUR_PERSONAL_API_KEY" in base64
set authorization = 'YXBpOjI4ODlmODE3Njk5ZjZiNzA4MTdhODliOGUwODYyNmEyLWU2MWFlOGRkLTgzMjRjYWZm';

-- Find the domain in your Mailgun account
set domain = 'sandbox859545b401674a95b906ab417d48c97c.mailgun.org';

-- Set the recipient email address.
--In this demo, we accept sending any email to any address.
-- If you do this in production, spammers WILL use your account to send spam.
-- Your application should only allow emails to be sent to addresses you have verified.
set to_email = :to_email;

-- Set the email subject
set subject = :subject;

-- Set the email message text
set message_text = :message_text;

set email_request = json_object(
    'url', 'https://api.mailgun.net/v3/' || $domain || '/messages',
    'method', 'POST',
    'headers', json_object(
        'Content-Type', 'application/x-www-form-urlencoded',
        'Authorization', 'Basic ' || $authorization
    ),
    'body', 
        'from=Your Name <noreply@' || $domain || '>'
        || '&to=' || sqlpage.url_encode($to_email)
        || '&subject=' || sqlpage.url_encode($subject)
        || '&text=' || sqlpage.url_encode($message_text)
);
-- Send the email using sqlpage.fetch
set email_response = sqlpage.fetch($email_request);

-- Handle the response
select 
    'alert' as component,
    case 
        when $email_response->>'id' is not null then 'Email sent successfully'
        else 'Failed to send email: ' || ($email_response->>'message')
    end as title;