---
rows:
  - component: table
    sort: true
    align_right:
      - Price
      - Amount in stock
    align_center:
      - part_no
    raw_numbers:
      - id
    currency: USD
    money:
      - Price
  - id: 31456
    part_no: SQL-TABLE-856-G
    Price: 12
    Amount in stock: 5
  - id: 996
    part_no: SQL-FORMS-86-M
    Price: 1
    Amount in stock: 1234
  - id: 131456
    part_no: SQL-CARDS-56-K
    Price: 127
    Amount in stock: 98
---

A table with column sorting. Sorting sorts numbers in numeric order, and strings in alphabetical order.

Numbers can be displayed
 - as raw digits without formatting using the `raw_numbers` property,
 - as currency using the `money` property to define columns that contain monetary values and `currency` to define the currency,
 - as numbers with a fixed maximum number of decimal digits using the `number_format_digits` property.
