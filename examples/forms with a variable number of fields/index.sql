SELECT 'shell' AS component, 'Products' AS title;

SELECT 'list' AS component, 'Products' AS title;
SELECT 'Add a new product' AS title,
    'red' AS color,
    'new_product_form.sql' AS link,
    TRUE AS active;
SELECT 'Pass an order' AS title,
    'blue' AS color,
    'order_form.sql' AS link,
    TRUE AS active;
SELECT 
    name AS title,
    price || '€' AS description
FROM products;