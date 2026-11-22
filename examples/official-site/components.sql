select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Components - SQLPage Documentation', 'section', 'Components',
    'description', 'Explore SQLPage components by purpose, with live examples and parameters for each one.'
)) as properties;

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-catalog.sql', json_object(
    'kind', 'component', 'q', $q, 'category', $category
)) as properties;
