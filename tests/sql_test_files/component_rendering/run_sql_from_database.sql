select
    'dynamic' as component,
    sqlpage.run_sql (
        'tests/components/display_text.sql',
        CONCAT ('{"html":"', html, '"}') -- UNSAFE. Don't do that with untrusted data. We do it here because we can't use json_object (the syntax is not the same in all supported databases)
    ) as properties
from
    (
        select
            'It ' as html
        union all
        select
            'works !'
    );
