GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Todo"
body contains "Add new todo"
body not contains "An error occurred"

POST http://localhost:8080/todo_form.sql
[FormParams]
todo: Created by Hurl
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080/
HTTP 200
[Captures]
todo_id: xpath "substring-after(string((//a[contains(@href, 'todo_form.sql?todo_id=')])[last()]/@href), 'todo_id=')"
[Asserts]
body contains "Created by Hurl"
body htmlUnescape contains "todo_form.sql?todo_id={{todo_id}}"
body htmlUnescape contains "delete.sql?todo_id={{todo_id}}"
body not contains "An error occurred"

POST http://localhost:8080/todo_form.sql?todo_id={{todo_id}}
[FormParams]
todo: Edited by Hurl
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080/delete.sql?todo_id={{todo_id}}
HTTP 200
[Asserts]
body contains "Confirm deletion"
body contains "Edited by Hurl"
body htmlUnescape contains "?todo_id={{todo_id}}&confirm=yes"
body not contains "An error occurred"

GET http://localhost:8080/delete.sql?todo_id={{todo_id}}&confirm=yes
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080/
HTTP 200
[Asserts]
body not contains "Edited by Hurl"
body not contains "An error occurred"
