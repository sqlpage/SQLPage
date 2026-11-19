GET http://localhost:8080/
HTTP 200
[Captures]
north_july: xpath "string(//tbody/tr[td[1]='2026-07' and td[2]='North']//a[normalize-space()='View sales']/@href)"
south_july: xpath "string(//tbody/tr[td[1]='2026-07' and td[2]='South']//a[normalize-space()='View sales']/@href)"
[Asserts]
xpath "//h2[normalize-space()='Sales by month and region']" exists
xpath "//tbody/tr" count == 4
xpath "string(//tbody/tr[td[1]='2026-07' and td[2]='North']/td[3])" == "18"
xpath "string(//tbody/tr[td[1]='2026-07' and td[2]='North']/td[4])" == "1662.34"
xpath "string(//tbody/tr[td[1]='2026-08' and td[2]='North']/td[3])" == "2"
xpath "string(//tbody/tr[td[1]='2026-08' and td[2]='North']/td[4])" == "100"

GET http://localhost:8080/{{north_july}}
HTTP 200
[Asserts]
xpath "//h2[normalize-space()='Sales and targets']" exists
xpath "//tbody/tr" count == 18

GET http://localhost:8080/{{north_july}}
Accept: application/json
HTTP 200
[Asserts]
header "Content-Type" contains "application/json"
jsonpath "$[?(@.sale_id)]" count == 18
jsonpath "$[1].sale_id" == 20
jsonpath "$[2].sale_id" == 19
jsonpath "$[3].sale_id" == 18
jsonpath "$[4].sale_id" == 17
jsonpath "$[5].sale_id" == 16
jsonpath "$[6].sale_id" == 15
jsonpath "$[7].sale_id" == 14
jsonpath "$[8].sale_id" == 13
jsonpath "$[9].sale_id" == 12
jsonpath "$[10].sale_id" == 11
jsonpath "$[11].sale_id" == 9
jsonpath "$[12].sale_id" == 8
jsonpath "$[13].sale_id" == 7
jsonpath "$[14].sale_id" == 6
jsonpath "$[15].sale_id" == 5
jsonpath "$[16].sale_id" == 3
jsonpath "$[17].sale_id" == 2
jsonpath "$[18].sale_id" == 1
jsonpath "$[1].sold_on" == "2026-07-20"
jsonpath "$[1].amount" == 1234.56
jsonpath "$[1].target_amount" == 26.75
jsonpath "$[4].amount" == null
jsonpath "$[4].target_amount" == 26.75
# The target changes for sales on July 14.
jsonpath "$[7].target_amount" == 26.75
jsonpath "$[8].target_amount" == 10.50
# Check the amounts and targets on either side of the baseline driver's 16-row boundary.
jsonpath "$[16].amount" == -8.75
jsonpath "$[16].target_amount" == 10.50
jsonpath "$[17].amount" == null
jsonpath "$[17].target_amount" == null
jsonpath "$[18].amount" == 3.5

GET http://localhost:8080/{{south_july}}
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[?(@.sale_id)]" count == 2
jsonpath "$[1].sale_id" == 10
jsonpath "$[1].amount" == 0
jsonpath "$[1].target_amount" == 50
jsonpath "$[2].sale_id" == 4
jsonpath "$[2].amount" == 90.01
jsonpath "$[2].target_amount" == 50
