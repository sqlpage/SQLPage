set demo = (select case $component when 'chart' then 'chart' when 'form' then 'form'
    when 'big_number' then 'big_number' else 'table' end);
select 'json' as component,
    json_object('source', sqlpage.read_file_as_text('landing-demos/' || $demo || '.sql')) as contents;
