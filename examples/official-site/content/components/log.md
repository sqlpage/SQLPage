---
icon: logs
since: 0.37.1
category: server-logic
summary: Write a message to the SQLPage server log.
top:
  - name: level
    type: TEXT
    description: How important the message is. One of 'trace', 'debug', 'info' (default), 'warn', 'error'.
      Not case-sensitive. Controls the level shown in the logs.
  - name: message
    type: TEXT
    required: true
    description: The text to write to the server logs. It is printed when the page runs.
---

A component that writes messages to the server logs.
When a page runs, it prints your message to the terminal/console (standard error).
Use it to track what happens and troubleshoot issues.

## Reading log messages

- Running from a terminal (Linux, macOS, or Windows PowerShell/Command Prompt): they show up in the window.
- Docker: run `docker logs <container_name>`.
- Linux service (systemd): run `journalctl -u sqlpage`.
- This component's output is written to [standard error (stderr)](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)). SQLPage request access logs are separate and are written to standard output (stdout).
