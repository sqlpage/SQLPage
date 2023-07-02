-- A view that shows all the expenses of a group, with at least one line per member
DROP VIEW IF EXISTS members_with_expenses;
CREATE VIEW members_with_expenses AS
SELECT group_member.group_id AS group_id,
    group_member.id AS spent_by_id,
    COALESCE(expense.amount, 0) AS amount,
    group_member.name AS spent_by_name
FROM group_member
    LEFT JOIN expense on expense.spent_by = group_member.id;
-- A view that shows the total amount of expense per person of a group
DROP VIEW IF EXISTS average_debt_per_person;
CREATE VIEW average_debt_per_person AS
SELECT group_id,
    sum(amount) / count(distinct spent_by_id) AS debt
FROM members_with_expenses
GROUP BY group_id;
-- A view that shows the total amount a person is owed in a group
DROP VIEW IF EXISTS individual_debts;
CREATE VIEW individual_debts AS
SELECT members_with_expenses.group_id AS group_id,
    spent_by_id AS member_id,
    spent_by_name AS member_name,
    sum(members_with_expenses.amount) - average_debt_per_person.debt AS is_owed
FROM members_with_expenses
    INNER JOIN average_debt_per_person ON average_debt_per_person.group_id = members_with_expenses.group_id
GROUP BY spent_by_id
ORDER BY is_owed DESC;