create temporary view sales_report as
select
    sales.sale_id,
    sales.region,
    sales.sold_on,
    time_bucket(interval '1 month', sales.sold_on) as sale_month,
    sales.amount,
    targets.target_amount
from read_csv(
    'sales/*.csv',
    header = true,
    columns = {
        'sale_id': 'INTEGER', 'region': 'VARCHAR',
        'sold_on': 'DATE', 'amount': 'DECIMAL(12,2)'
    }
) as sales
asof left join read_csv(
    'targets.csv',
    header = true,
    columns = {'region': 'VARCHAR', 'effective_on': 'DATE', 'target_amount': 'DECIMAL(12,2)'}
) as targets
    on sales.region = targets.region
    and sales.sold_on >= targets.effective_on;
