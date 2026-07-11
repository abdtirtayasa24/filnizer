$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$PackageJsonPath = Join-Path $RepoRoot "package.json"
$PackageJson = Get-Content $PackageJsonPath -Raw | ConvertFrom-Json
$Version = $PackageJson.version
$ArtifactRoot = Join-Path $RepoRoot "artifacts"
$PortableRoot = Join-Path $ArtifactRoot "portable"
$AppFolder = Join-Path $PortableRoot "Filnizer"
$ZipPath = Join-Path $ArtifactRoot "Filnizer-$Version-portable-windows-x64.zip"
$ReleaseTarget = Join-Path $RepoRoot "src-tauri\target\release"
$BuiltExeCandidates = @(
  (Join-Path $ReleaseTarget "filnizer.exe"),
  (Join-Path $ReleaseTarget "Filnizer.exe")
)

Write-Host "Building Filnizer frontend..."
Push-Location $RepoRoot
try {
  npm run build
  Write-Host "Building Filnizer Tauri executable..."
  npm run tauri -- build --no-bundle
} finally {
  Pop-Location
}

$BuiltExe = $BuiltExeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $BuiltExe) {
  throw "Could not find built executable in $ReleaseTarget"
}

if (Test-Path $PortableRoot) {
  Remove-Item $PortableRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $AppFolder | Out-Null

Copy-Item $BuiltExe (Join-Path $AppFolder "Filnizer.exe") -Force

$BinarySources = @(
  (Join-Path $RepoRoot "binaries"),
  (Join-Path $RepoRoot "src-tauri\binaries")
)
foreach ($Source in $BinarySources) {
  if (Test-Path $Source) {
    $Destination = Join-Path $AppFolder "binaries"
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    Copy-Item (Join-Path $Source "*") $Destination -Recurse -Force
  }
}

$DocsFolder = Join-Path $AppFolder "docs"
New-Item -ItemType Directory -Path $DocsFolder -Force | Out-Null

$PortableReadme = Join-Path $RepoRoot "docs\release\portable-readme.md"
if (Test-Path $PortableReadme) {
  Copy-Item $PortableReadme (Join-Path $AppFolder "README.md") -Force
}

$LicenseCandidates = @(
  (Join-Path $RepoRoot "LICENSE"),
  (Join-Path $RepoRoot "LICENSE.md"),
  (Join-Path $RepoRoot "docs\release\licenses.md")
)
foreach ($License in $LicenseCandidates) {
  if (Test-Path $License) {
    Copy-Item $License $DocsFolder -Force
  }
}

if (Test-Path $ZipPath) {
  Remove-Item $ZipPath -Force
}
Compress-Archive -Path (Join-Path $AppFolder "*") -DestinationPath $ZipPath -Force

Write-Host "Portable folder: $AppFolder"
Write-Host "Portable ZIP: $ZipPath"
