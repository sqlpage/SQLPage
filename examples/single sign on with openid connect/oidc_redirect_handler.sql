-- If the oauth_state cookie does not match the state parameter in the query string, then the request is invalid (CSRF attack)
-- and we should redirect the user to the login page.
select 'redirect' as component, '/oidc_login.sql' as link
    where sqlpage.cookie('oauth_state') != $state;

-- Exchange the authorization code for an access token
set $authorization_code_request = json_object(
    'url', 'http://keycloak:8181/realms/sqlpage_demo/protocol/openid-connect/token', -- replace this with the URL of your OpenID Connect provider
    'method', 'POST',
    'headers', json_object(
        'Content-Type', 'application/x-www-form-urlencoded'
    ),
    'body', 'grant_type=authorization_code'
        || '&code=' || $code
        || '&redirect_uri=http://localhost:8080/oidc_redirect_handler.sql' -- replace this with the URL of your application
        || '&client_id=' || sqlpage.environment_variable('OIDC_CLIENT_ID')
        || '&client_secret=' || sqlpage.environment_variable('OIDC_CLIENT_SECRET')
);
set $access_token = sqlpage.fetch($authorization_code_request);

-- Redirect the user to the login page if the access token could not be obtained
select 'redirect' as component, '/oidc_login.sql' as link
    where $access_token is null or $access_token->>'error' is not null;

-- At this point we have $access_token which contains {"access_token":"eyJ...", "scope":"openid profile email" }

-- Fetch the user's profile
set $profile_request = json_object(
    'url', 'http://keycloak:8181/realms/sqlpage_demo/protocol/openid-connect/userinfo', -- replace this with the URL of your OpenID Connect provider
    'method', 'GET',
    'headers', json_object(
        'Authorization', 'Bearer ' || ($access_token->>'access_token')
    )
);
set $user_profile = sqlpage.fetch($profile_request);

-- Redirect the user to the login page if the user's profile could not be obtained
select 'redirect' as component, '/oidc_login.sql' as link
    where $user_profile is null or $user_profile->>'error' is not null;

-- at this point we have $user_profile which contains {"sub":"0cc01234","email_verified":false,"name":"John Smith","preferred_username":"demo","given_name":"John","family_name":"Smith","email":"demo@example.com"}

-- Now we have a valid access token, we can create a session for the user
-- in our database
insert into user_sessions(session_id, user_id, email)
    values(sqlpage.random_string(32), $user_profile->>'sub', $user_profile->>'email') -- you can get additional information like 'name', 'given_name', 'family_name', 'email_verified', 'preferred_username', 'picture' from the user profile
    returning 'cookie' as component, 'session_id' as name, session_id as value;

-- Redirect the user to the home page
select 'redirect' as component, '/' as link;