# Run from an elevated Windows PowerShell prompt, or on the Windows CI runner.
param([string]$Binary = "$PSScriptRoot\..\target\debug\sqlpage.exe")
$ErrorActionPreference = 'Stop'
$Binary = (Resolve-Path $Binary).Path
$name = 'SQLPageTest' + [Guid]::NewGuid().ToString('N')
$root = Join-Path ([IO.Path]::GetTempPath()) ("SQLPage service test " + $name)
$created = $false
$listener = $null
$client = $null

function Wait-State([string]$state) {
    $service = Get-Service $name
    $service.WaitForStatus($state, [TimeSpan]::FromSeconds(60))
}

try {
    New-Item -ItemType Directory -Path (Join-Path $root 'sqlpage') -Force | Out-Null
    if (-not [Diagnostics.EventLog]::SourceExists('SQLPage')) {
        New-EventLog -LogName Application -Source SQLPage
    }
    # Reserve a port to exercise startup failure before allowing a successful start.
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = $listener.LocalEndpoint.Port
    $configuration = @{ listen_on = "127.0.0.1:$port"; database_url = 'sqlite::memory:' } |
        ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $root 'sqlpage\sqlpage.json'), $configuration)
    [IO.File]::WriteAllText((Join-Path $root 'index.sql'), "SELECT 'text' AS component, 'service ready' AS contents;")
    $command = '"{0}" --service {1} --web-root "{2}"' -f $Binary, $name, $root
    New-Service -Name $name -BinaryPathName $command -StartupType Manual | Out-Null
    $created = $true
    $failed = $false
    try { Start-Service $name } catch { $failed = $true }
    if (-not $failed) { throw 'A port conflict must fail service startup' }
    Wait-State 'Stopped'
    $status = Get-CimInstance Win32_Service -Filter "Name='$name'"
    if ($status.ServiceSpecificExitCode -ne 1) { throw 'Startup failure was not reported to SCM' }
    $listener.Stop()

    Start-Service $name
    Wait-State 'Running'
    $response = Invoke-WebRequest "http://127.0.0.1:$port/" -UseBasicParsing
    if ($response.Content -notmatch 'service ready') { throw 'The service did not use its web root' }

    # Exercise STOP while a real SQL request is waiting on an HTTP fetch.
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $upstreamPort = $listener.LocalEndpoint.Port
    [IO.File]::WriteAllText((Join-Path $root 'slow.sql'), "SELECT 'text' AS component, sqlpage.fetch('http://127.0.0.1:$upstreamPort/') AS contents;")
    Add-Type -AssemblyName System.Net.Http
    $client = [Net.Http.HttpClient]::new()
    $pendingResponse = $client.GetStringAsync("http://127.0.0.1:$port/slow.sql")
    $accepted = $listener.AcceptTcpClientAsync()
    if (-not $accepted.Wait(20000)) { throw 'SQL request did not reach the upstream server' }
    $upstream = $accepted.Result
    $stream = $upstream.GetStream()
    $stream.ReadTimeout = 20000
    $headers = New-Object byte[] 4096
    if ($stream.Read($headers, 0, $headers.Length) -eq 0) { throw 'Missing upstream request' }
    sc.exe stop $name | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'SCM rejected STOP' }
    Wait-State 'StopPending'
    $bytes = [Text.Encoding]::ASCII.GetBytes("HTTP/1.1 200 OK`r`nContent-Length: 16`r`nConnection: close`r`n`r`nrequest finished")
    $stream.Write($bytes, 0, $bytes.Length)
    $upstream.Dispose()
    if (-not $pendingResponse.Wait(20000)) { throw 'Active request did not finish' }
    if ($pendingResponse.Result -notmatch 'request finished') { throw 'Active response was truncated' }
    Wait-State 'Stopped'
    $status = Get-CimInstance Win32_Service -Filter "Name='$name'"
    if ($status.ExitCode -ne 0) { throw 'Clean stop reported a failure' }
    Start-Service $name
    Wait-State 'Running'
    Stop-Service $name
    Wait-State 'Stopped'
    Write-Host 'Windows service startup failure, readiness, graceful stop, and restart passed.'
} finally {
    if ($client) { $client.Dispose() }
    if ($listener) { $listener.Stop() }
    if ($created) {
        Stop-Service $name -ErrorAction SilentlyContinue
        sc.exe delete $name | Out-Null
    }
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
