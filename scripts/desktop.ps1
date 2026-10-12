param([ValidateSet("dev","build","check","test")][string]$Mode="dev")
$ErrorActionPreference="Stop"
$taskRoot=Split-Path -Parent $PSScriptRoot
$vswhere=Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (!(Test-Path -LiteralPath $vswhere)) {throw "Install Visual Studio C++ Build Tools and Windows SDK first."}
$vsInstall=& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$vsInstall) {throw "Visual Studio C++ tools were not found."}
& (Join-Path $vsInstall "Common7\Tools\Launch-VsDevShell.ps1") -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
Set-Location -LiteralPath $taskRoot
if ($Mode -eq "dev" -or $Mode -eq "build") { & npm.cmd run tauri -- $Mode }
else { Set-Location -LiteralPath (Join-Path $taskRoot "src-tauri"); & cargo $Mode }
$resultCode = $LASTEXITCODE
if ($Mode -eq "build" -and $resultCode -eq 0) { & (Join-Path $PSScriptRoot "publish-installer.ps1") }
exit $resultCode
