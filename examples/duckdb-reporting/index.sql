select 'shell' as component, 'Monthly sales' as title;
select 'table' as component, 'Sales by month and region' as title, 'Report' as markdown;

with sales as (
    select region, amount, time_bucket(interval '1 month', sold_on) as sale_month
    from read_csv(
        'sales/*.csv',
        header = true,
        columns = {
            'sale_id': 'INTEGER', 'region': 'VARCHAR',
            'sold_on': 'DATE', 'amount': 'DECIMAL(12,2)'
        }
    )
)
select
    strftime(sale_month, '%Y-%m') as Month,
    region as Region,
    count(*) as Sales,
    sum(amount) as Revenue,
    '[View sales](' || sqlpage.link(
        'sales.sql',
        json_object('month', sale_month, 'region', region)
    ) || ')' as Report
from sales
group by all
order by Month desc, Region;
