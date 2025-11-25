select '$argon2id$' as expected_contains,
    coalesce(sqlpage.hash_password($x), 'NULL') as actual;
