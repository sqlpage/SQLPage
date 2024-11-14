set n=coalesce($n, 1);

select 
    'chart'                    as component,
    'Syracuse Sequence'        as title,
    coalesce($type, 'area')    as type,
    coalesce($color, 'indigo')  as color,
    5                          as marker,
    0                          as ymin;
with recursive seq(x, y) as (
    select 0, CAST($n as integer)
    union all
    select x+1, case
        when y % 2 = 0 then y/2
        else 3*y+1
    end
    from seq
    where x<10
)
select x, y from seq;

