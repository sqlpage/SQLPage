set attachment_name = sqlpage.uploaded_file_name('attachment');
set attachment_path = sqlpage.uploaded_file_path('attachment');
set attachment_data_url = sqlpage.read_file_as_data_url($attachment_path);

set message = json_object(
    'to', json_array(:recipient, :second_recipient),
    'cc', :cc,
    'from', :sender,
    'reply_to', :reply_to,
    'subject', :subject,
    'body', :body,
    'body_html', NULLIF(:body_html, ''),
    'attachments', json_array(json_object(
        'filename', $attachment_name,
        'data_url', $attachment_data_url
    ))
);
set result = sqlpage.send_mail($message);

select 'shell' as component, 'Advanced email result' as title;

select
    'alert' as component,
    'send-result' as id,
    case when json_extract($result, '$.status') = 'accepted' then 'success' else 'danger' end as color,
    case when json_extract($result, '$.status') = 'accepted' then 'Email sent successfully' else 'Email could not be sent' end as title,
    json_extract($result, '$.error') as description;

select 'button' as component;
select 'Open Mailpit inbox' as title, 'http://localhost:8025' as link, '_blank' as target, 'inbox' as icon, 'primary' as color;
select 'Send another advanced email' as title, 'advanced.sql' as link;
select 'Back to examples' as title, 'index.sql' as link, 'secondary' as color;
