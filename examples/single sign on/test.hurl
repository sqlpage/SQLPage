GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Welcome"
body contains "Log in"
body htmlUnescape contains "/protected"
body not contains "An error occurred"

GET http://localhost:8080/protected
HTTP 303
[Captures]
login_url: header "Location"
[Asserts]
header "Location" contains "http://localhost:8181/realms/sqlpage_demo/protocol/openid-connect/auth"

GET {{login_url}}
HTTP 200
[Captures]
login_action: xpath "string(//form[@id='kc-form-login']/@action)"
[Asserts]
xpath "//form[@id='kc-form-login']" exists
xpath "//input[@name='username']" exists
xpath "//input[@name='password']" exists
body htmlUnescape contains "client_id=sqlpage"

POST {{login_action}}
[FormParams]
username: demo
password: demo
credentialId:
HTTP 302
[Captures]
callback_url: header "Location"
[Asserts]
header "Location" contains "http://localhost:8080/sqlpage/oidc_callback"

GET {{callback_url}}
HTTP 303
[Asserts]
header "Location" == "/protected"

GET http://localhost:8080/protected/
HTTP 200
[Asserts]
body htmlUnescape contains "You're in, John Smith"
body contains "demo@example.com"
body contains "You have access to this protected page."
body htmlUnescape contains "/assets/welcome.jpeg"
body not contains "An error occurred"
