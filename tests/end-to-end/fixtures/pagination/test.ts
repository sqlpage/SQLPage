import { expect, test } from "../../fixture.ts";

test("pagination names its controls and only disables the requested links", async ({
  page,
}) => {
  for (const name of ["First page", "Next page", "Last page"]) {
    const link = page.getByRole("link", { name });
    await expect(link).toBeVisible();
    await expect(link).not.toHaveAttribute("aria-disabled");
    await link.focus();
    await expect(link).toBeFocused();
  }
  const previous = page.getByRole("link", { name: "Previous page" });
  await expect(previous).toHaveAttribute("aria-disabled", "true");
  await expect(previous).toHaveAttribute("tabindex", "-1");
});
