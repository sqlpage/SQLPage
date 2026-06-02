GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "Welcome to Our Site"
body contains "served by Apache web server"
body contains "href=\"/my_website\""

GET http://localhost:8080/my_website
HTTP 301
[Asserts]
header "Location" == "/my_website/"

GET http://localhost:8080/my_website/
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Welcome to my website"
body contains "Using SQLPage v"
body contains "Connected to"
body contains "MySQL"
