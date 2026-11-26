$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$Root = $PSScriptRoot
Set-Location -LiteralPath $Root
$RunLog = Join-Path $Root 'BUILD-AURORA-last-run.log'
$OutputLog = Join-Path $Root 'BUILD-AURORA-output.log'
$Utf8 = New-Object System.Text.UTF8Encoding($false)
$script:NpmCachePath = ''
[System.IO.File]::WriteAllText($RunLog, "Aurora AI build diagnostic log`r`nStarted: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff')`r`nFolder: $Root`r`n", $Utf8)
[System.IO.File]::WriteAllText($OutputLog, "Aurora AI full build output`r`nStarted: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff')`r`n", $Utf8)

function Write-RunLog([string]$Message) {
    [System.IO.File]::AppendAllText($RunLog, $Message + "`r`n", $Utf8)
}
function Write-OutputLog([string]$Message) {
    [System.IO.File]::AppendAllText($OutputLog, $Message + "`r`n", $Utf8)
    Write-Host $Message
}
function Write-ProcessText([string]$Text) {
    if ([string]::IsNullOrEmpty($Text)) { return }
    foreach ($Line in ($Text -split "`r?`n")) {
        if ($Line.Length -gt 0) { Write-OutputLog $Line }
    }
}
function Invoke-NativeStep([string]$Label, [string]$Executable, [string[]]$Arguments, [switch]$UseCmd) {
    Write-Host ''
    Write-Host ('=' * 68) -ForegroundColor DarkCyan
    Write-Host $Label -ForegroundColor Cyan
    Write-Host ('Command: ' + $Executable + ' ' + ($Arguments -join ' ')) -ForegroundColor DarkGray
    Write-Host 'Running. Output will be added to BUILD-AURORA-output.log.' -ForegroundColor DarkGray
    Write-OutputLog "`r`n===== $Label ====="
    Write-OutputLog ('COMMAND: ' + $Executable + ' ' + ($Arguments -join ' '))

    $StartInfo = New-Object System.Diagnostics.ProcessStartInfo
    if ($UseCmd) {
        $StartInfo.FileName = $env:ComSpec
        $CommandLine = 'call "' + $Executable + '" ' + ($Arguments -join ' ')
        $StartInfo.Arguments = '/d /s /c "' + $CommandLine + '"'
    }
    else {
        $StartInfo.FileName = $Executable
        $StartInfo.Arguments = ($Arguments -join ' ')
    }
    $StartInfo.WorkingDirectory = $Root
    $StartInfo.UseShellExecute = $false
    $StartInfo.CreateNoWindow = $true
    $StartInfo.RedirectStandardOutput = $true
    $StartInfo.RedirectStandardError = $true
    $StartInfo.EnvironmentVariables['NPM_CONFIG_ENGINE_STRICT'] = 'false'
    $StartInfo.EnvironmentVariables['NPM_CONFIG_FUND'] = 'false'
    $StartInfo.EnvironmentVariables['NPM_CONFIG_AUDIT'] = 'false'

    $Process = New-Object System.Diagnostics.Process
    $Process.StartInfo = $StartInfo
    try {
        if (-not $Process.Start()) { throw "Could not start $Executable." }
        $StdoutTask = $Process.StandardOutput.ReadToEndAsync()
        $StderrTask = $Process.StandardError.ReadToEndAsync()
        while (-not $Process.WaitForExit(1000)) { Write-Host '.' -NoNewline }
        $Process.WaitForExit()
        $Stdout = $StdoutTask.GetAwaiter().GetResult()
        $Stderr = $StderrTask.GetAwaiter().GetResult()
        if ($Stdout) { Write-ProcessText $Stdout }
        if ($Stderr) { Write-ProcessText $Stderr }
        $ExitCode = $Process.ExitCode
    }
    catch {
        Write-OutputLog ('PROCESS ERROR: ' + $_.Exception.Message)
        $ExitCode = 1
    }
    finally {
        $Process.Dispose()
    }

    Write-OutputLog ("EXIT CODE: $ExitCode")
    Write-RunLog ("$Label exit code: $ExitCode")
    if ($ExitCode -ne 0) { throw "$Label failed with exit code $ExitCode. See BUILD-AURORA-output.log for the exact output." }
}
function Add-LatestNpmDebugLog {
    $Cache = $script:NpmCachePath
    if ([string]::IsNullOrWhiteSpace($Cache)) { $Cache = $env:npm_config_cache }
    if ([string]::IsNullOrWhiteSpace($Cache)) {
        if ($env:LOCALAPPDATA) { $Cache = Join-Path $env:LOCALAPPDATA 'npm-cache' }
    }
    if ([string]::IsNullOrWhiteSpace($Cache)) { return }
    $LogFolder = Join-Path $Cache '_logs'
    if (-not (Test-Path -LiteralPath $LogFolder -PathType Container)) { return }
    $Latest = Get-ChildItem -LiteralPath $LogFolder -Filter '*-debug-0.log' -File -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($null -eq $Latest) { return }
    Write-RunLog ("npm debug log: $($Latest.FullName)")
    Write-OutputLog "`r`n===== NPM DEBUG LOG: $($Latest.FullName) ====="
    Get-Content -LiteralPath $Latest.FullName -Tail 250 -ErrorAction SilentlyContinue | ForEach-Object { Write-OutputLog ([string]$_) }
    Write-OutputLog '===== END NPM DEBUG LOG ====='
}

