$ErrorActionPreference = 'Stop'
$toolsDir = "c:\Users\Black\OneDrive\Área de Trabalho\revengine\tools\godot"
New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null
$zipPath = Join-Path $toolsDir "godot.zip"

Write-Host "Baixando Godot Engine 4.3 Stable..."
$urls = @(
    "https://github.com/godotengine/godot-builds/releases/download/4.3-stable/Godot_v4.3-stable_win64.exe.zip",
    "https://downloads.tuxfamily.org/godotengine/4.3/Godot_v4.3-stable_win64.exe.zip"
)

$downloaded = $false
foreach ($url in $urls) {
    try {
        Write-Host "Tentando: $url"
        Invoke-WebRequest -Uri $url -OutFile $zipPath -TimeoutSec 120
        if (Test-Path $zipPath) {
            $downloaded = $true
            break
        }
    } catch {
        Write-Host "Falha na URL: $_"
    }
}

if (-not $downloaded) {
    throw "Não foi possível baixar o Godot Engine das fontes oficiais."
}

Write-Host "Extraindo Godot..."
Expand-Archive -Path $zipPath -DestinationPath $toolsDir -Force
Remove-Item -Force $zipPath

$targetExe = Join-Path $toolsDir "godot.exe"
$foundExes = Get-ChildItem -Path $toolsDir -Filter "*.exe"
foreach ($f in $foundExes) {
    if ($f.Name -ne "godot.exe") {
        Copy-Item -Path $f.FullName -Destination $targetExe -Force
        Write-Host "Godot CLI configurado com sucesso em: $targetExe"
        break
    }
}

& "$targetExe" --version
