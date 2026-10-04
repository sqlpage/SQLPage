GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "SQLPage"
body contains "Documentation"
body contains "Run it"
body contains "Instant by default."
body contains "Override any component."
body contains "Copy files. Or don't."
xpath "string(//button[@id='instant-tab']/@aria-selected)" == "true"
body contains "/assets/landing/js/main.js"
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

GET http://localhost:8080/landing-demos/demo.sql?component=table
HTTP 200
[Asserts]
body contains "Acme Corp"
body contains "data-pre-init=\"table\""
body not contains "An error occurred"

GET http://localhost:8080/landing-demos/demo.sql?component=chart
HTTP 200
[Asserts]
body contains "Tickets resolved"
body contains "2026-09-30"
body not contains "An error occurred"

POST http://localhost:8080/landing-demos/save.sql
[FormParams]
name: Ada
team: Query crew
HTTP 302
[Asserts]
header "Location" == "/landing-demos/demo.sql?component=form&saved=1"

GET http://localhost:8080/landing-demos/demo.sql?component=form&saved=1
HTTP 200
[Asserts]
body contains "Hello, Ada!"
body contains "Query crew"
body not contains "An error occurred"

GET http://localhost:8080/landing-demos/demo.sql?component=big_number
HTTP 200
[Asserts]
body contains "98%"
body not contains "An error occurred"

GET http://localhost:8080/landing-demos/source.sql?component=../../configuration
HTTP 200
[Asserts]
jsonpath "$.source" contains "from customers"


GET http://localhost:8080/landing-demos/source.sql?component=form
HTTP 200
[Asserts]
jsonpath "$.save_source" contains "update profiles"

GET http://localhost:8080/landing-demos/demo.sql?component=catalog
HTTP 200
[Asserts]
body contains "component-catalog"
xpath "string(//a[span='form']/@href)" == "/component.sql?component=form"
body not contains "An error occurred"

POST http://localhost:8080/landing-demos/save.sql
[FormParams]
name:
team: Rejected input
HTTP 302
[Asserts]
header "Location" == "/landing-demos/demo.sql?component=form&saved=0"

GET http://localhost:8080/landing-demos/demo.sql?component=form
HTTP 200
[Asserts]
body contains "Query crew"
body not contains "Rejected input"
