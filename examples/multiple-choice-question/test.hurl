GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "What dog lover are you ?"
body contains "process.sql"
body contains "I love dogs"
body contains "I hate them"
body not contains "An error occurred"

POST http://localhost:8080/process.sql
[FormParams]
profile: 1
HTTP 302
[Asserts]
header "Location" == "results.sql"

GET http://localhost:8080/results.sql
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "I love dogs"
body contains "100"
body not contains "An error occurred"
