import { expect, test } from "../../fixture.ts";

test("shows embedded content in the selected cells", async ({ page }) => {
  const moss = page.getByRole("row", { name: /^Moss\b/ });
  await expect(moss.getByRole("cell")).toHaveText(
    ["Moss", "Forest preview", "Grows in shade"],
    { useInnerText: true },
  );

  const pine = page.getByRole("row", { name: /^Pine\b/ });
  await expect(pine.getByRole("cell")).toHaveText(["Pine", "", ""], {
    useInnerText: true,
  });
});
