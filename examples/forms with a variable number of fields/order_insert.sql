INSERT INTO orders(customer_name, customer_email) 
VALUES (:Name, :Email);

INSERT INTO order_items(order_id, quantity, product_id)
SELECT
    last_insert_rowid(), -- The id of the order we just inserted. Requires SQLPage v0.9.0 or later.
    CAST(quantity.value AS INTEGER),
    CAST(product.value AS INTEGER)
FROM JSON_EACH(:product_quantity) quantity
INNER JOIN JSON_EACH(:product_id) product USING (key)
WHERE CAST(quantity.value AS INTEGER) > 0
RETURNING
    'orders.sql?id=' || order_id as link,
    'redirect' as component;