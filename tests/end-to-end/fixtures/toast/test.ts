import { checkToastNotifications } from "../../component-assertions.ts";
import { loadFragment, test } from "../../fixture.ts";

test("toast notifications initialize, stack, dismiss, and render safely (fragment)", async ({
  page,
}) => {
  await loadFragment(page, "/toast/?fragment=1");
  await checkToastNotifications(page);
});
