select 'dynamic' as component,
    sqlpage.run_sql('tests/sql_test_files/component_rendering/dynamic_shell.sql') as properties;
