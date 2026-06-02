GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "User Name"
body contains "View the cookie from a subdirectory"
body not contains "An error occurred"

POST http://localhost:8080/
[FormParams]
username: Hurl Cookie
HTTP 200
[Asserts]
header "Set-Cookie" contains "username=Hurl%20Cookie"
body contains "Hurl Cookie"
body not contains "An error occurred"

GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Hurl Cookie"
body not contains "An error occurred"

GET http://localhost:8080/subdirectory/read_cookies.sql
HTTP 200
[Asserts]
body contains "The value of your username cookie was: Hurl Cookie"
body contains "It has now been removed"
body not contains "An error occurred"

GET http://localhost:8080/
HTTP 200
[Asserts]
body not contains "Hurl Cookie"
body not contains "An error occurred"
