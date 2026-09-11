import { expect, test } from "../../fixture";

const fields = [
  { selector: 'input[name="modern_text"]', name: "text" },
  { selector: 'textarea[name="modern_textarea"]', name: "textarea" },
  { selector: 'select[name="modern_select"]', name: "select" },
  { selector: 'input[name="legacy_radio"]', name: "radio" },
  { selector: 'input[name="legacy_checkbox"]', name: "checkbox" },
  { selector: 'input[name="modern_switch"]', name: "switch" },
];

test("renders markdown descriptions for every form field layout (#808)", async ({
  page,
}) => {
  for (const { selector, name } of fields) {
    const field = page.locator(selector);
    const hint = field
      .locator("xpath=ancestor::label[1]")
      .locator(".form-hint");

    await expect(field).toBeVisible();
    await expect(hint.locator("strong")).toHaveText(`Bold ${name}`);
    await expect(hint.locator("em")).toHaveText(`italic ${name}`);
  }
});
