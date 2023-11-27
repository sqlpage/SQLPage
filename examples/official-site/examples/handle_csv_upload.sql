-- temporarily store the data in a table with text columns
create temporary table if not exists product_tmp(name text, description text, price text);
delete from product_tmp;

-- copy the data from the CSV file into the temporary table
copy product_tmp(name, description, price) from 'product_data_file';

select 'table' as component;
select * from product_tmp;