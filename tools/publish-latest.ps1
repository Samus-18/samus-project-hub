# Future development: publish a NEW version only after local validation and consent.
param(
 [Parameter(Mandatory=$true)][ValidateSet('aurora-ai','horizon-boost','thalvoryn')][string]$Project,
 [Parameter(Mandatory=$true)][string]$VersionTag,
 [Parameter(Mandatory=$true)][string]$Asset,
 [Parameter(Mandatory=$true)][string]$NotesFile,
 [switch]$RightsVerified
)
$ErrorActionPreference='Stop'
$repo='Samus-18/samus-project-hub'
if ($Project -eq 'thalvoryn' -and -not $RightsVerified) {
 throw 'Thalvoryn source/structure permissions have not been verified. Refusing binary release.'
}
$prefix=if($Project -eq 'aurora-ai'){'aurora-ai-v'}elseif($Project -eq 'horizon-boost'){'horizon-boost-v'}else{'thalvoryn-v'}
if(-not $VersionTag.StartsWith($prefix)){throw 'Wrong project release tag prefix'}
$p=(Resolve-Path -LiteralPath $Asset -ErrorAction Stop).Path
$n=(Resolve-Path -LiteralPath $NotesFile -ErrorAction Stop).Path
if(-not (Get-Command gh -ErrorAction SilentlyContinue)){throw 'GitHub CLI required'}
gh release view $VersionTag --repo $repo 2>$null | Out-Null
if($LASTEXITCODE -eq 0){throw "Tag already exists: $VersionTag. Do not overwrite release assets."}
$sum=(Get-FileHash -Algorithm SHA256 -LiteralPath $p).Hash
$checksum=Join-Path (Split-Path $p) ((Split-Path $p -Leaf)+'.sha256')
"$sum  $(Split-Path $p -Leaf)" | Set-Content $checksum -Encoding ascii
gh release create $VersionTag $p $checksum --repo $repo --notes-file $n --prerelease --target main
if($LASTEXITCODE -ne 0){throw "Release upload failed"}
gh release view $VersionTag --repo $repo --json url,assets
