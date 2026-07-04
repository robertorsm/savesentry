param(
    [string]$Profile = "release-windows",
    [string]$Project = "SaveSentry"
)

$exePath = "target\$Profile\$Project.exe"
$zipPath = "target\$Profile\$Project.zip"

Write-Host "=== Build Release para Windows ===" -ForegroundColor Cyan
Write-Host "Compilando com perfil $Profile..." -ForegroundColor Yellow

cargo build --profile $Profile

if (Test-Path $exePath) {
    $size = [math]::Round((Get-Item $exePath).Length / 1MB, 2)
    Write-Host "Build concluido com sucesso!" -ForegroundColor Green
    Write-Host "Executavel: $exePath"
    Write-Host "Tamanho: $size MB"

    Write-Host "Criando pacote ZIP..." -ForegroundColor Yellow

    if (Test-Path $zipPath) {
        Remove-Item -Force $zipPath
    }

    $filesToZip = @($exePath)
    $licenseFile = "LICENSE"
    $thirdPartyFile = "THIRD-PARTY-LICENSES.md"

    if (Test-Path $licenseFile) {
        $filesToZip += $licenseFile
    } else {
        Write-Host "Aviso: $licenseFile nao encontrado" -ForegroundColor Yellow
    }

    if (Test-Path $thirdPartyFile) {
        $filesToZip += $thirdPartyFile
    } else {
        Write-Host "Aviso: $thirdPartyFile nao encontrado" -ForegroundColor Yellow
    }

    Compress-Archive -Path $filesToZip -DestinationPath $zipPath -Force

    $zipSize = [math]::Round((Get-Item $zipPath).Length / 1MB, 2)
    Write-Host "Pacote ZIP criado com sucesso!" -ForegroundColor Green
    Write-Host "Arquivo: $zipPath"
    Write-Host "Tamanho ZIP: $zipSize MB"
} else {
    Write-Host "Erro: Executavel nao foi gerado" -ForegroundColor Red
    exit 1
}
