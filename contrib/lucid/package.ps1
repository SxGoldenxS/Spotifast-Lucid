[CmdletBinding()]
param(
    [ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version = '0.7.0',
    [string]$Binary = '',
    [string]$Iscc = '',
    [string]$OutputDirectory = ''
)
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
if (!$Binary) { $Binary = Join-Path $repo 'dist\Spotifast-Lucid\spotifast.exe' }
if (!$OutputDirectory) { $OutputDirectory = Join-Path $repo 'dist\Installers' }
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$output = (Resolve-Path -LiteralPath $OutputDirectory).Path
$cache = Join-Path $repo '.cache'
$stagingRoot = Join-Path $cache ('lucid-package-' + [guid]::NewGuid().ToString('N'))
$stage = Join-Path $stagingRoot 'Spotifast-Lucid-Windows-x64'
try {
    New-Item -ItemType Directory -Force -Path "$stage\spotifast-data\config\themes" | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination "$stage\spotifast.exe"
    foreach ($file in @('LICENSE','FORK.md','INSTALL-LUCID.md')) {
        Copy-Item -LiteralPath (Join-Path $repo $file) -Destination $stage
    }
    Copy-Item -LiteralPath (Join-Path $repo 'packaging\spotifast-portable.txt') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'default-settings.json') -Destination "$stage\spotifast-data\config\settings.json"
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'TrueVision.json'),(Join-Path $PSScriptRoot 'GreatSage.json') -Destination "$stage\spotifast-data\config\themes"
    $zip = Join-Path $output 'Spotifast-Lucid-Windows-x64.zip'
    Compress-Archive -LiteralPath $stage -DestinationPath $zip -CompressionLevel Optimal -Force
    & (Join-Path $PSScriptRoot 'build-installer.ps1') -Version $Version -Binary $binaryPath -Iscc $Iscc -OutputDirectory $output
    $setup = Join-Path $output 'Spotifast-Lucid-Windows-x64-Setup.exe'
    $lines = foreach ($file in @($zip, $setup)) {
        (Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant() + '  ' + [IO.Path]::GetFileName($file)
    }
    Set-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt') -Value $lines -Encoding ascii
    Write-Output ('Packaged ZIP and installer: ' + $output)
} finally {
    $resolved = [IO.Path]::GetFullPath($stagingRoot)
    if (!$resolved.StartsWith([IO.Path]::GetFullPath($cache) + [IO.Path]::DirectorySeparatorChar) -or
        [IO.Path]::GetFileName($resolved) -notlike 'lucid-package-*') { throw 'Unsafe staging cleanup path.' }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
