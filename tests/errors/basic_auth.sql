SELECT 'authentication' AS component,
    '$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$oKBq5E8XFTHO2w' AS password_hash, -- this is a hash of the password 'password'
    sqlpage.basic_auth_password() AS password; -- this is the password that the user entered in the browser popup

SELECT 'text' AS component, 'Success!' AS contents;
