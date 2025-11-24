select 'dynamic' as component,
    sqlpage.run_sql('tests/sql_test_files/component_rendering/it_works_simple.sql') as properties;
