GET http://localhost:8080
HTTP 200
[Asserts]
body contains "SQLPage Authentication Demo"
body contains "Access protected page"
body not contains "An error occurred"

GET http://localhost:8080/signin.sql
HTTP 200
[Asserts]
body contains "Sign in"
body contains "Username"
body contains "Password"

GET http://localhost:8080/protected_page.sql
HTTP 302
[Asserts]
header "Location" contains "signin.sql?error"

GET http://localhost:8080/signin.sql?error
HTTP 200
[Asserts]
body contains "We could not authenticate you"
body htmlUnescape contains "href=\"signup.sql\""

POST http://localhost:8080/create_user.sql
[FormParams]
username: hurl_user
password: Password123
terms: on
HTTP 302
[Asserts]
header "Location" contains "create_user_welcome_message.sql?username=hurl_user"

POST http://localhost:8080/login.sql
[FormParams]
username: hurl_user
password: Password123
HTTP 302
[Asserts]
cookie "session" exists
header "Location" contains "protected_page.sql"

GET http://localhost:8080/protected_page.sql
HTTP 200
[Asserts]
body contains "Welcome, hurl_user"
body contains "top secret"
body not contains "An error occurred"

GET http://localhost:8080/logout.sql
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080/protected_page.sql
HTTP 302
[Asserts]
header "Location" contains "signin.sql?error"
