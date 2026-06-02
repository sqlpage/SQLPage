# Public homepage is reachable without a session.
GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Demo/Template CRUD with Authentication"
body not contains "An error occurred"

# The currencies CRUD page is protected.
GET http://localhost:8080/currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "/login.sql?path=/currencies_list.sql"

GET http://localhost:8080/login.sql?path=/currencies_list.sql
HTTP 200
[Asserts]
body contains "Login"
body contains "Username"
body contains "Password"
body not contains "An error occurred"

# Demo credentials from the example migration: admin/admin.
POST http://localhost:8080/create_session.sql?path=/currencies_list.sql
[FormParams]
username: admin
password: admin
HTTP 302
[Asserts]
header "Set-Cookie" contains "session_token="
header "Location" == "/currencies_list.sql"

# Hurl's cookie store reuses the session_token set above.
GET http://localhost:8080/currencies_list.sql
HTTP 200
[Asserts]
body contains "Currencies"
body contains "New Record"
body contains "RUR"
body contains "USD"
body not contains "An error occurred"

GET http://localhost:8080/currencies_item_form.sql?id=1
HTTP 200
[Asserts]
body contains "Currency"
body contains "Exchange Rate to RUR"
body contains "RUR"
body not contains "An error occurred"

GET http://localhost:8080/currencies_item_form.sql?action=INSERT
HTTP 200
[Asserts]
body contains "Currency"
body contains "Exchange Rate to RUR"
body contains "BROWSE"
body not contains "An error occurred"

GET http://localhost:8080/logout.sql?path=currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "currencies_list.sql"

GET http://localhost:8080/currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "/login.sql?path=/currencies_list.sql"
