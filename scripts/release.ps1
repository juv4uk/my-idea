[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# EN: Stop before changing files when the repository is not clean.
# UK: Zupynytysia do zminy failiv, yakshcho repozytorii ne chystyi.
# DE: Vor Dateianderungen stoppen, wenn das Repository nicht sauber ist.
if (-not (Test-Path -LiteralPath '.git') -or -not (Test-Path -LiteralPath 'package.json') -or -not (Test-Path -LiteralPath 'src-tauri/Cargo.toml')) {
    throw 'Run this script from the repository root.'
}

$pendingChanges = @(git status --porcelain)
if ($LASTEXITCODE -ne 0) { throw 'Could not read git status.' }
if ($pendingChanges.Count -gt 0) { throw 'Commit or stash existing changes first.' }

$currentBranch = (git branch --show-current).Trim()
if ($LASTEXITCODE -ne 0 -or $currentBranch -ne 'main') {
    throw "Release must be created from main; current branch: $currentBranch"
}

git rev-parse --verify --quiet "refs/tags/v$Version" | Out-Null
if ($LASTEXITCODE -eq 0) { throw "Tag v$Version already exists." }

Write-Host "Preparing my-idea v$Version" -ForegroundColor Cyan

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

# EN: bun.lock doesn't store the root package's own version (only name +
# dependencies), so there's no lockfile to keep in sync here — unlike the
# old package-lock.json, which this repo no longer uses. Set package.json's
# version directly instead of shelling out to npm.
# UK: bun.lock не зберігає версію кореневого пакета, тому синхронізувати
# lockfile тут не потрібно — на відміну від колишнього package-lock.json.
# DE: bun.lock speichert die Version des Root-Pakets nicht, daher muss hier
# keine Lockfile synchronisiert werden.
$packageJsonPath = 'package.json'
$packageJson = Get-Content -LiteralPath $packageJsonPath -Raw | ConvertFrom-Json
$packageJson.version = $Version
$packageJsonText = $packageJson | ConvertTo-Json -Depth 100
[System.IO.File]::WriteAllText($packageJsonPath, "$packageJsonText`n", $utf8NoBom)

function Set-FirstRegexMatch {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Pattern,
        [Parameter(Mandatory = $true)][string]$Replacement,
        [System.Text.RegularExpressions.RegexOptions]$Options = [System.Text.RegularExpressions.RegexOptions]::None
    )

    $content = [System.IO.File]::ReadAllText($Path)
    $regex = [regex]::new($Pattern, $Options)
    if (-not $regex.IsMatch($content)) { throw "Version entry not found in $Path" }
    $updated = $regex.Replace($content, $Replacement, 1)
    [System.IO.File]::WriteAllText($Path, $updated, $utf8NoBom)
}

# EN: Only these first matches are application versions.
# UK: Lyshe tsi pershi zbiihy ye versiiamy zastosunku.
# DE: Nur diese ersten Treffer sind Anwendungsversionen.
Set-FirstRegexMatch -Path 'src-tauri/Cargo.toml' -Pattern '^version = "\d+\.\d+\.\d+"' -Replacement "version = `"$Version`"" -Options Multiline
Set-FirstRegexMatch -Path 'src-tauri/tauri.conf.json' -Pattern '"version"\s*:\s*"\d+\.\d+\.\d+"' -Replacement "`"version`":  `"$Version`""

Write-Host 'Running release checks...' -ForegroundColor Cyan
cargo check --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'cargo check failed.' }
# EN: The my-lisp Rust crates ship independently of src-tauri (CLI, WASM); a release must not skip their tests.
# UK: Rust-крейти my-lisp постачаються окремо від src-tauri (CLI, WASM); реліз не повинен пропускати їхні тести.
# DE: Die my-lisp-Rust-Crates werden unabhängig von src-tauri ausgeliefert (CLI, WASM); ein Release darf ihre Tests nicht überspringen.
cargo test --manifest-path crates/my-lisp/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'cargo test (my-lisp) failed.' }
cargo test --manifest-path crates/my-lisp-cli/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'cargo test (my-lisp-cli) failed.' }
cargo test --manifest-path crates/my-lisp-literate/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'cargo test (my-lisp-literate) failed.' }
bun install --frozen-lockfile
if ($LASTEXITCODE -ne 0) { throw 'bun install --frozen-lockfile failed.' }
bun run test
if ($LASTEXITCODE -ne 0) { throw 'bun run test failed.' }
bun run check
if ($LASTEXITCODE -ne 0) { throw 'bun run check failed.' }
bun run build
if ($LASTEXITCODE -ne 0) { throw 'bun run build failed.' }

# EN: Refuse to publish changes outside the five version files.
# UK: Ne publikuvaty zminy poza piatma failamy versii.
# DE: Anderungen ausserhalb der funf Versionsdateien nicht veroffentlichen.
$releaseFiles = @('package.json', 'src-tauri/Cargo.toml', 'Cargo.lock', 'src-tauri/tauri.conf.json')
$changedFiles = @(git status --porcelain | ForEach-Object { $_.Substring(3) })
$unexpectedFiles = @($changedFiles | Where-Object { $_ -notin $releaseFiles })
if ($unexpectedFiles.Count -gt 0) { throw "Unexpected changed files: $($unexpectedFiles -join ', ')" }

# UTF-8 text is Base64 encoded so Windows PowerShell 5.1 can parse this BOM-less script safely.
$commitTemplate = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('cmVsZWFzZTogdnswfSB8INCS0LjQv9GD0YHQuiB2ezB9IHwgVmVyw7ZmZmVudGxpY2h1bmcgdnswfQ=='))
$tagTemplate = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('bXktaWRlYSB2ezB9IHwg0JLQtdGA0YHRltGPIHZ7MH0gfCBWZXLDtmZmZW50bGljaHVuZyB2ezB9'))
$commitMessage = $commitTemplate -f $Version
$tagMessage = $tagTemplate -f $Version

git add -- $releaseFiles
if ($LASTEXITCODE -ne 0) { throw 'git add failed.' }
git commit -m $commitMessage
if ($LASTEXITCODE -ne 0) { throw 'git commit failed.' }
git tag -a "v$Version" -m $tagMessage
if ($LASTEXITCODE -ne 0) { throw 'git tag failed.' }

# EN/UK/DE: Atomic push prevents a branch-only or tag-only partial release.
git push --atomic origin main "v$Version"
if ($LASTEXITCODE -ne 0) { throw 'Atomic push failed; commit and tag remain local for inspection.' }

Write-Host "Release v$Version pushed successfully." -ForegroundColor Green