try {
    Write-Host '============================================================'
    Write-Host '             AURORA AI v0.8.6 - WINDOWS BUILD'
    Write-Host '============================================================'
    Write-Host "Workspace: $Root"
    if (-not (Test-Path -LiteralPath (Join-Path $Root 'package.json') -PathType Leaf)) {
        throw 'package.json was not found. Extract the complete ZIP before starting the build.'
    }
    $NodeCommand = Get-Command node.exe -ErrorAction SilentlyContinue
    if ($null -eq $NodeCommand) {
        $Winget = Get-Command winget.exe -ErrorAction SilentlyContinue
        if ($null -eq $Winget) { throw 'Node.js chybí a winget není dostupný. Nainstaluj Node.js 24 LTS z https://nodejs.org/ a spusť build znovu.' }
        Write-Host 'Node.js was not found. Installing Node.js 24 LTS automatically through Windows Package Manager...' -ForegroundColor Yellow
        Write-OutputLog 'Node.js was not found. Starting automatic Node.js 24 LTS installation through winget.'
        & $Winget.Source install --id OpenJS.NodeJS.LTS --exact --silent --accept-package-agreements --accept-source-agreements
        if ($LASTEXITCODE -ne 0) { throw "Automatická instalace Node.js LTS přes winget selhala (kód $LASTEXITCODE). Nainstaluj Node.js 24 LTS z https://nodejs.org/." }
        $PossibleNodePaths = @(
            (Join-Path $env:ProgramFiles 'nodejs\node.exe'),
            (Join-Path $env:LOCALAPPDATA 'Programs\nodejs\node.exe'),
            (Join-Path $env:LOCALAPPDATA 'Programs\Node.js\node.exe')
        )
        foreach ($Candidate in $PossibleNodePaths) {
            if ($Candidate -and (Test-Path -LiteralPath $Candidate -PathType Leaf)) { $env:Path = (Split-Path -Parent $Candidate) + ';' + $env:Path; break }
        }
        $NodeCommand = Get-Command node.exe -ErrorAction SilentlyContinue
    }
    if ($null -eq $NodeCommand) { throw 'Node.js instalace byla dokončena, ale node.exe se nepodařilo najít. Zavři a znovu otevři Aurora Build.' }
    $NodePath = $NodeCommand.Source
    if ([string]::IsNullOrWhiteSpace($NodePath)) { $NodePath = $NodeCommand.Definition }
    $NpmCommand = Get-Command npm.cmd -ErrorAction SilentlyContinue
    if ($null -eq $NpmCommand) { throw 'npm.cmd was not found in PATH. Repair the Node.js installation.' }
    $NpmPath = $NpmCommand.Source
    if ([string]::IsNullOrWhiteSpace($NpmPath)) { $NpmPath = $NpmCommand.Definition }

    $NodeVersion = (& $NodePath -p 'process.versions.node').Trim()
    $NodeVersionParsed = [version]$NodeVersion
    $NodeMajor = [int]($NodeVersion.Split('.')[0])
    $NpmVersion = (& $env:ComSpec /d /s /c ('call "' + $NpmPath + '" --version')).Trim()
    $CacheLines = @(& $env:ComSpec /d /s /c ('call "' + $NpmPath + '" config get cache') 2>$null)
    if ($CacheLines.Count -gt 0) { $script:NpmCachePath = ([string]$CacheLines[-1]).Trim() }
    $Package = Get-Content -LiteralPath (Join-Path $Root 'package.json') -Raw | ConvertFrom-Json
    $AppVersion = [string]$Package.version
    Write-RunLog "Node.js: $NodeVersion"
    Write-RunLog "npm: $NpmVersion"
    Write-RunLog "App version: $AppVersion"
    Write-OutputLog "Node.js: $NodeVersion"
    Write-OutputLog "npm: $NpmVersion"
    Write-OutputLog "App version: $AppVersion"
    if ($NodeVersionParsed -lt [version]'22.12.0') { throw "Electron 45.1.0 requires Node.js 22.12.0 or newer. Detected $NodeVersion. Install Node.js 24 LTS or a newer compatible release." }
    if ($NodeMajor -ge 26) {
        Write-Host 'NOTE: Node 26 is detected. Node 24 LTS is the preferred host runtime for the most predictable Electron build.' -ForegroundColor Yellow
        Write-OutputLog 'NOTE: Node 26 detected. The build will continue, but Node 24 LTS is preferred for maximum Electron tooling compatibility.'
    }

    $InstallSucceeded = $false
    for ($Attempt = 1; $Attempt -le 2; $Attempt++) {
        try {
            $InstallArgs = @('install', '--no-audit', '--no-fund', '--ignore-scripts', '--engine-strict=false', '--fetch-retries=5', '--fetch-timeout=300000', '--loglevel=notice')
            if ($Attempt -eq 2) { $InstallArgs += '--prefer-online' }
            Invoke-NativeStep "1/7 - Install npm dependencies (lifecycle scripts deferred; attempt $Attempt/2)" $NpmPath $InstallArgs -UseCmd
            $InstallSucceeded = $true
            break
        }
        catch {
            if ($Attempt -eq 1) {
                Write-Host 'Dependency install did not complete. Verifying npm cache and retrying once...' -ForegroundColor Yellow
                Write-RunLog 'First dependency install attempt failed; verifying cache before one retry.'
                try { Invoke-NativeStep 'npm cache verify' $NpmPath @('cache', 'verify', '--no-fund', '--no-audit') -UseCmd }
                catch { Write-OutputLog ('CACHE VERIFY WARNING: ' + $_.Exception.Message) }
            }
            else {
                Add-LatestNpmDebugLog
                throw
            }
        }
    }
    if (-not $InstallSucceeded) { Add-LatestNpmDebugLog; throw 'npm dependency installation failed after two attempts.' }

    try { Invoke-NativeStep '2/7 - Rebuild required dependency tooling' $NpmPath @('rebuild', '--foreground-scripts', '--no-audit', '--no-fund', '--fetch-retries=5', '--fetch-timeout=300000') -UseCmd }
    catch { Add-LatestNpmDebugLog; throw }

    # Electron 42+ intentionally no longer downloads its runtime during npm install.
    # Invoke the local install-electron bin shipped by the pinned Electron package.
    $ElectronInstaller = Join-Path $Root 'node_modules\.bin\install-electron.cmd'
    if (-not (Test-Path -LiteralPath $ElectronInstaller -PathType Leaf)) {
        Add-LatestNpmDebugLog
        throw 'The local install-electron.cmd command is missing. The Electron npm package was not installed correctly.'
    }
    try { Invoke-NativeStep '3/7 - Download Electron 45.1.0 Windows runtime (install-electron)' $ElectronInstaller @('--no') -UseCmd }
    catch { Add-LatestNpmDebugLog; throw }

    $ElectronExe = Join-Path $Root 'node_modules\electron\dist\electron.exe'
    if (-not (Test-Path -LiteralPath $ElectronExe -PathType Leaf)) {
        Add-LatestNpmDebugLog
        throw 'Electron installer finished but electron.exe is still missing. The downloaded binary could not be verified; check proxy/firewall or GitHub release access.'
    }
    $ElectronSize = (Get-Item -LiteralPath $ElectronExe).Length
    if ($ElectronSize -lt 1000000) { throw "Electron binary verification failed: electron.exe is only $ElectronSize bytes." }
    Write-OutputLog "Electron runtime verified: $ElectronExe ($ElectronSize bytes)"

    Invoke-NativeStep '4/7 - JavaScript syntax check' $NodePath @('scripts/check-syntax.js')
    Invoke-NativeStep '5/7 - Automated test suite' $NpmPath @('test') -UseCmd
    Invoke-NativeStep '6/7 - Release preflight' $NodePath @('scripts/release-preflight.js')

    $Builder = Join-Path $Root 'node_modules\.bin\electron-builder.cmd'
    if (-not (Test-Path -LiteralPath $Builder -PathType Leaf)) { throw 'electron-builder.cmd is missing after dependency installation.' }
    Invoke-NativeStep '7/7 - Windows NSIS installer packaging' $Builder @('--win', 'nsis') -UseCmd

    $Installer = Join-Path $Root "dist\Aurora-AI-Setup-$AppVersion.exe"
    if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) {
        throw "Packaging returned success but the expected installer is missing: $Installer"
    }
    $Hash = (Get-FileHash -LiteralPath $Installer -Algorithm SHA256).Hash
    $HashFile = Join-Path $Root "dist\Aurora-AI-Setup-$AppVersion-SHA256.txt"
    [System.IO.File]::WriteAllText($HashFile, "$Hash  Aurora-AI-Setup-$AppVersion.exe`r`n", $Utf8)
    Write-RunLog 'RESULT: SUCCESS'
    Write-RunLog "Installer: $Installer"
    Write-RunLog "SHA256: $Hash"
    Write-OutputLog "`r`nSUCCESS - INSTALLER CREATED: $Installer"
    Write-OutputLog "SHA256: $Hash"
    exit 0
}
catch {
    $Message = $_.Exception.Message
    Write-RunLog 'RESULT: FAILED'
    Write-RunLog "Stage/error: $Message"
    Write-OutputLog "`r`nBUILD FAILED: $Message"
    Write-OutputLog "Full output: $OutputLog"
    Write-Host ''
    Write-Host 'BUILD FAILED - NO INSTALLER CONFIRMED' -ForegroundColor Red
    Write-Host $Message -ForegroundColor Red
    Write-Host "Full output: $OutputLog"
    exit 1
}
finally {
    Write-RunLog "Ended: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff')"
}
