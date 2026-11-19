select 'table' as component, 'Sales and targets' as title;

select
    sales.sale_id,
    sales.sold_on,
    sales.amount,
    arg_max_null(targets.target_amount, targets.effective_on) as target_amount
from read_csv(
    'sales/*.csv',
    header = true,
    columns = {
        'sale_id': 'INTEGER', 'region': 'VARCHAR',
        'sold_on': 'DATE', 'amount': 'DECIMAL(12,2)'
    }
) as sales
left join read_csv(
    'targets.csv',
    header = true,
    columns = {'region': 'VARCHAR', 'effective_on': 'DATE', 'target_amount': 'DECIMAL(12,2)'}
) as targets
    on sales.region = targets.region
    and sales.sold_on >= targets.effective_on
where time_bucket(interval '1 month', sales.sold_on) = cast($month as date)
    and sales.region = $region
group by all
order by sales.sold_on desc, sales.sale_id desc;
