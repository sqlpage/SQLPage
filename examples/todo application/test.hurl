GET http://localhost:8080
HTTP 200
[Asserts]
body contains "Todo"
body contains "Add new todo"

POST http://localhost:8080/todo_form.sql
[FormParams]
todo: Hurl todo item
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080
HTTP 200
[Asserts]
body contains "Hurl todo item"
