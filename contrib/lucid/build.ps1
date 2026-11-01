[CmdletBinding()]
param([switch]$Demo, [switch]$MilkDrop)
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (!(Test-Path -LiteralPath $cargo)) { throw 'Install Rust using rustup first.' }
if (!$env:VCINSTALLDIR) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$vs) { throw 'Install Visual Studio Build Tools with Desktop development with C++.' }
    & (Join-Path $vs 'Common7\Tools\Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
}
Push-Location $repo
try {
    function Invoke-Cargo([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments) {
        & $cargo @Arguments
        if ($LASTEXITCODE -ne 0) { throw ('cargo failed: ' + ($Arguments -join ' ')) }
    }
    $features = if ($Demo) { 'lucid,demo,soloist-native-audio' } else { 'lucid,soloist-native-audio' }
    $featureArgs = @('--features', $features)
    if (!$MilkDrop) { $featureArgs += '--no-default-features' }
    Invoke-Cargo @('fmt','--all','--check')
    # Demo fixtures are tested and linted even when the packaged binary is
    # a production build. The release command below uses only $featureArgs.
    Invoke-Cargo (@('test','--locked','--all-targets','--features','demo') + $featureArgs)
    Invoke-Cargo (@('clippy','--locked','--all-targets','--features','demo') + $featureArgs + @('--','-D','warnings'))
    Invoke-Cargo (@('build','--locked','--release') + $featureArgs)
    $out = Join-Path $repo 'dist\Spotifast-Lucid'
    New-Item -ItemType Directory -Force -Path $out | Out-Null
    $exe = Join-Path $out 'spotifast.exe'
    if (Get-Process -Name spotifast -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }) {
        throw 'Build passed. Close the Lucid app and rerun to package it safely.'
    }
    $stagedExe = $exe + '.new'
    Copy-Item -LiteralPath 'target\release\spotifast.exe' -Destination $stagedExe -Force
    if ((Get-FileHash -LiteralPath $stagedExe).Hash -ne (Get-FileHash -LiteralPath 'target\release\spotifast.exe').Hash) {
        throw 'Packaged executable did not match the build. Previous executable was retained.'
    }
    if (Test-Path -LiteralPath $exe) {
        [IO.File]::Replace($stagedExe, $exe, ($exe + '.previous'), $true)
    } else {
        Move-Item -LiteralPath $stagedExe -Destination $exe
    }
    Copy-Item -LiteralPath 'LICENSE' -Destination $out -Force
    Copy-Item -LiteralPath 'FORK.md','INSTALL-LUCID.md' -Destination $out -Force
    Copy-Item -LiteralPath 'packaging/spotifast-portable.txt' -Destination $out -Force
    $themes = Join-Path $out 'spotifast-data\config\themes'
    New-Item -ItemType Directory -Force -Path $themes | Out-Null
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'TrueVision.json'),(Join-Path $PSScriptRoot 'GreatSage.json') -Destination $themes -Force
    $settings = Join-Path $out 'spotifast-data\config\settings.json'
    if (!(Test-Path -LiteralPath $settings)) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'default-settings.json') -Destination $settings
    }
    Write-Output ('Built: ' + $exe)
    Write-Output ('SHA256: ' + (Get-FileHash -LiteralPath $exe).Hash)
} finally { Pop-Location }
