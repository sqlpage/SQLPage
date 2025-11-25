select 'It%20works%20%21' as expected, sqlpage.url_encode(sqlpage.read_file_as_text('tests/it_works.txt')) as actual;
