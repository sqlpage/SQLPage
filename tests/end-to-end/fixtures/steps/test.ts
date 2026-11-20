import { expect, test } from "../../fixture.ts";

test("the step icon aligns with the capital letters of its title", async ({
  page,
}) => {
  const icon = page.getByRole("link", { name: "Review" }).locator("svg");
  const offset = await icon.evaluate((svg) => {
    // An empty inline-block sits on the text baseline. Its top is one cap height above it.
    const capital = document.createElement("span");
    capital.style.cssText = "display: inline-block; height: 1cap";
    svg.after(capital);
    const centerY = (element: Element) => {
      const { top, height } = element.getBoundingClientRect();
      return top + height / 2;
    };
    const delta = centerY(svg) - centerY(capital);
    capital.remove();
    return delta;
  });
  expect(Math.abs(offset)).toBeLessThanOrEqual(1);
});
