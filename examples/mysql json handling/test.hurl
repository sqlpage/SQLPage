GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
xpath "//html" exists
xpath "//body" exists
body not contains "An error occurred"

POST http://localhost:8080/
Content-Type: application/x-www-form-urlencoded
[FormParams]
Name: Hurl Analysts
HTTP 200
[Asserts]
body contains "Hurl Analysts"
body not contains "An error occurred"

POST http://localhost:8080/
Content-Type: application/x-www-form-urlencoded
[FormParams]
Name: Hurl Reviewers
HTTP 200
[Captures]
analysts_id: xpath "string(//select[@name='Memberships[]']/option[normalize-space(.)='Hurl Analysts']/@value)"
reviewers_id: xpath "string(//select[@name='Memberships[]']/option[normalize-space(.)='Hurl Reviewers']/@value)"
[Asserts]
body contains "Hurl Reviewers"
body not contains "An error occurred"

POST http://localhost:8080/
Content-Type: application/x-www-form-urlencoded
`UserName=Hurl+User&Memberships%5B%5D={{analysts_id}}&Memberships%5B%5D={{reviewers_id}}`
HTTP 200
[Asserts]
body contains "Hurl User"
body htmlUnescape contains "Hurl Analysts,Hurl Reviewers"
body not contains "An error occurred"
