GET http://localhost:8080/index.sql
HTTP 200
[Asserts]
body contains "Available Items"
body contains "Selected Items"
body contains "John Smith"
body contains "Jane Doe"
body not contains "An error occurred"

POST http://localhost:8080/form_action.sql
[FormParams]
selected_items[]: 6
selected_items[]: 8
HTTP 200
[Asserts]
body contains "Group members successfully updated"
body contains "Users in this group"
body contains "Sarah Davis"
body contains "sarah@email.com"
body contains "Lisa Anderson"
body contains "lisa@email.com"
body not contains "John Smith"
body not contains "An error occurred"
