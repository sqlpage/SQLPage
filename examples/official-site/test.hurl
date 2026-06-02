GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "SQLPage"
body contains "Documentation"
body contains "Get Started"
body not contains "An error occurred"

GET http://localhost:8080/documentation.sql
HTTP 200
[Asserts]
body contains "SQLPage v"
body contains "components"
body contains "shell"
body not contains "An error occurred"

GET http://localhost:8080/component.sql?component=form
HTTP 200
[Asserts]
body contains "The form component"
body contains "Building forms in SQL"
body not contains "An error occurred"

GET http://localhost:8080/functions.sql?function=url_encode
HTTP 200
[Asserts]
body contains "sqlpage.url_encode"
body contains "Parameters"
body not contains "An error occurred"

GET http://localhost:8080/search.sql?search=form
HTTP 302
[Asserts]
header "Location" == "/component.sql?component=form"

GET http://localhost:8080/not-a-real-doc-page
HTTP 404
[Asserts]
body contains "Not Found"
body not contains "An error occurred"

GET http://localhost:8080/examples/authentication/
HTTP 302
[Asserts]
header "Location" == "login.sql"

GET http://localhost:8080/examples/authentication/login.sql
HTTP 200
[Asserts]
body contains "Demo Login Form"
body contains "admin"
body not contains "An error occurred"

POST http://localhost:8080/examples/authentication/create_session_token.sql
[FormParams]
username: admin
password: admin
HTTP 302
[Asserts]
header "Location" == "/examples/authentication"

GET http://localhost:8080/examples/authentication/
HTTP 200
[Asserts]
body contains "You are logged in as admin"
body contains "Log out"
body not contains "An error occurred"

GET http://localhost:8080/examples/authentication/logout.sql
HTTP 302
[Asserts]
header "Location" == "login.sql"
