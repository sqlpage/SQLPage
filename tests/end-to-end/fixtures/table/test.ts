import {
  checkTableFiltering,
  checkTableSort,
} from "../../component-assertions.ts";
import { loadFragment, test } from "../../fixture.ts";

test("table filtering (fragment)", async ({ page }) => {
  await loadFragment(page, "/table/?fragment=1", null);
  await checkTableFiltering(page, true);
});

for (const [name, column, reverse] of [
  ["table sorts a column when its header is clicked", "id", false],
  ["table reverses the sort when the header is clicked again", "id", true],
  [
    "table sorts a column of formatted numbers by value",
    "Amount in stock",
    false,
  ],
] as const) {
  test(`${name} (fragment)`, async ({ page }) => {
    await loadFragment(page, "/table/?fragment=1", "document");
    await checkTableSort(page, column, reverse);
  });
}
