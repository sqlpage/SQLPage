-- The CAS server will redirect the user to this URL after the user has authenticated
-- This page will be loaded with a ticket parameter in the query string, which we can read in the variable $ticket

-- If we don't have a ticket, go back to the CAS login page
select 'redirect' as component, '/cas/' as link where $ticket is null;

-- We must then validate the ticket with the CAS server
-- CAS v3 specifies the following URL for ticket validation (see https://apereo.github.io/cas/6.6.x/protocol/CAS-Protocol-Specification.html#28-p3servicevalidate-cas-30)
-- https://cas.example.org/p3/serviceValidate?ticket=ST-1856339-aA5Yuvrxzpv8Tau1cYQ7&service=http://myclient.example.org/myapp&format=JSON
SET $ticket_url =
    sqlpage.environment_variable('CAS_ROOT_URL') 
        || '/p3/serviceValidate'
        || '?ticket=' || sqlpage.url_encode($ticket)
        || '&service=' || sqlpage.protocol() || '://' || sqlpage.header('host') || '/cas/redirect_handler.sql'
        || '&format=JSON';

-- We must then make a request to the CAS server to validate the ticket
set $validation_response = sqlpage.fetch($ticket_url);

-- If the ticket is invalid, the CAS server will return a 200 OK response with a JSON object like this:
-- { "serviceResponse": { "authenticationFailure": { "code": "INVALID_TICKET", "description": "..." } } }
select 'redirect' as component,
    '/cas/login.sql' as link
where $validation_response->'serviceResponse'->'authenticationFailure' is not null;

-- If the ticket is valid, the CAS server will return a 200 OK response with a JSON object like this:
-- { "serviceResponse": { "authenticationSuccess": { "user": "username", "attributes": { "attribute": "value" } } } }
-- You can use the following SQL code to inspect what the CAS server returned:
-- select 'debug' as component, $validation_response;
insert into user_sessions(session_id, user_id, email, oidc_token)
    values(
        sqlpage.random_string(32),
        $validation_response->'serviceResponse'->'authenticationSuccess'->>'user', -- The '->' operator extracts a JSON object field as JSON, while the '->>' operator extracts a JSON object field as text
        $validation_response->'serviceResponse'->'authenticationSuccess'->'attributes'->>'mail',
        $ticket
    )
returning 
    'cookie' as component, 'session_id' as name, session_id as value;

-- Redirect the user to the home page
select 'redirect' as component, '/cas/' as link;
