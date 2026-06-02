GET http://127.0.0.1:8080/
HTTP 200
[Asserts]
xpath "string(//form/@action)" == "email.sql"
xpath "string(//input[@name='to_email']/@value)" == "recipient@example.com"
xpath "string(//input[@name='subject']/@value)" == "Test email"
xpath "string(//textarea[@name='message_text'])" == "This is a test email"
