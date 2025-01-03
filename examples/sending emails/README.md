# Sending Emails with SQLPage

SQLPage lets you interact with any email service through their API,
using the [`sqlpage.fetch` function](https://sql-page.com/functions.sql?function=fetch).

## Why Use an Email Service?

Sending emails directly from your server can be challenging:
- Many ISPs block direct email sending to prevent spam
- Email deliverability requires proper setup of SPF, DKIM, and DMARC records
- Managing bounce handling and spam complaints is complex
- Direct sending can impact your server's IP reputation

Email services solve these problems by providing reliable APIs for sending emails while handling deliverability, tracking, and compliance.

## Popular Email Services

- [Mailgun](https://www.mailgun.com/) - Developer-friendly, great for transactional emails
- [SendGrid](https://sendgrid.com/) - Powerful features, owned by Twilio
- [Amazon SES](https://aws.amazon.com/ses/) - Cost-effective for high volume
- [Postmark](https://postmarkapp.com/) - Focused on transactional email delivery
- [SMTP2GO](https://www.smtp2go.com/) - Simple SMTP service with API options

## Example: Sending Emails with Mailgun

Here's a complete example using Mailgun's API to send emails through SQLPage:

### [`email.sql`](./email.sql)
```sql
-- Configure the email request
set email_request = json_object(
    'url', 'https://api.mailgun.net/v3/' || sqlpage.environment_variable('MAILGUN_DOMAIN') || '/messages',
    'method', 'POST',
    'headers', json_object(
        'Content-Type', 'application/x-www-form-urlencoded',
        'Authorization', 'Basic ' || encode(('api:' || sqlpage.environment_variable('MAILGUN_API_KEY'))::bytea, 'base64')
    ),
    'body', 
        'from=Your Name <noreply@' || sqlpage.environment_variable('MAILGUN_DOMAIN') || '>'
        || '&to=' || $to_email
        || '&subject=' || $subject
        || '&text=' || $message_text
        || '&html=' || $message_html
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
```

### Setup Instructions

1. Sign up for a [Mailgun account](https://signup.mailgun.com/new/signup)
2. Verify your domain or use the sandbox domain for testing
3. Get your API key from the Mailgun dashboard
4. Set these environment variables in your SQLPage configuration:
   ```
   MAILGUN_API_KEY=your-api-key-here
   MAILGUN_DOMAIN=your-domain.com
   ```

## Best Practices

- If you share your code with others, it should not contain sensitive data like API keys
  - Instead, use environment variables with [`sqlpage.environment_variable`](https://sql-page.com/functions.sql?function=environment_variable)
- Implement proper error handling
- Consider rate limiting for bulk sending
- Include unsubscribe links when sending marketing emails
- Follow email regulations (GDPR, CAN-SPAM Act)
