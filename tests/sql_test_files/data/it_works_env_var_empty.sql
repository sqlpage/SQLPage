select 'NULL' as expected,
    coalesce(sqlpage.environment_variable('I_DO_NOT_EXIST'), 'NULL') as actual;