param(
    [Parameter(Mandatory = $true)][string]$InstallRoot,
    [switch]$StartServer
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

function Emit([string]$Message) { Write-Output ("AURORA_PROGRESS|" + $Message) }
function Find-Python {
    $Launcher = Get-Command py.exe -ErrorAction SilentlyContinue
    if ($Launcher) {
        foreach ($Minor in @('3.13','3.12')) {
            try {
                $PythonTag = '-' + $Minor
                $Candidate = (& $Launcher.Source $PythonTag -c 'import sys; print(sys.executable)' 2>$null | Select-Object -First 1)
                if ($LASTEXITCODE -eq 0 -and $Candidate -and (Test-Path -LiteralPath $Candidate.Trim())) { return $Candidate.Trim() }
            } catch {}
        }
    }
    foreach ($Name in @('python.exe','python3.exe')) {
        $Cmd = Get-Command $Name -ErrorAction SilentlyContinue
        if ($Cmd) {
            try {
                $Version = (& $Cmd.Source -c 'import sys; print("%s.%s" % sys.version_info[:2])' 2>$null | Select-Object -First 1)
                if ($LASTEXITCODE -eq 0 -and $Version -match '^3\.(12|13)$') { return $Cmd.Source }
            } catch {}
        }
    }
    foreach ($Candidate in @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Python\Python313\python.exe'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Python\Python312\python.exe'),
        'C:\Program Files\Python313\python.exe',
        'C:\Program Files\Python312\python.exe'
    )) { if ($Candidate -and (Test-Path -LiteralPath $Candidate)) { return $Candidate } }
    return $null
}

