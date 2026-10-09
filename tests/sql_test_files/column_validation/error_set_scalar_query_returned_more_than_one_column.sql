set invalid_value = (
    select n as duplicate, n as duplicate from (select 1 as n) source_rows
);
