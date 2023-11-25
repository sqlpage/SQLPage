set actual = sqlpage.read_file_as_data_url('tests/it_works.txt')
set expected = 'data:text/plain;base64,SXQgd29ya3MgIQ==';

select 'text' as component, 
    case $actual
        when $expected
          then 'It works !'
        else 
            'Failed.
            Expected: ' || $expected ||
            'Got: ' || $actual
        end as contents;
