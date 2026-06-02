GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Recent blog posts"
body contains "Write a post !"
body not contains "An error occurred"

GET http://localhost:8080/write.sql
HTTP 200
[Asserts]
body contains "Title of the post"
body contains "Main Topic"
body contains "Technology"
body contains "Travel"
body not contains "An error occurred"

POST http://localhost:8080/write_submit.sql
Content-Type: application/x-www-form-urlencoded
`Title=Hurl%20many-to-many%20post&Content=Hurl%20content%20for%20the%20many-to-many%20form.&Main%20Topic=1&Topics%5B%5D=2&Topics%5B%5D=3`
HTTP 302
[Captures]
location: header "Location"
[Asserts]
header "Location" matches "^post\\.sql\\?id=\\d+$"

GET http://localhost:8080/{{location}}
HTTP 200
[Asserts]
body contains "Hurl content for the many-to-many form."
body contains "Technology"
body contains "Travel"
body contains "Food"
body not contains "An error occurred"
