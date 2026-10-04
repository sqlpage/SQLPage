select 'big_number' as component, 'landing-kpis' as id, 2 as columns;
select 'Tickets resolved' as title, count(*) as value,
    'This week' as description, 'cyan' as color
from tickets;
select 'Happy customers' as title,
    cast(round(100.0 * avg(satisfied)) as integer) || '%' as value,
    'Positive customer feedback' as description
from customer_feedback;
