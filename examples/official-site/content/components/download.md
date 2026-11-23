---
icon: download
since: 0.37.0
category: http-responses
summary: Send a file directly to the visitor.
top:
  - name: data_url
    type: TEXT
    required: true
    description: 'The file content to send, written as a data URL (for example: data:text/plain,Hello%20world
      or data:application/octet-stream;base64,SGVsbG8h). The part before the comma declares the content type
      and whether the data is base64-encoded. The part after the comma is the actual data.'
  - name: filename
    type: TEXT
    description: 'The suggested name of the file to save (for example: report.csv). When set, the browser will
      download the file as an attachment with this name. When omitted, many browsers may try to display the
      file inline depending on its content type.'
---

The *download* component lets a page immediately return a file to the visitor.

Instead of showing a web page, it sends the file's bytes as the whole response,
so it should be used **at the very top of your SQL page** (before the shell or any other page contents).
It is an error to use this component after another component that would display content.

## Sending a file

- You provide the file content using a [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs).
A data URL is just a text string that contains both the file type and the actual data.
- Optionally, you provide a "filename" so the browser shows a proper Save As name.
If you do not provide a filename, many browsers will try to display the file inline (for example images or JSON), depending on the content type.
- You link to the page that uses the download component from another page, using the [button](/component.sql?component=button) component for example.

## Data URL format

- It looks like this: `data:[content-type][;base64],DATA`
- Examples:
  - Plain text (URL-encoded): `data:text/plain,Hello%20world`
  - JSON (URL-encoded): `data:application/json,%7B%22message%22%3A%22Hi%22%7D`
  - Binary data (Base64): `data:application/octet-stream;base64,SGVsbG8h`

Tips:
- Use URL encoding when you have textual data. You can use [`sqlpage.url_encode(source_text)`](/functions?function=url_encode) to encode the data.
- Use Base64 when you have binary data (images, PDFs, or content that may include special characters).
- Use [`sqlpage.read_file_as_data_url(file_path)`](/functions?function=read_file_as_data_url) to read a file from the server and return it as a data URL.

> Keep in mind that large files are better served from disk or object storage. Data URLs are best for small to medium files.
There is a big performance penalty for loading large files as data URLs, so it is not recommended.
