GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Add a user"
body contains "Users"
body not contains "An error occurred"

POST http://localhost:8080/
[FormParams]
Username: Hurl User
HTTP 200
[Asserts]
body contains "Hurl User"
body contains "user.sql?id"
body not contains "An error occurred"

GET http://localhost:8080/user.sql?id=1
HTTP 200
[Asserts]
body contains "Hurl User"
body contains "Edit user"
body contains "Delete this user"
body not contains "An error occurred"

POST http://localhost:8080/edit.sql?id=1
[FormParams]
Username: Hurl Admin
Administrator: on
HTTP 200
[Asserts]
body contains "Hurl Admin"
body contains "Has administrator privileges"
body not contains "An error occurred"

GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Hurl Admin"
body not contains "Hurl User is a user"
body not contains "An error occurred"

GET http://localhost:8080/delete.sql?id=1
HTTP 200
[Asserts]
body contains "The user Hurl Admin has been deleted."
body contains "Back to the user list"
body not contains "An error occurred"
