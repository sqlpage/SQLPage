select 'text' as component, coalesce(sqlpage.environment_variable('I_DO_NOT_EXIST'), 'It works !') as contents;