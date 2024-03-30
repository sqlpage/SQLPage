select 'text' as component,
    case sqlpage.url_encode(sqlpage.read_file_as_text('tests/it_works.txt'))
        when 'It%20works%20%21' then 'It works !'
        else 'Error! Nested sqlpage functions are not working as expected.'
    end as contents;
