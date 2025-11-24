set i_am_null = NULL;
select 'NULL' as expected,
    coalesce($i_am_null, 'NULL') as actual;
