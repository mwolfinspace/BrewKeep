# BrewKeep — Self-sign the release binary
# Creates a self-signed code-signing certificate and signs brewkeep.exe

# Create self-signed code-signing cert
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=BrewKeep Local" `
  -KeyUsage DigitalSignature `
  -FriendlyName "BrewKeep Local" `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -HashAlgorithm SHA256 `
  -NotAfter (Get-Date).AddYears(5)

# Trust it
$store = New-Object System.Security.Cryptography.X509Certificates.X509Store("Root", "CurrentUser")
$store.Open("ReadWrite"); $store.Add($cert); $store.Close()
$store = New-Object System.Security.Cryptography.X509Certificates.X509Store("TrustedPeople", "CurrentUser")
$store.Open("ReadWrite"); $store.Add($cert); $store.Close()

# Find signtool (Windows SDK)
$signtool = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" |
            Sort-Object FullName | Select-Object -Last 1

if (-not $signtool) {
    Write-Error "signtool.exe not found. Install Windows SDK."
    exit 1
}

& $signtool.FullName sign /fd SHA256 /n "BrewKeep Local" /tr http://timestamp.digicert.com /td SHA256 `
   "$PSScriptRoot\..\target\release\brewkeep.exe"

Write-Host "Signed: brewkeep.exe"
