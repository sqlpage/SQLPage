GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "SQLPage Form Demo"
body contains "Users"
body contains "Add a new user"
body not contains "An error occurred"

POST http://localhost:8080/insert_user.sql?id=987654
Content-Type: application/x-www-form-urlencoded
`First%20name=Hurl&Last%20name=Masterdetail&Email=hurl-masterdetail%40example.com`
HTTP 302
[Asserts]
header "Location" == "edit_user.sql?id=987654"

GET http://localhost:8080/edit_user.sql?id=987654
HTTP 200
[Asserts]
body contains "Edit user"
body contains "Hurl"
body contains "Masterdetail"
body contains "hurl-masterdetail@example.com"
body contains "Addresses"
body contains "Add address"
body not contains "An error occurred"

POST http://localhost:8080/insert_address.sql?user_id=987654
Content-Type: application/x-www-form-urlencoded
`Street=42%20Hurl%20Street&City=Testville&Country=Exampleland`
HTTP 302
[Asserts]
header "Location" == "edit_user.sql?id=987654"

GET http://localhost:8080/edit_user.sql?id=987654
HTTP 200
[Asserts]
body contains "Hurl"
body contains "Masterdetail"
body contains "42 Hurl Street, Testville, Exampleland"
body not contains "An error occurred"

POST http://localhost:8080/insert_user.sql?id=987654
Content-Type: application/x-www-form-urlencoded
`First%20name=Hurl&Last%20name=Updated&Email=hurl-updated-masterdetail%40example.com`
HTTP 302
[Asserts]
header "Location" == "edit_user.sql?id=987654"

GET http://localhost:8080/edit_user.sql?id=987654
HTTP 200
[Asserts]
body contains "Hurl"
body contains "Updated"
body contains "hurl-updated-masterdetail@example.com"
body contains "42 Hurl Street, Testville, Exampleland"
body not contains "Masterdetail"
body not contains "An error occurred"

GET http://localhost:8080
HTTP 200
[Asserts]
body contains "Hurl Updated"
body contains "hurl-updated-masterdetail@example.com"
body not contains "An error occurred"
