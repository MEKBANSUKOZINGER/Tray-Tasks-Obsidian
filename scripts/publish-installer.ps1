param()
$ErrorActionPreference = "Stop"
$taskRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$version = (Get-Content -LiteralPath (Join-Path $taskRoot "package.json") -Raw | ConvertFrom-Json).version
$currentName = "Tray Tasks_$($version)_x64-setup.exe"
$bundle = Join-Path $taskRoot "src-tauri\target\release\bundle\nsis"
$artifacts = Join-Path $taskRoot "artifacts"
$legacy = Join-Path $artifacts "Legacy"
$source = Join-Path $bundle $currentName
if (!(Test-Path -LiteralPath $source -PathType Leaf)) { throw "Current installer not found: $source" }
foreach ($directory in @($bundle, $artifacts, $legacy)) {
    $resolved = [IO.Path]::GetFullPath($directory)
    if (!$resolved.StartsWith($taskRoot + "\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Installer directory is outside the project: $resolved"
    }
}
New-Item -ItemType Directory -Path $legacy -Force | Out-Null
function Get-InstallerHash($path) {
    $stream = [IO.File]::OpenRead($path)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace("-", "") }
    finally { $stream.Dispose(); $algorithm.Dispose() }
}
function Assert-SameInstaller($first, $second) {
    if ((Get-InstallerHash $first) -ne
        (Get-InstallerHash $second)) {
        throw "Different installers have the same filename: $first / $second"
    }
}
# Validate duplicate build outputs before consolidating them in Legacy.
$old = @(foreach ($directory in @($artifacts, $bundle)) {
    Get-ChildItem -LiteralPath $directory -File |
        Where-Object { $_.Name -match '^Tray Tasks_\d+\.\d+\.\d+_x64-setup\.exe$' -and $_.Name -ne $currentName }
})
foreach ($group in ($old | Group-Object Name)) {
    $destination = Join-Path $legacy $group.Name
    $reference = $group.Group[0].FullName
    foreach ($item in $group.Group) { Assert-SameInstaller $reference $item.FullName }
    if (Test-Path -LiteralPath $destination) { Assert-SameInstaller $reference $destination }
}
foreach ($item in $old) {
    $destination = Join-Path $legacy $item.Name
    if (Test-Path -LiteralPath $destination) {
        Assert-SameInstaller $item.FullName $destination
        Remove-Item -LiteralPath $item.FullName
    } else {
        Move-Item -LiteralPath $item.FullName -Destination $destination
    }
}
$current = Join-Path $artifacts $currentName
Copy-Item -LiteralPath $source -Destination $current -Force
Assert-SameInstaller $source $current
$entries = @(foreach ($item in (Get-ChildItem -LiteralPath $legacy -File | Sort-Object Name)) {
    if ($item.Name -match '^Tray Tasks_\d+\.\d+\.\d+_x64-setup\.exe$') {
        "$( (Get-InstallerHash $item.FullName) )  Legacy/$($item.Name)"
    }
})
$entries += "$( (Get-InstallerHash $current) )  $currentName"
[IO.File]::WriteAllLines((Join-Path $artifacts "SHA256SUMS.txt"), [string[]]$entries, (New-Object Text.UTF8Encoding($false)))
Write-Host "Current installer: $current"
Write-Host "Legacy installers: $legacy"
