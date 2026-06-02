GET http://localhost:8080
HTTP 200
[Asserts]
body contains "TinyTweeter"
body contains "Tell me your story..."
body contains "Terms and conditions"
body not contains "An error occurred"

POST http://localhost:8080
[FormParams]
new_tweet: Hurl tiny twitter post
HTTP 200
[Asserts]
body contains "TinyTweeter"
body contains "Hurl tiny twitter post"
body not contains "An error occurred"
