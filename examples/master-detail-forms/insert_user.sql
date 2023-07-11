INSERT INTO user (id, first_name, last_name, email) VALUES ($id, :"First name", :"Last name", :Email)
ON CONFLICT (id) DO UPDATE SET first_name = excluded.first_name, last_name = excluded.last_name, email = excluded.email
RETURNING
    'redirect' AS component,
    'edit_user.sql?id=' || id AS link; 