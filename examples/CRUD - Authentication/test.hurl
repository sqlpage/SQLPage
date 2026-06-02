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

POST http://localhost:8080/create_session.sql?path=/currencies_list.sql
[FormParams]
username: admin
password: wrong
HTTP 302
[Asserts]
header "Location" contains "login.sql"
header "Location" contains "error=1"

GET http://localhost:8080/currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "/login.sql?path=/currencies_list.sql"

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

POST http://localhost:8080/currencies_item_dml.sql?path=currencies_list.sql&action=UPDATE
[FormParams]
id:
name: HURLTEST
to_rub: 7.89
HTTP 302
[Captures]
currency_id: header "Location" regex "id=(\\d+)"
[Asserts]
header "Location" contains "currencies_list.sql"
header "Location" contains "info=INSERT"

GET http://localhost:8080/currencies_list.sql
HTTP 200
[Asserts]
body contains "HURLTEST"
body contains "7.89"
body htmlUnescape contains "currencies_item_form.sql?&path=/currencies_list.sql&id={{currency_id}}"
body not contains "An error occurred"

POST http://localhost:8080/currencies_item_dml.sql?path=currencies_list.sql&action=UPDATE
[FormParams]
id: {{currency_id}}
name: HURLEDITED
to_rub: 8.91
HTTP 302
[Asserts]
header "Location" contains "currencies_list.sql"
header "Location" contains "id={{currency_id}}"
header "Location" contains "info=UPDATE"

GET http://localhost:8080/currencies_item_form.sql?id={{currency_id}}
HTTP 200
[Asserts]
body contains "HURLEDITED"
body contains "8.91"
body not contains "An error occurred"

POST http://localhost:8080/currencies_item_dml.sql?path=currencies_list.sql&action=DELETE
[FormParams]
id: {{currency_id}}
name: HURLEDITED
to_rub: 8.91
HTTP 302
[Asserts]
header "Location" contains "currencies_list.sql"
header "Location" contains "info=DELETE"

GET http://localhost:8080/currencies_list.sql
HTTP 200
[Asserts]
body not contains "HURLEDITED"
body not contains "An error occurred"

GET http://localhost:8080/logout.sql?path=currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "currencies_list.sql"

GET http://localhost:8080/currencies_list.sql
HTTP 302
[Asserts]
header "Location" == "/login.sql?path=/currencies_list.sql"