try {
    if (-not (Test-Path -LiteralPath $InstallRoot -PathType Container)) { New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null }
    $ComfyRoot = Join-Path $InstallRoot 'ComfyUI'
    $VenvRoot = Join-Path $InstallRoot 'venv'
    $VenvPython = Join-Path $VenvRoot 'Scripts\python.exe'
    $ModelFile = Join-Path $ComfyRoot 'models\checkpoints\v1-5-pruned-emaonly.safetensors'
    $ExpectedHash = '6ce0161689b3853acaa03779ec93eafe75a02f4ced659bee03f50797806fa2fa'
    $ModelUrl = 'https://huggingface.co/Comfy-Org/stable-diffusion-1.5-archive/resolve/main/v1-5-pruned-emaonly.safetensors'

    Emit 'Kontroluji Python a existující instalaci obrazového enginu…'
    $Python = Find-Python
    if (-not $Python) {
        $Winget = Get-Command winget.exe -ErrorAction SilentlyContinue
        if (-not $Winget) { throw 'Python 3.12/3.13 není nainstalovaný a Windows Package Manager (winget) není dostupný. Nainstaluj Python 3.13 x64 a spusť přípravu znovu.' }
        Emit 'Instaluji Python 3.13 x64 přes Windows Package Manager…'
        & $Winget.Source install --id Python.Python.3.13 --exact --scope user --silent --accept-package-agreements --accept-source-agreements
        if ($LASTEXITCODE -ne 0) { throw "Instalace Pythonu přes winget skončila kódem $LASTEXITCODE." }
        $env:Path = "$env:LOCALAPPDATA\Programs\Python\Python313;$env:LOCALAPPDATA\Programs\Python\Python313\Scripts;$env:Path"
        $Python = Find-Python
        if (-not $Python) { throw 'Python se nainstaloval, ale jeho python.exe nebyl nalezen. Zavři a znovu spusť Auroru, potom zkus přípravu znovu.' }
    }
    Emit ("Používám " + $Python)

    $Drive = [System.IO.Path]::GetPathRoot($InstallRoot)
    $Disk = Get-CimInstance Win32_LogicalDisk -Filter ("DeviceID='" + $Drive.TrimEnd('\') + "'") -ErrorAction SilentlyContinue
    if ($Disk -and [double]$Disk.FreeSpace -lt 14GB) { throw 'Na disku zbývá méně než 14 GB. Pro ComfyUI, CPU PyTorch a model SD 1.5 uvolni alespoň 14 GB.' }

    if (-not (Test-Path -LiteralPath (Join-Path $ComfyRoot 'main.py') -PathType Leaf)) {
        Emit 'Stahuji oficiální ComfyUI…'
        $Archive = Join-Path $InstallRoot 'ComfyUI-source.zip'
        if (Test-Path -LiteralPath $Archive) { Remove-Item -LiteralPath $Archive -Force }
        Invoke-WebRequest -Uri 'https://github.com/Comfy-Org/ComfyUI/archive/refs/heads/master.zip' -OutFile $Archive -UseBasicParsing
        $ExtractRoot = Join-Path $InstallRoot 'source-extract'
        if (Test-Path -LiteralPath $ExtractRoot) { Remove-Item -LiteralPath $ExtractRoot -Recurse -Force }
        Expand-Archive -LiteralPath $Archive -DestinationPath $ExtractRoot -Force
        $Extracted = Get-ChildItem -LiteralPath $ExtractRoot -Directory | Select-Object -First 1
        if (-not $Extracted -or -not (Test-Path -LiteralPath (Join-Path $Extracted.FullName 'main.py'))) { throw 'Stažený archiv neobsahuje očekávaný ComfyUI/main.py.' }
        if (Test-Path -LiteralPath $ComfyRoot) { Remove-Item -LiteralPath $ComfyRoot -Recurse -Force }
        Move-Item -LiteralPath $Extracted.FullName -Destination $ComfyRoot
        Remove-Item -LiteralPath $ExtractRoot -Recurse -Force
        Remove-Item -LiteralPath $Archive -Force
    } else { Emit 'ComfyUI už je stažené; ponechávám existující instalaci.' }

    if (-not (Test-Path -LiteralPath $VenvPython -PathType Leaf)) {
        Emit 'Vytvářím izolované Python prostředí pro ComfyUI…'
        & $Python -m venv $VenvRoot
        if ($LASTEXITCODE -ne 0) { throw "Vytvoření Python virtualenv selhalo (kód $LASTEXITCODE)." }
    }
    $DependencyMarker = Join-Path $InstallRoot 'python-dependencies.ready'
    $DependenciesReady = $false
    if (Test-Path -LiteralPath $DependencyMarker) {
        try { & $VenvPython -c 'import torch' 2>$null; $DependenciesReady = ($LASTEXITCODE -eq 0) } catch {}
    }
    if (-not $DependenciesReady) {
        Emit 'Instaluji CPU PyTorch (kompatibilní fallback pro různé grafické karty)…'
        & $VenvPython -m pip install --upgrade pip
        if ($LASTEXITCODE -ne 0) { throw 'Aktualizace pip selhala.' }
        & $VenvPython -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu
        if ($LASTEXITCODE -ne 0) { throw 'Instalace CPU PyTorch selhala.' }
        Emit 'Instaluji závislosti ComfyUI…'
        & $VenvPython -m pip install -r (Join-Path $ComfyRoot 'requirements.txt')
        if ($LASTEXITCODE -ne 0) { throw 'Instalace závislostí ComfyUI selhala. Podrobnosti jsou v konzoli Aurory.' }
        & $VenvPython -c 'import torch'
        if ($LASTEXITCODE -ne 0) { throw 'Kontrola importu PyTorch/ComfyUI selhala.' }
        Set-Content -LiteralPath $DependencyMarker -Value 'ready' -Encoding ascii
    } else { Emit 'Python závislosti už jsou nainstalované; přeskočím opakované stahování.' }

    $CheckpointDir = Join-Path $ComfyRoot 'models\checkpoints'
    New-Item -ItemType Directory -Path $CheckpointDir -Force | Out-Null
    $ValidModel = $false
    if (Test-Path -LiteralPath $ModelFile -PathType Leaf) {
        Emit 'Ověřuji existující model Stable Diffusion 1.5…'
        $CurrentHash = (Get-FileHash -LiteralPath $ModelFile -Algorithm SHA256).Hash.ToLowerInvariant()
        $ValidModel = $CurrentHash -eq $ExpectedHash
        if (-not $ValidModel) { Remove-Item -LiteralPath $ModelFile -Force }
    }
    if (-not $ValidModel) {
        Emit 'Stahuji model Stable Diffusion 1.5 (přibližně 4,27 GB)…'
        $Curl = Get-Command curl.exe -ErrorAction SilentlyContinue
        if (-not $Curl) { throw 'Windows curl.exe nebyl nalezen. Aktualizuj Windows nebo nainstaluj curl.' }
        & $Curl.Source --fail --location --retry 5 --retry-delay 3 --retry-all-errors --progress-bar --output $ModelFile $ModelUrl
        if ($LASTEXITCODE -ne 0) { throw "Stažení modelu selhalo (curl exit $LASTEXITCODE). Neúplný soubor zůstává pro diagnostiku; můžeš spustit přípravu znovu." }
        $DownloadedHash = (Get-FileHash -LiteralPath $ModelFile -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($DownloadedHash -ne $ExpectedHash) { Remove-Item -LiteralPath $ModelFile -Force; throw 'Kontrolní SHA-256 modelu nesouhlasí. Soubor byl odstraněn a je nutné stažení zopakovat.' }
    }
    Emit 'Model ověřen kontrolním součtem SHA-256.'

    if ($StartServer) {
        $AlreadyOnline = $false
        try { $null = Invoke-RestMethod -Uri 'http://127.0.0.1:8188/system_stats' -TimeoutSec 2; $AlreadyOnline = $true } catch {}
        if (-not $AlreadyOnline) {
            Emit 'Spouštím ComfyUI lokálně v CPU režimu…'
            $StdoutLog = Join-Path $InstallRoot 'comfyui-stdout.log'
            $StderrLog = Join-Path $InstallRoot 'comfyui-stderr.log'
            $Args = @('main.py','--cpu','--listen','127.0.0.1','--port','8188')
            $null = Start-Process -FilePath $VenvPython -ArgumentList $Args -WorkingDirectory $ComfyRoot -WindowStyle Hidden -RedirectStandardOutput $StdoutLog -RedirectStandardError $StderrLog -PassThru
        } else { Emit 'ComfyUI už běží; nepouštím druhou instanci.' }
        $Ready = $false
        for ($Index = 0; $Index -lt 180; $Index++) {
            try { $null = Invoke-RestMethod -Uri 'http://127.0.0.1:8188/system_stats' -TimeoutSec 2; $Ready = $true; break } catch {}
            Start-Sleep -Seconds 2
            if (($Index % 5) -eq 0) { Emit 'Čekám, až ComfyUI dokončí start…' }
        }
        if (-not $Ready) { throw "ComfyUI se nepodařilo spustit v limitu 6 minut. Logy: $StdoutLog a $StderrLog" }
    }
    Emit 'HOTOVO: Python, ComfyUI, CPU backend a model jsou připravené.'
    exit 0
}
catch {
    Emit ("CHYBA: " + $_.Exception.Message)
    exit 1
}
