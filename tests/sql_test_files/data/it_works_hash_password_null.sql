select 'NULL' as expected, coalesce(sqlpage.hash_password(null), 'NULL') as actual;
