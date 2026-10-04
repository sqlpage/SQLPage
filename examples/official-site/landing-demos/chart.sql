select 'chart' as component, 'Tickets resolved' as title,
    'bar' as type, '#31cbd8' as color, 200 as height;
select date(resolved_at) as x, count(*) as y
from tickets
group by date(resolved_at)
order by x;
