SELECT 'shell' AS component, 'Order' AS title;

SELECT 'form' as component,
    'Pass an order' as title,
    'Order' as validate,
    'order_insert.sql' as action;

SELECT 'Name' as name, 'Your full name' AS placeholder;
SELECT 'Email' as name, 'Your email address' AS placeholder;
SELECT id AS name,
    'Quantity of ' || name || '' AS label,
    'Number of ' || name || ' you wish to order, for ' || price || ' € each.' AS description,
    'number' AS type,
    1 AS step,
    0 as min,
    0 as value
FROM products
ORDER BY id;
