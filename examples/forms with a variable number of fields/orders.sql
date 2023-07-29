SELECT 'alert' as component,
    'Thanks !' as title,
    'analyze' as icon,
    'teal' as color,
    'Your order is being processed. You will head from us soon.' as description;
SELECT 'index.sql' as link,
    'Back to homepage' as title;

SELECT 'list' AS component,
    'Order summary' AS title;
SELECT 
    quantity || ' x ' || name AS title,
    'Subtotal: ' || quantity || ' x ' || price || ' € = ' || (quantity * price) || ' €' AS description
FROM order_items
INNER JOIN products ON products.id = order_items.product_id
WHERE order_id = $id;

SELECT 
    'Total: ' || SUM(quantity * price) || ' €' AS title,
    'red' AS color,
    TRUE AS active
FROM order_items
INNER JOIN products ON products.id = order_items.product_id
WHERE order_id = $id;