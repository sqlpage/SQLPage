-- Write the name of the group in the title of the page
SELECT 'title' as component, name as contents FROM expense_group WHERE id = $id;


-- Handle the form to add a member to the group (we do it at the top of the page to see it right away)
INSERT INTO group_member(group_id, name)
SELECT $id, :new_member_name WHERE :new_member_name IS NOT NULL;

-- List of members of the group
SELECT 'list' as component, 'Membres' as title;
SELECT name AS title FROM group_member WHERE group_id = $id;

-- Form to add a new member to the group
SELECT 'form' as component, 'Ajouter un membre au groupe' as validate;
SELECT 'Nom du membre' AS 'label', 'new_member_name' AS name;

SELECT 'title' as component, 'Dépenses' as contents

-- Form to add an expense
SELECT 'form' as component, 'Ajouter une dépense' as title, 'Ajouter' as validate;
SELECT 'Description' AS name;
SELECT 'Montant' AS name, 'number' AS type;
SELECT 'Dépensé par' AS name, 'select' as type,
    json_group_array(json_object("label", name, "value", id)) as options
FROM group_member WHERE group_id = $id;

-- Insert the expense posted by the form into the database
INSERT INTO expense(spent_by, name, amount)
SELECT :"Dépensé par", :Description, :Montant WHERE :Montant IS NOT NULL;

-- List of expenses of the group
SELECT 'card' as component, 'Dépenses' as title;
SELECT expense.name as title,
    'Par ' || group_member.name || ', le ' || expense.date as description,
    expense.amount || ' €' as footer,
    CASE
        WHEN expense.amount > 100 THEN 'red'
        WHEN expense.amount > 50 THEN 'orange'
        ELSE 'blue'
    END AS color
FROM expense
    INNER JOIN group_member on expense.spent_by = group_member.id
WHERE group_member.group_id = $id;

-- Show the positive and negative debts of each member
SELECT 'chart' AS component, 'Dette par personne' AS title, 'bar' AS type, TRUE AS horizontal;
SELECT member_name AS label, is_owed AS value FROM individual_debts
WHERE group_id = $id ORDER BY is_owed DESC;