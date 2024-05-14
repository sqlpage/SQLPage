set actual = sqlpage.request_method()
set expected = 'GET';

select 'text' as component, 
    case $actual
        when $expected
          then 'It works !'
        else 
            'Failed.
            Expected: ' || $expected ||
            'Got: ' || $actual
        end as contents;
