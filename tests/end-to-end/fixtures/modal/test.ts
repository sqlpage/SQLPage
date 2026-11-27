import { checkModal } from "../../component-assertions.ts";
import { expect, loadFragment, test } from "../../fixture.ts";

test("modal (fragment)", async ({ page }) => {
  await loadFragment(page, "/modal/?fragment=1");
  await checkModal(page);
  await expect(
    page.locator("#my_embed_form_modal").getByLabel("Modern text field"),
  ).toBeAttached();
});
