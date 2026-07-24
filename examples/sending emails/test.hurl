DELETE http://127.0.0.1:8025/api/v1/messages
HTTP 200

GET http://127.0.0.1:8080/
HTTP 200
[Asserts]
xpath "string(//a[contains(., 'Simple email')]/@href)" == "simple.sql"
xpath "string(//a[contains(., 'Advanced email')]/@href)" == "advanced.sql"

GET http://127.0.0.1:8080/simple.sql
HTTP 200
[Asserts]
xpath "string(//form/@action)" == "send-simple.sql"
xpath "string(//form/@method)" == "post"
xpath "string(//input[@name='recipient']/@value)" == "recipient@example.com"
xpath "string(//input[@name='subject']/@value)" == "Simple SMTP demo"
xpath "string(//textarea[@name='body'])" == "Sent with sqlpage.send_mail"

POST http://127.0.0.1:8080/send-simple.sql
[FormParams]
recipient: recipient@example.com
subject: Simple SMTP demo
body: Sent with the configured sender
HTTP 200
[Asserts]
xpath "normalize-space(//*[@id='send-result'])" == "Email sent successfully"
xpath "string(//a[normalize-space(.)='Open Mailpit inbox']/@href)" == "http://localhost:8025"

GET http://127.0.0.1:8025/api/v1/search?query=subject%3A%22Simple%20SMTP%20demo%22
HTTP 200
[Captures]
simple_message_id: jsonpath "$.messages[0].ID"
[Asserts]
jsonpath "$.messages_count" == 1
jsonpath "$.messages[0].Subject" == "Simple SMTP demo"
jsonpath "$.messages[0].To[0].Address" == "recipient@example.com"

GET http://127.0.0.1:8025/api/v1/message/{{simple_message_id}}
HTTP 200
[Asserts]
jsonpath "$.From.Name" == "SQLPage Demo"
jsonpath "$.From.Address" == "sqlpage@example.com"
jsonpath "$.Text" contains "Sent with the configured sender"
jsonpath "$.Attachments" count == 0

GET http://127.0.0.1:8080/advanced.sql
HTTP 200
[Asserts]
xpath "string(//form/@action)" == "send-advanced.sql"
xpath "string(//form/@method)" == "post"
xpath "string(//form//button[@type='submit']/@formenctype)" == "multipart/form-data"
xpath "string(//input[@name='second_recipient']/@value)" == "second@example.com"
xpath "string(//input[@name='cc']/@value)" == "team@example.com"
xpath "string(//input[@name='reply_to']/@value)" == "replies@example.com"
xpath "string(//input[@name='attachment']/@type)" == "file"

POST http://127.0.0.1:8080/send-advanced.sql
[Multipart]
recipient: first@example.com
second_recipient: second@example.com
cc: team@example.com
sender: advanced@example.com
reply_to: replies@example.com
subject: Advanced SMTP demo
body: Sent to a team with an attachment
body_html: <p>Sent to a team with an <strong>attachment</strong></p>
attachment: file,test-attachment.txt; text/plain
HTTP 200
[Asserts]
xpath "normalize-space(//*[@id='send-result'])" == "Email sent successfully"
xpath "string(//a[normalize-space(.)='Open Mailpit inbox']/@href)" == "http://localhost:8025"

GET http://127.0.0.1:8025/api/v1/search?query=subject%3A%22Advanced%20SMTP%20demo%22
HTTP 200
[Captures]
advanced_message_id: jsonpath "$.messages[0].ID"
[Asserts]
jsonpath "$.messages_count" == 1
jsonpath "$.messages[0].Attachments" == 1

GET http://127.0.0.1:8025/api/v1/message/{{advanced_message_id}}
HTTP 200
[Captures]
attachment_part_id: jsonpath "$.Attachments[0].PartID"
[Asserts]
jsonpath "$.Subject" == "Advanced SMTP demo"
jsonpath "$.From.Address" == "advanced@example.com"
jsonpath "$.To[0].Address" == "first@example.com"
jsonpath "$.To[1].Address" == "second@example.com"
jsonpath "$.Cc[0].Address" == "team@example.com"
jsonpath "$.ReplyTo[0].Address" == "replies@example.com"
jsonpath "$.Text" contains "Sent to a team with an attachment"
jsonpath "$.HTML" contains "Sent to a team with an"
jsonpath "$.Attachments[0].FileName" == "test-attachment.txt"
jsonpath "$.Attachments[0].ContentType" == "text/plain"

GET http://127.0.0.1:8025/api/v1/message/{{advanced_message_id}}/part/{{attachment_part_id}}
HTTP 200
[Asserts]
body contains "This attachment was sent by the SQLPage email example."
