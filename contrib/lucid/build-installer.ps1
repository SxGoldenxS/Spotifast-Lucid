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
if (!$Iscc) {
    $found = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($found) { $Iscc = $found.Source }
    foreach ($candidate in @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe")) {
        if (!$Iscc -and (Test-Path -LiteralPath $candidate)) { $Iscc = $candidate }
    }
}
if (!$Iscc -or !(Test-Path -LiteralPath $Iscc)) { throw 'Inno Setup 6.3+ is required on the build machine only. GitHub Windows runners include it.' }
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$outputPath = (Resolve-Path -LiteralPath $OutputDirectory).Path
& $Iscc '/Q' "/DVersion=$Version" "/DBinary=$binaryPath" "/DOutputDir=$outputPath" (Join-Path $PSScriptRoot 'windows-installer.iss')
if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed.' }
$installer = Join-Path $outputPath 'Spotifast-Lucid-Windows-x64-Setup.exe'
Write-Output ('Installer: ' + $installer)
Write-Output ('SHA256: ' + (Get-FileHash -LiteralPath $installer).Hash.ToLowerInvariant())
