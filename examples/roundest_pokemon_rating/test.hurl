GET http://localhost:8080/
HTTP 302
[Asserts]
header "Location" == "/populate.sql"

GET http://localhost:8080/favicon.ico
HTTP 200
