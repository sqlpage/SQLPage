import { expect, test } from "../../fixture";

for (const theme of ["light", "dark"]) {
  test(`colored values match their cards in the ${theme} theme`, async ({
    page,
  }) => {
    await page.locator("html").evaluate((html, selectedTheme) => {
      html.dataset.bsTheme = selectedTheme;
    }, theme);

    for (const { id, linked } of [
      { id: "plain-color", linked: false },
      { id: "linked-color", linked: true },
    ]) {
      const styles = await page.locator(`#${id}`).evaluate((item) => ({
        card: getComputedStyle(item.querySelector(".card") as HTMLElement)
          .color,
        verticalSpacing: getComputedStyle(
          item.querySelector(".card") as HTMLElement,
        )
          .getPropertyValue("--tblr-card-spacer-y")
          .trim(),
        value: getComputedStyle(item.querySelector(".h1") as HTMLElement).color,
        link: item.querySelector(".h1 a")
          ? getComputedStyle(item.querySelector(".h1 a") as HTMLElement).color
          : null,
      }));

      expect(styles.value).toBe(styles.card);
      expect(styles.link).toBe(linked ? styles.card : null);
      expect(styles.verticalSpacing).toBe("1rem");
    }
  });
}
