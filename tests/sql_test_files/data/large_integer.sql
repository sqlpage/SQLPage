-- SQLPage evaluates literal-only SELECTs itself, so + 0 forces database execution.
-- Keep the expected value as text so it cannot overflow in the integer decoder.
select '3000000000' as expected_contains, 3000000000 + 0 as actual;
