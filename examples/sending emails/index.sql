select 'shell' as component, 'Sending emails with SQLPage' as title;

select
    'text' as component,
    'Choose a focused example. Both send through the SMTP server configured for SQLPage.' as contents;

select 'card' as component, 2 as columns;
select
    'Simple email' as title,
    'Send a plain-text message using the configured default sender.' as description,
    'simple.sql' as link;
select
    'Advanced email' as title,
    'Multiple recipients, Cc, Reply-To, a sender override, an HTML body, and an uploaded attachment.' as description,
    'advanced.sql' as link;
