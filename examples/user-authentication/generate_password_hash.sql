SELECT 'form' AS component;
SELECT 'password' AS name, 'Password to create a hash for' AS label, :password AS value;

SELECT 'code' AS component;
SELECT sqlpage.hash_password(:password) AS contents;