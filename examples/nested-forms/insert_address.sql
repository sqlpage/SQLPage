INSERT INTO address (user_id, street, city, country) VALUES ($user_id, :Street, :City, :Country)
RETURNING
    'http_header' AS component,
    'edit_user.sql?id=' || user_id AS location;