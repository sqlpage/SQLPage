INSERT INTO orders(customer_name, customer_email) 
VALUES (:Name, :Email);

INSERT INTO order_items(order_id, quantity, product_id)
SELECT
    last_insert_rowid() AS order_id, -- The id of the order we just inserted. Requires SQLPage v0.9.0 or later.
    CAST(value AS INTEGER) AS quantity,
    CAST(key AS INTEGER) AS product_id -- extracts converts the field name into a number
FROM JSON_EACH(sqlpage.variables('post')) -- Iterates on all posted variables. Requires SQLPage v0.15.0 or later
WHERE product_id > 0 -- we used the product id as a variable name in the product quatntity form fields 
   and CAST(value AS INTEGER) > 0
RETURNING 'redirect' as component, 'orders.sql?id=' || order_id as link;