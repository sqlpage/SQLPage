-- Preview only: request-local variables never change the shared demo database.
set valid = sqlpage.request_method() = 'POST'
    and length(trim(:name)) between 1 and 80
    and length(coalesce(:team, '')) <= 160;
set component = 'form';
set saved = case when $valid then '1' else '0' end;
select 'dynamic' as component,
    sqlpage.run_sql('landing-demos/demo.sql') as properties;
