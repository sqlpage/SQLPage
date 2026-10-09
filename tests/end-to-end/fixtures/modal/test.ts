import { checkModal } from "../../component-assertions.ts";
import { loadFragment, test } from "../../fixture.ts";

test("modal (fragment)", async ({ page }) => {
  await loadFragment(page, "/modal/?fragment=1");
  await checkModal(page);
});
