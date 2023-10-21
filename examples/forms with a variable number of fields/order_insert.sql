INSERT INTO orders(customer_name, customer_email) 
VALUES (:Name, :Email);

INSERT INTO order_items(order_id, quantity, product_id)
SELECT
    last_insert_rowid() AS order_id, -- The id of the order we just inserted. Requires SQLPage v0.9.0 or later.
    CAST(value AS INTEGER) AS quantity,
    CAST(substr(key, length('product_quantity_') + 1) AS INTEGER) AS product_id -- extracts $id from product_quantity_$id
FROM JSON_EACH(sqlpage.variables('post')) -- Iterates on all posted variables. Requires SQLPage v0.15.0 or later
WHERE key LIKE 'product_quantity_%' -- variable names that start with 'product_quantity_' 
   and CAST(value AS INTEGER) > 0
RETURNING 'redirect' as component, 'orders.sql?id=' || order_id as link;