# Public gallery is reachable without a session.
GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "My image gallery"
body contains "Upload a new image"
body not contains "An error occurred"

# Upload form is protected.
GET http://localhost:8080/upload_form.sql
HTTP 302
[Asserts]
header "Location" == "/login.sql"

GET http://localhost:8080/login.sql
HTTP 200
[Asserts]
body contains "Login"
body contains "Username"
body contains "Password"
body not contains "An error occurred"

# Demo credentials from the example migration: admin/admin.
POST http://localhost:8080/create_session.sql
[Form]
Username: admin
Password: admin
HTTP 302
[Asserts]
header "Set-Cookie" contains "session_token="
header "Location" == "/"

# Hurl's cookie store reuses the session_token set above.
GET http://localhost:8080/upload_form.sql
HTTP 200
[Asserts]
body contains "Upload a new image"
body contains "Title"
body contains "Description"
body contains "Image"
body not contains "An error occurred"

POST http://localhost:8080/upload.sql
[Multipart]
Title: Hurl test image
Description: Uploaded by Hurl
Image: file,screenshots/upload.png; image/png
HTTP 302
[Captures]
location: header "Location"
[Asserts]
header "Location" matches "^/\\?created_id=\\d+$"

GET http://localhost:8080{{location}}
HTTP 200
[Asserts]
body contains "Hurl test image"
body contains "Uploaded by Hurl"
body contains "/images/"
body not contains "An error occurred"
