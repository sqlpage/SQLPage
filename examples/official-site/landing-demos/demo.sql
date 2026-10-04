-- Only these four public examples can be selected; no request-controlled paths.
set demo = (select case $component when 'chart' then 'chart' when 'form' then 'form'
    when 'big_number' then 'big_number' else 'table' end);
select 'shell' as component, 'Live SQLPage demo' as title,
    'dark' as theme, '' as footer, 'fluid' as layout,
    '/assets/landing/styles/demo.css' as css;
select 'dynamic' as component,
    sqlpage.run_sql('landing-demos/' || $demo || '.sql') as properties;
