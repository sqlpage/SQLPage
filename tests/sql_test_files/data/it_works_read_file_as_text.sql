select 'It works !' as expected,
    sqlpage.read_file_as_text('tests/it_works.txt') as actual;
