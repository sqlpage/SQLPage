select 'table' as component, 'Sales and targets' as title;

select sale_id, sold_on, amount, target_amount
from sales_report
where sale_month = cast($month as date) and region = $region
order by sold_on desc, sale_id desc;
