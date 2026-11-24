---
icon: download
category: http-responses
summary: Export query rows as a downloadable CSV file.
top:
  - name: bom
    type: BOOLEAN
    description: Whether to include a Byte Order Mark (a special character indicating the character encoding)
      at the beginning of the file. This is useful for Excel compatibility.
  - class
  - name: color
    type: COLOR
    description: Color of the button. Ignored when used as a header component.
  - name: filename
    type: TEXT
    description: The name of the file that should be downloaded (without the extension).
  - name: icon
    type: ICON
    description: Name of the icon (from tabler-icons.io) to display in the button. Ignored when used as a header
      component.
  - id
  - name: separator
    type: TEXT
    description: How individual values should be separated in the CSV. "," by default, set it to "\t" for tab-separated
      values.
  - name: size
    type: TEXT
    description: The size of the button (e.g., sm, lg). Ignored when used as a header component.
  - name: title
    type: TEXT
    required: true
    description: The text displayed on the download button.
---

Lets the user download data as a CSV file.
Each column from the items in the component will map to a column in the resulting CSV.

When `csv` is used as a **header component** (without a [shell](?component=shell)), it will trigger a download of the CSV file directly on page load.
If the csv file to download is large, we recommend using this approach.

When used inside a page (after calling the shell component), this will add a button to the page that lets the user download the CSV file.
The button will need to load the entire contents of the CSV file in memory, inside the browser, even if the user does not click on it.
If the csv file to download is large, we recommend using this component without a shell component in order to efficiently stream the data to the browser.
