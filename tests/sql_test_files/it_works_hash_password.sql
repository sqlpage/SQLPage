SELECT 'text' as component, 'It works ! The hashed password is: ' || sqlpage.hash_password($x) as contents;
