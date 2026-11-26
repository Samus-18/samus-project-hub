# Vanta FX production Authenticode signing. Requires a CA-trusted code-signing
# certificate with private key in CurrentUser/My and Windows SDK signtool.
# Refuse to self-sign or imply SmartScreen trust without a valid publisher cert.
[CmdletBinding()]
param(
 [Parameter(Mandatory=$true)][string]$File,
 [Parameter(Mandatory=$true)][string]$CertThumbprint,
 [string]$TimeStampUrl='http://timestamp.digicert.com'
)
$ErrorActionPreference='Stop'
$path=(Resolve-Path -LiteralPath $File -ErrorAction Stop).Path
if(-not $path.ToLowerInvariant().EndsWith('.exe')){throw 'Only Windows EXE binaries can be Authenticode signed'}
$thumbprint=($CertThumbprint -replace '[^a-fA-F0-9]','').ToUpperInvariant()
if($thumbprint.Length -ne 40){throw 'Expected 40-char certificate SHA1 thumbprint'}
$cert=Get-Item -LiteralPath ("Cert:\CurrentUser\My\"+$thumbprint) -ErrorAction Stop
if(-not $cert.HasPrivateKey){throw 'Signing certificate has no accessible private key'}
if($cert.NotAfter -lt [DateTime]::UtcNow){throw 'Signing certificate has expired'}
$eku=$cert.EnhancedKeyUsageList | ForEach-Object {$_.ObjectId.Value}
if($eku -notcontains '1.3.6.1.5.5.7.3.3'){throw 'Certificate is not for code signing'}
$sign=Get-Command 'signtool.exe' -ErrorAction Stop
& $sign.Source sign /debug /sha1 $thumbprint /s My /fd SHA256 /tr $TimeStampUrl /td SHA256 $path
if($LASTEXITCODE -ne 0){throw 'SignTool failed'}
$result=Get-AuthenticodeSignature -FilePath $path
if($result.Status -ne 'Valid' -or $result.SignerCertificate.Thumbprint -ne $thumbprint){
 throw ('Authenticode verification failed: '+$result.Status)
}
Write-Output ('AUTHENTICODE_VALID '+$result.SignerCertificate.Subject)
Get-FileHash -Algorithm SHA256 -LiteralPath $path
