select 'big_number' as component, 'landing-kpis' as id, 2 as columns;
select 'Tickets resolved' as title, count(*) as value,
    'Weekly goal: 50 tickets' as description, 'cyan' as color,
    round(100.0 * count(*) / 50) as progress_percent, 'cyan' as progress_color
from tickets;
select 'Happy customers' as title,
    cast(round(100.0 * avg(satisfied)) as integer) || '%' as value,
    'From ' || count(*) || ' recent responses' as description,
    'teal' as color, round(100.0 * avg(satisfied)) as progress_percent,
    'teal' as progress_color
from customer_feedback;
