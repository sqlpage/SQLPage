-- Checks that the UPPER function is working correctly with unicode characters.
select 'É' as expected,
    UPPER('é') as actual;