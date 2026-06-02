GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Toggle Light / Dark Mode"
body contains "Categories"
body contains "☀"
body not contains "An error occurred"

GET http://localhost:8080/toggle.sql
Referer: http://localhost:8080/
HTTP 302
[Asserts]
header "Set-Cookie" contains "lightdarkstatus=dark"
header "Location" == "http://localhost:8080/"

GET http://localhost:8080/biography.sql
HTTP 200
[Asserts]
body contains "Biography"
body contains "Morbi fermentum"
body contains "data-bs-theme=\"dark\""
body not contains "An error occurred"

GET http://localhost:8080/toggle.sql
Referer: http://localhost:8080/biography.sql
HTTP 302
[Asserts]
header "Set-Cookie" contains "lightdarkstatus="
header "Location" == "http://localhost:8080/biography.sql"
