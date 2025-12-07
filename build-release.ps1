# Script de Build Otimizado - SaveGameWatcher
# Gera executável release otimizado para tamanho mínimo

Write-Host "=== SaveGameWatcher - Build Release Otimizado ===" -ForegroundColor Cyan
Write-Host ""

# Limpar builds anteriores
Write-Host "Limpando builds anteriores..." -ForegroundColor Yellow
cargo clean
Write-Host ""

# Build release com otimizações
Write-Host "Compilando em modo release (otimizado para tamanho)..." -ForegroundColor Yellow
Write-Host "Isso pode levar alguns minutos..." -ForegroundColor Gray
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Erro na compilação!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Build concluído com sucesso! ===" -ForegroundColor Green
Write-Host ""

# Informações do executável
$exePath = "target\release\SaveGameWatcher.exe"
if (Test-Path $exePath) {
    $fileInfo = Get-Item $exePath
    $sizeInMB = [math]::Round($fileInfo.Length / 1MB, 2)
    
    Write-Host "Executável gerado:" -ForegroundColor Cyan
    Write-Host "  Caminho: $exePath" -ForegroundColor White
    Write-Host "  Tamanho: $sizeInMB MB" -ForegroundColor White
    Write-Host ""
    
    Write-Host ""
    
    # Sugestão de compressão adicional com UPX (opcional)
    Write-Host "DICA: Para reduzir ainda mais o tamanho (50-70%), use UPX:" -ForegroundColor Yellow
    Write-Host "  1. Instale: winget install upx.upx" -ForegroundColor Gray
    Write-Host "  2. Comprima: upx --best --lzma $exePath" -ForegroundColor Gray
    Write-Host ""
}

Write-Host "Pronto para distribuição!" -ForegroundColor Green
