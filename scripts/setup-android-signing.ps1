$ErrorActionPreference = 'Stop'

# This script creates the one long-lived Android release identity and uploads it as GitHub secrets.
# Скрипт створює єдиний довготривалий Android release-ключ і завантажує його як GitHub secrets.
# Dieses Skript erstellt den langlebigen Android-Release-Schlüssel und lädt ihn als GitHub-Secrets hoch.

function ConvertFrom-PrivateSecureString {
    param([Parameter(Mandatory)][Security.SecureString]$Value)

    $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($Value)
    try {
        [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
    }
    finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
    }
}

$repository = 'juv4uk/my-idea'
$keyAlias = 'my-idea-release'
$privateDirectory = Join-Path $PSScriptRoot '..\private\android'
$keystorePath = Join-Path $privateDirectory 'my-idea-release.jks'

if (Test-Path -LiteralPath $keystorePath) {
    throw "Refusing to overwrite the Android key / Відмова від перезапису Android-ключа / Android-Schlüssel wird nicht überschrieben: $keystorePath"
}

if (-not (Get-Command keytool -ErrorAction SilentlyContinue)) {
    throw 'keytool was not found; install Java 21 / keytool не знайдено; встановіть Java 21 / keytool wurde nicht gefunden; installieren Sie Java 21'
}

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw 'GitHub CLI was not found / GitHub CLI не знайдено / GitHub CLI wurde nicht gefunden'
}

$storePasswordSecure = Read-Host 'Keystore password · Пароль keystore · Keystore-Passwort' -AsSecureString
$keyPasswordSecure = Read-Host 'Private-key password · Пароль приватного ключа · Passwort des privaten Schlüssels' -AsSecureString
$storePassword = ConvertFrom-PrivateSecureString $storePasswordSecure
$keyPassword = ConvertFrom-PrivateSecureString $keyPasswordSecure

if ($storePassword.Length -lt 12 -or $keyPassword.Length -lt 12) {
    throw 'Use passwords of at least 12 characters / Використайте паролі щонайменше з 12 символів / Verwenden Sie Passwörter mit mindestens 12 Zeichen'
}

New-Item -ItemType Directory -Path $privateDirectory -Force | Out-Null

& keytool -genkeypair `
    -keystore $keystorePath `
    -storepass $storePassword `
    -alias $keyAlias `
    -keypass $keyPassword `
    -keyalg RSA `
    -keysize 4096 `
    -validity 10000 `
    -dname 'CN=my-idea, O=my-idea'

if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $keystorePath)) {
    throw 'Key generation failed / Створення ключа не вдалося / Schlüsselerzeugung fehlgeschlagen'
}

$keystoreBase64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($keystorePath))
$keystoreBase64 | gh secret set ANDROID_KEYSTORE_BASE64 --repo $repository
$storePassword | gh secret set ANDROID_KEYSTORE_PASSWORD --repo $repository
$keyAlias | gh secret set ANDROID_KEY_ALIAS --repo $repository
$keyPassword | gh secret set ANDROID_KEY_PASSWORD --repo $repository

$storePassword = $null
$keyPassword = $null
$keystoreBase64 = $null

Write-Host "Android signing is configured / Android-підпис налаштовано / Android-Signatur ist konfiguriert"
Write-Host "BACK UP THIS FILE TWICE / ЗРОБІТЬ ДВІ РЕЗЕРВНІ КОПІЇ / ERSTELLEN SIE ZWEI SICHERUNGSKOPIEN: $keystorePath"
