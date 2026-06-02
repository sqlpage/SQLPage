GET http://localhost:8080/
HTTP 200
[Asserts]
xpath "count(//div[contains(@class, 'card')])" >= 4
body contains "A card with a Markdown description"
body contains "<strong>Markdown</strong>"
body htmlUnescape contains "/chart-example.sql?_sqlpage_embed"
body htmlUnescape contains "/map-example.sql?_sqlpage_embed"
body htmlUnescape contains "/table-example.sql?_sqlpage_embed"
body not contains "An error occurred"

GET http://localhost:8080/chart-example.sql?_sqlpage_embed
HTTP 200
[Asserts]
xpath "normalize-space(//*[contains(., 'Quarterly Revenue')])" contains "Quarterly Revenue"
body contains "2023-07-01T00:00:00Z"
body not contains "An error occurred"

GET http://localhost:8080/map-example.sql?_sqlpage_embed
HTTP 200
[Asserts]
body contains "New Delhi"
body contains "28.6139"
body not contains "An error occurred"

GET http://localhost:8080/table-example.sql?_sqlpage_embed
HTTP 200
[Asserts]
xpath "normalize-space(//*[contains(., 'Forename')])" contains "Forename"
body contains "Ophir"
body contains "Torvalds"
body not contains "An error occurred"
