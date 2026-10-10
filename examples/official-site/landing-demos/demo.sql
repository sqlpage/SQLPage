-- Public demos use fixed paths; no request-controlled file access.
set demo = (select case $component when 'chart' then 'chart' when 'form' then 'form'
    when 'big_number' then 'big_number' when 'catalog' then 'catalog' else 'table' end);
select 'shell' as component, 'Live SQLPage demo' as title,
    'dark' as theme, '' as footer, 'fluid' as layout,
    '/assets/landing/styles/demo.css' as css;
select 'alert' as component, 'Profile previewed' as title,
    'Hello, ' || trim(:name) || '! This preview is private to your submission.' as description,
    'circle-check' as icon, 'cyan' as color
where $demo = 'form' and $saved = '1' and sqlpage.request_method() = 'POST';
select 'alert' as component, 'Please check your profile' as title,
    'Enter a name up to 80 characters and a team up to 160 characters.' as description,
    'alert-circle' as icon, 'orange' as color
where $demo = 'form' and $saved = '0';
select 'dynamic' as component,
    sqlpage.run_sql('landing-demos/' || $demo || '.sql') as properties;
