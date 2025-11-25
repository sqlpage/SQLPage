set actual = sqlpage.read_file_as_data_url('tests/it_works.txt');
select 'data:text/plain;base64,SXQgd29ya3MgIQ==' as expected,
    coalesce($actual, 'NULL') as actual;
