GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "New shared expense account"

POST http://localhost:8080/
Content-Type: application/x-www-form-urlencoded
[FormParams]
shared_expense_name: Hurl Trip
HTTP 200
[Captures]
group_id: xpath "substring-after(string((//a[contains(., 'Hurl Trip')])[last()]/@href), 'id=')"
[Asserts]
body contains "Hurl Trip"

POST http://localhost:8080/group.sql?id={{group_id}}
Content-Type: application/x-www-form-urlencoded
[FormParams]
new_member_name: Alice
HTTP 200
[Asserts]
body contains "Alice"
body contains "Add an expense"

POST http://localhost:8080/group.sql?id={{group_id}}
Content-Type: application/x-www-form-urlencoded
[FormParams]
new_member_name: Bob
HTTP 200
[Captures]
alice_id: xpath "string(//select[@name='Spent By']/option[normalize-space(.)='Alice']/@value)"
[Asserts]
body contains "Bob"

POST http://localhost:8080/group.sql?id={{group_id}}
Content-Type: application/x-www-form-urlencoded
`Description=Train+tickets&Amount=90&Spent+By={{alice_id}}`
HTTP 200
[Asserts]
body contains "Train tickets"
body contains "90 €"
body contains "Debts by Person"
body contains "&quot;Alice&quot;,45"
body contains "&quot;Bob&quot;,-45"
