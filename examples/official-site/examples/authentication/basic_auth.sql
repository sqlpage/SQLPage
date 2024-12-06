select 'http_header' as component, 'noindex' as "X-Robots-Tag";

SELECT 'authentication' AS component,
    case sqlpage.basic_auth_username()
        when 'admin'
            then '$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$oKBq5E8XFTHO2w' -- the password is 'password'
        when 'user'
            then '$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$qsrWdjgl96ooYw' -- the password is 'user'
    end AS password_hash, -- this is a hash of the password 'password'
    sqlpage.basic_auth_password() AS password; -- this is the password that the user entered in the browser popup

select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '
# Authentication

Read the [source code](//github.com/sqlpage/SQLPage/blob/main/examples/official-site/examples/authentication/basic_auth.sql) for this demo.
' as contents_md;

select 'alert' as component, 'info' as color, CONCAT('You are logged in as ', sqlpage.basic_auth_username()) as title;
