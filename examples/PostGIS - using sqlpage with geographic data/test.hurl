GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Points of interest"
body contains "Add a point"
body not contains "An error occurred"

POST http://localhost:8080/add_point.sql
[FormParams]
Title: Hurl Paris
Latitude: 48.8566
Longitude: 2.3522
Text: First Hurl point
HTTP 302
[Asserts]
header "Location" == "index.sql"

POST http://localhost:8080/add_point.sql
[FormParams]
Title: Hurl Lyon
Latitude: 45.764
Longitude: 4.8357
Text: Second Hurl point
HTTP 302

GET http://localhost:8080/
HTTP 200
[Captures]
paris_id: xpath "substring-after(string(//a[normalize-space(.)='Hurl Paris']/@href), 'id=')"
[Asserts]
body contains "Hurl Paris"
body contains "Hurl Lyon"
body htmlUnescape contains "point.sql?id={{paris_id}}"
body not contains "An error occurred"

GET http://localhost:8080/point.sql?id={{paris_id}}
HTTP 200
[Asserts]
body contains "48.8566"
body contains "2.3522"
body contains "First Hurl point"
body contains "Closest points"
body contains "Hurl Lyon"
body htmlUnescape contains "edition_form.sql?id={{paris_id}}"
body not contains "An error occurred"

POST http://localhost:8080/edition_form.sql?id={{paris_id}}
[FormParams]
Text: Updated Hurl point
HTTP 200
[Asserts]
body contains "Updated Hurl point"
body not contains "An error occurred"
