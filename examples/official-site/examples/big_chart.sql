select 
    'chart'    as component,
    'bar'      as type,
    TRUE       as toolbar,
    TRUE       as time;

with recursive integers(i) as (
    select 0 as i
    union all
    select i + 1
    from integers
    where i + 1 < 100
)
select 
    'S' || (i%10) as series,
    format('%d-01-01', 2010 + (i/10))     as x,
    abs(random() % 10)   as value
from integers;
