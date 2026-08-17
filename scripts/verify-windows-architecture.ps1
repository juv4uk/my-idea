[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Path,

    [Parameter(Mandatory = $true)]
    [int]$ExpectedMachine
)

$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Path).Path
$stream = [System.IO.File]::OpenRead($resolved)
$reader = [System.IO.BinaryReader]::new($stream)

try {
    # EN: Read the PE machine field instead of trusting the output directory name.
    # UK: Читаємо поле machine у PE, а не довіряємо назві вихідного каталогу.
    # DE: Das PE-Maschinenfeld wird gelesen, statt dem Namen des Ausgabeverzeichnisses zu vertrauen.
    if ($reader.ReadUInt16() -ne 0x5A4D) { throw "Not an MZ executable: $resolved" }
    $stream.Position = 0x3C
    $peOffset = $reader.ReadUInt32()
    $stream.Position = $peOffset
    if ($reader.ReadUInt32() -ne 0x00004550) { throw "PE signature not found: $resolved" }
    $machine = $reader.ReadUInt16()
    if ($machine -ne $ExpectedMachine) {
        throw ('Unexpected PE machine 0x{0:X4}; expected 0x{1:X4}: {2}' -f $machine, $ExpectedMachine, $resolved)
    }
    Write-Host ('Verified PE machine 0x{0:X4}: {1}' -f $machine, $resolved) -ForegroundColor Green
}
finally {
    $reader.Dispose()
    $stream.Dispose()
}
