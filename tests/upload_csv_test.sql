create table bill(quantity text, price text);
copy bill(quantity, price) from 'prices_file' with (format csv, header true);
select 'text' as component,
    'total: ' || sum(cast(quantity as float) * cast(price as float)) as contents
from bill;