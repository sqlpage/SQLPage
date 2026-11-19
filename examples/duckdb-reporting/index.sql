select 'json' as component;

select sale_id, amount
from read_csv(
    'sales.csv',
    header = true,
    columns = {'sale_id': 'INTEGER', 'region': 'VARCHAR', 'amount': 'DECIMAL(12,2)'}
)
where region = coalesce($region, 'North')
order by sale_id desc;
