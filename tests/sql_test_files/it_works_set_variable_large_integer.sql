SET test_stored_number = 123456789123456789123456789;
select 'text' as component,
    case $test_stored_number 
        when '123456789123456789123456789' then 'It works !'
        else 'It failed ! Expected 123456789123456789123456789 but got ' || COALESCE($test_stored_number, 'NULL')
    end
AS contents;