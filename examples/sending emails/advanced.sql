select 'shell' as component, 'Send an advanced email' as title;

select
    'form' as component,
    'Advanced email' as title,
    'send-advanced.sql' as action,
    'post' as method,
    'Send email' as validate;

select 'email' as type, 'recipient' as name, 'To' as label, 'first@example.com' as value, true as required;
select 'email' as type, 'second_recipient' as name, 'Second recipient' as label, 'second@example.com' as value, true as required;
select 'email' as type, 'cc' as name, 'Cc' as label, 'team@example.com' as value, true as required;
select 'email' as type, 'sender' as name, 'From override' as label, 'advanced@example.com' as value, true as required;
select 'email' as type, 'reply_to' as name, 'Reply-To' as label, 'replies@example.com' as value, true as required;
select 'subject' as name, 'Subject' as label, 'Advanced SMTP demo' as value, true as required;
select 'textarea' as type, 'body' as name, 'Message' as label, 'Sent to a team with an attachment' as value, true as required;
select 'textarea' as type, 'body_html' as name, 'HTML body (optional)' as label, '<p>Sent to a team with an <strong>attachment</strong></p>' as value;
select 'file' as type, 'attachment' as name, 'Attachment' as label, true as required;

select 'button' as component;
select 'Back to examples' as title, 'index.sql' as link, 'arrow-left' as icon;
