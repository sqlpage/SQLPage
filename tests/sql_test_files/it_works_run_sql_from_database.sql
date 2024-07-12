select 'dynamic' as component,
    sqlpage.run_sql('tests/display_text.sql', json_object('html', html)) as properties
from (
    select 'It ' as html
    union all
    select 'works !'
) as t1;