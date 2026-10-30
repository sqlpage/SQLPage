import { expect, test } from "../../../fixture.ts";

test("only the active colored tab is underlined, in its color", async ({
  page,
}) => {
  const inactive = page.getByRole("link", { name: "Red" });
  const active = page.getByRole("link", { name: "Green" });
  await expect(inactive).toHaveCSS("border-bottom-color", "rgba(0, 0, 0, 0)");
  const activeColor = await active.evaluate(
    (link) => getComputedStyle(link).color,
  );
  await expect(active).toHaveCSS("border-bottom-color", activeColor);
});
