select 'shell' as component, 'Monthly sales' as title;
select 'table' as component, 'Sales by month and region' as title, 'Report' as markdown;

select
    strftime(sale_month, '%Y-%m') as Month,
    region as Region,
    count(*) as Sales,
    sum(amount) as Revenue,
    '[View sales](' || sqlpage.link(
        'sales.sql',
        json_object('month', sale_month, 'region', region)
    ) || ')' as Report
from sales_report
group by all
order by Month desc, Region;
