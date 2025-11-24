select 'NULL' as expected,
    coalesce(sqlpage.client_ip(), 'NULL') as actual;