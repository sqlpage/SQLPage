INSERT INTO products (name, price)
VALUES (:Name, :Price)
RETURNING
    'redirect' AS component,
    'index.sql' AS link
;