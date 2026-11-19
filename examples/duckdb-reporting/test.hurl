GET http://localhost:8080/?region=North
HTTP 200
[Asserts]
header "Content-Type" contains "application/json"
jsonpath "$" count == 18
jsonpath "$[0].sale_id" == 20
jsonpath "$[1].sale_id" == 19
jsonpath "$[2].sale_id" == 18
jsonpath "$[3].sale_id" == 17
jsonpath "$[4].sale_id" == 16
jsonpath "$[5].sale_id" == 15
jsonpath "$[6].sale_id" == 14
jsonpath "$[7].sale_id" == 13
jsonpath "$[8].sale_id" == 12
jsonpath "$[9].sale_id" == 11
jsonpath "$[10].sale_id" == 9
jsonpath "$[11].sale_id" == 8
jsonpath "$[12].sale_id" == 7
jsonpath "$[13].sale_id" == 6
jsonpath "$[14].sale_id" == 5
jsonpath "$[15].sale_id" == 3
jsonpath "$[16].sale_id" == 2
jsonpath "$[17].sale_id" == 1
jsonpath "$[0].amount" == 1234567890.12
jsonpath "$[3].amount" == null
# Check the amounts on either side of the baseline driver's 16-row boundary.
jsonpath "$[15].amount" == -8.75
jsonpath "$[16].amount" == null
jsonpath "$[17].amount" == 3.5

GET http://localhost:8080/?region=South
HTTP 200
[Asserts]
jsonpath "$" count == 2
jsonpath "$[0].sale_id" == 10
jsonpath "$[0].amount" == 0
jsonpath "$[1].sale_id" == 4
jsonpath "$[1].amount" == 90.01
