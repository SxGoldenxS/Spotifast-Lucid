[CmdletBinding()]
param([switch]$Demo, [switch]$MilkDrop)
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
Push-Location $repo
try {
    function Invoke-Git([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments) {
        & git @Arguments
        if ($LASTEXITCODE -ne 0) { throw ('git failed: ' + ($Arguments -join ' ')) }
    }
    $dirty = & git status --porcelain
    if ($LASTEXITCODE -ne 0 -or $dirty) { throw 'Commit or stash local changes before updating. Nothing has been changed.' }
    $branch = & git branch --show-current
    if (!$branch) { throw 'Check out your Lucid branch before updating.' }
    $remote = & git remote get-url upstream
    if ($remote -notmatch '^https://github\.com/crmne/spotifast(?:\.git)?$') { throw 'upstream must be the official crmne/spotifast repository.' }
    Invoke-Git @('fetch','upstream','main')
    $baseFile = Join-Path $PSScriptRoot 'upstream.txt'
    $base = (Get-Content -LiteralPath $baseFile -Raw).Trim()
    if ($base -notmatch '^[0-9a-f]{40}$') { throw 'Invalid recorded upstream commit.' }
    & git merge-base --is-ancestor $base upstream/main
    if ($LASTEXITCODE -ne 0) { throw 'Upstream history changed. Review it before importing updates.' }
    $commits = @(& git rev-list --reverse ($base + '..upstream/main'))
    if ($LASTEXITCODE -ne 0) { throw 'Could not list upstream changes.' }
    if (!$commits.Count) { Write-Output 'Already up to date with upstream.'; return }
    $merges = & git rev-list --merges ($base + '..upstream/main')
    if ($merges) { throw 'Upstream contains merge commits. Review the update manually.' }
    $upstreamHead = & git rev-parse upstream/main
    # Import new upstream commits on a separate branch. Cherry-pick keeps
    # upstream authorship and our published history; no force-push is needed.
    $candidate = 'codex/lucid-update-' + (Get-Date -Format 'yyyyMMdd-HHmmss')
    Invoke-Git @('switch','-c',$candidate)
    Write-Output ('Original branch retained: ' + $branch)
    Invoke-Git (@('-c','user.name=Project maintainer','-c','user.email=maintainer@spotifast-lucid.invalid','cherry-pick') + $commits)
    Set-Content -LiteralPath $baseFile -Value $upstreamHead -Encoding ascii
    Invoke-Git @('add','contrib/lucid/upstream.txt')
    Invoke-Git @('-c','user.name=Project maintainer','-c','user.email=maintainer@spotifast-lucid.invalid','commit','-m',('Record upstream integration through ' + $upstreamHead))
    & (Join-Path $PSScriptRoot 'build.ps1') -Demo:$Demo -MilkDrop:$MilkDrop
    Invoke-Git @('switch',$branch)
    Invoke-Git @('merge','--ff-only',$candidate)
    Write-Output 'Update checked and integrated. Push normally; no published history was rewritten.'
} finally { Pop-Location }
