
-- Simple form to create a shared expense account
SELECT 'form' as component,
    'Nouveau compte partagé' as title,
    'Créer le compte de dépenses partagé !' as validate;
SELECT 'Nom du compte' AS label,
    'shared_expense_name' AS name;

-- Insert the shared expense account posted by the form into the database
INSERT INTO expense_group(name)
SELECT :shared_expense_name
WHERE :shared_expense_name IS NOT NULL;

-- List of shared expense accounts
-- (we put it after the insertion because we want to see new accounts right away when they are created)
SELECT 'list' as component;
SELECT name AS title,
    'group.sql?id=' || id AS link
FROM expense_group;