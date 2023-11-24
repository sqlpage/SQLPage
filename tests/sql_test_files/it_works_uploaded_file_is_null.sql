-- checks that sqlpage.uploaded_file_path returns null when there is no uploaded_file
set actual = sqlpage.uploaded_file_path('my_file');
select 'text' as component, 
    case when $actual is null
          then 'It works !'
        else 'Failed. Expected: null. Got: ' || $actual
        end as contents;
