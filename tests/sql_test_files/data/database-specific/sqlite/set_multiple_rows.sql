set actual = (
    select 'first' as value
    union all
    select 'second' as value
);

select 'first' as expected, $actual as actual;
