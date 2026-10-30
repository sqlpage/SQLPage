import { expect, type Page, test } from "../../../fixture.ts";

async function heroLayout(page: Page, width: number) {
  await page.setViewportSize({ width, height: 900 });
  const image = await page
    .getByRole("img", { name: "Reversed hero" })
    .boundingBox();
  const title = await page
    .getByRole("heading", { name: "Reversed hero" })
    .boundingBox();
  if (!image || !title) throw new Error("the hero is not visible");
  return { image, title };
}

test("a reversed hero shows the image above the title on phones", async ({
  page,
}) => {
  const { image, title } = await heroLayout(page, 390);
  expect(image.y + image.height).toBeLessThanOrEqual(title.y);
});

test("a reversed hero shows the image left of the title on desktops", async ({
  page,
}) => {
  const { image, title } = await heroLayout(page, 1400);
  expect(image.x + image.width).toBeLessThanOrEqual(title.x);
});
