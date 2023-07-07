-- Checks if the login was successful, and redirects to the right page.
SELECT 'http_header' AS component,
    CASE WHEN is_valid_session(sqlpage.cookie('session')) 
        THEN 'protected_page.sql'
        ELSE 'sign in.sql'
    END AS location;