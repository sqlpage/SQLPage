---
rows:
  - component: csv
    title: Download my data
    filename: people
    icon: file-download
    color: green
    separator: ;
    bom: true
  - Forename: Ophir
    Surname: Lojkine
    Pseudonym: lovasoa
  - Forename: Linus
    Surname: Torvalds
    Pseudonym: torvalds
---

### CSV download button

This will generate a button to download the CSV file.
The button element itself will embed the entire contents of the CSV file, so it should not be used for large files.
The file will be entirely loaded in memory on the user's browser, even if the user does not click on the button.
For smaller files, this is easier and faster to use than creating a separate SQL file to generate the CSV.
