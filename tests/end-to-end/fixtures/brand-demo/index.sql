-- This isolated recording/test app deliberately supports real persistence.
CREATE TABLE IF NOT EXISTS brand_demo_customers (
    id INTEGER PRIMARY KEY, name TEXT NOT NULL,
    plan TEXT NOT NULL, status TEXT NOT NULL, seats INTEGER NOT NULL
);
DELETE FROM brand_demo_customers WHERE $reset = '1';
INSERT INTO brand_demo_customers
SELECT 1, 'Acme Corp', 'Pro', 'Active', 12 WHERE NOT EXISTS (SELECT 1 FROM brand_demo_customers WHERE id = 1);
INSERT INTO brand_demo_customers
SELECT 2, 'Globex', 'Team', 'Active', 8 WHERE NOT EXISTS (SELECT 1 FROM brand_demo_customers WHERE id = 2);
INSERT INTO brand_demo_customers
SELECT 3, 'Initech', 'Free', 'Trial', 3 WHERE NOT EXISTS (SELECT 1 FROM brand_demo_customers WHERE id = 3);
INSERT INTO brand_demo_customers
SELECT 4, 'Umbrella', 'Pro', 'Late', 20 WHERE NOT EXISTS (SELECT 1 FROM brand_demo_customers WHERE id = 4);
select 'shell' as component, 'Customer tracker' as title, '' as footer,
    '/brand-demo/style.css' as css, '/brand-demo/symbol.svg' as image;
select 'button' as component, 'end' as justify;
select 'Add a customer' as title, 'new_customer.sql' as link;
select 'dynamic' as component, sqlpage.run_sql('brand-demo/customers.sql') as properties;
