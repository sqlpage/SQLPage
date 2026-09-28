# Run SQLPage as a service

SQLPage can start automatically at boot and run without an open terminal. It reports
readiness after connecting to the database, applying migrations, initializing the
application, and starting its HTTP listeners and workers. A startup failure is
reported to the service manager so it can apply the configured recovery policy.

When stopped, SQLPage stops accepting new connections and gives active HTTP requests
up to 30 seconds to finish, then closes database connections and flushes telemetry.
Allow extra time for database cleanup and telemetry when configuring service stop
timeouts. Requests that exceed the drain timeout can be interrupted.

## Linux with systemd

Install the Linux executable as `/usr/local/bin/sqlpage.bin`, create a dedicated
`sqlpage` user and group, and put your application in `/var/www/sqlpage`. The account
needs read access to the application and write access to its database, uploads, and
configuration directory as appropriate.

Download the [provided systemd unit](https://github.com/sqlpage/SQLPage/blob/main/sqlpage.service)
to `/etc/systemd/system/sqlpage.service`, adjusting `User`, `Group`, `WorkingDirectory`,
`ExecStart`, and `LISTEN_ON` to match your installation. The example listens on port
80 and grants only the capability needed to bind a privileged port. For port 8080,
change `LISTEN_ON` and remove `AmbientCapabilities`.

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now sqlpage
sudo systemctl status sqlpage
journalctl -u sqlpage -f
```

The unit uses `Type=notify`: `systemctl start` waits for SQLPage's readiness
notification. Startup has a five-minute timeout to allow for migrations. Change
`TimeoutStartSec` if your deployment needs more time. `systemctl stop sqlpage` sends
SIGTERM, which initiates graceful shutdown. The example allows 60 seconds for the
whole stop operation and restarts the process on failure. SIGINT and SIGQUIT also
initiate graceful shutdown when running from a terminal or another supervisor.

Logs go to the journal. Use `sqlpage/sqlpage.json`, a `.env` file in the working
directory, or systemd `Environment`/`EnvironmentFile` settings for configuration.
SQLPage stays in the foreground; no PID file or daemonization is needed.

## Windows Service Control Manager

SQLPage supports native Windows services without a wrapper. Download `sqlpage.exe`,
then prepare an application folder such as `C:\SQLPage\website`, with its
configuration in `C:\SQLPage\website\sqlpage\sqlpage.json`.

In **Windows PowerShell run as Administrator**, register the event-log source and
create the service (adjust the paths):

```powershell
New-EventLog -LogName Application -Source SQLPage
$binary = 'C:\SQLPage\sqlpage.exe'
$root = 'C:\SQLPage\website'
$command = '"{0}" --service SQLPage --web-root "{1}"' -f $binary, $root
New-Service -Name SQLPage -BinaryPathName $command -StartupType Automatic `
    -DisplayName 'SQLPage website'
```

Skip `New-EventLog` if the `SQLPage` source is already registered. It is shared by
all SQLPage services. This command is available in Windows PowerShell 5.1.

Before starting, open `services.msc`, find **SQLPage website**, and set its **Log On**
account to a dedicated service account with access to your application, database,
and uploads. `New-Service` defaults to LocalSystem; choose an account with only the
permissions your application needs. Configure automatic recovery on the **Recovery**
tab if desired, including recovery for non-crash failures.

```powershell
Start-Service SQLPage
Get-Service SQLPage
Get-WinEvent -FilterHashtable @{ LogName = 'Application'; ProviderName = 'SQLPage' } -MaxEvents 20
Restart-Service SQLPage
Stop-Service SQLPage
```

`--service NAME` must match the registered service name and requires an **absolute**
`--web-root`. In service mode this directory is also the working directory, so `.env`,
relative configuration paths, SQLite files, and uploads resolve there instead of
Windows' system directory. You may also pass `--config-dir` or `--config-file` in the
service command. Use distinct names and listening ports to run multiple services.

SQLPage reports `START_PENDING` during initialization, `RUNNING` once ready,
`STOP_PENDING` while draining requests, and `STOPPED` after cleanup. Startup progress
updates carry a five-minute wait hint. Both service-stop and operating-system
shutdown controls initiate cleanup. Failures report service-specific exit code 1;
details and application logs appear in **Event Viewer → Windows Logs → Application**
under the **SQLPage** source. Existing OpenTelemetry export remains available.

To remove the service:

```powershell
Stop-Service SQLPage
sc.exe delete SQLPage
```

Running `sqlpage.exe` without `--service` continues to run in a terminal, with normal
console logging and graceful shutdown on Ctrl+C or Ctrl+Break. To diagnose a service
configuration, open a terminal in its web root and run the same command without
`--service NAME`.
