# SaveSentry - Makefile
# Sistema de build e automação para Windows
# Data: 2025-12-13

.PHONY: help build-windows test-perf docs-update clean check fmt clippy run install dev all

# Variáveis
PROJECT_NAME = SaveSentry
RELEASE_PROFILE = release-windows
CARGO = cargo
PWSH = powershell.exe

# Cores para output (Windows PowerShell)
YELLOW = Write-Host -ForegroundColor Yellow
GREEN = Write-Host -ForegroundColor Green
RED = Write-Host -ForegroundColor Red
CYAN = Write-Host -ForegroundColor Cyan

#==============================================================================
# Help - Exibe todos os comandos disponíveis
#==============================================================================
help:
	@$(PWSH) -Command "$(CYAN) '=== SaveSentry - Comandos Disponíveis ==='"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'Build e Release:'"
	@$(PWSH) -Command "Write-Host '  make build-windows    - Build otimizado para Windows (release)'"
	@$(PWSH) -Command "Write-Host '  make dev              - Build debug rápido para desenvolvimento'"
	@$(PWSH) -Command "Write-Host '  make run              - Executar aplicação em modo debug'"
	@$(PWSH) -Command "Write-Host '  make install          - Build e copiar executável para diretório local'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'Qualidade de Código:'"
	@$(PWSH) -Command "Write-Host '  make check            - Verificar compilação sem build completo'"
	@$(PWSH) -Command "Write-Host '  make fmt              - Formatar código (cargo fmt)'"
	@$(PWSH) -Command "Write-Host '  make clippy           - Executar linter (cargo clippy)'"
	@$(PWSH) -Command "Write-Host '  make test             - Executar testes unitários'"
	@$(PWSH) -Command "Write-Host '  make validate         - Validação completa (fmt + clippy + check)'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'Performance e Documentação:'"
	@$(PWSH) -Command "Write-Host '  make test-perf        - Executar testes de performance'"
	@$(PWSH) -Command "Write-Host '  make docs-update      - Atualizar documentação automática'"
	@$(PWSH) -Command "Write-Host '  make docs-serve       - Abrir documentação local (README)'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'Limpeza e Manutenção:'"
	@$(PWSH) -Command "Write-Host '  make clean            - Limpar artefatos de build'"
	@$(PWSH) -Command "Write-Host '  make clean-all        - Limpeza completa (incluindo Cargo.lock)'"
	@$(PWSH) -Command "Write-Host '  make update-deps      - Atualizar dependências'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'Workflows Completos:'"
	@$(PWSH) -Command "Write-Host '  make all              - Pipeline completo (validate + build-windows)'"
	@$(PWSH) -Command "Write-Host '  make release-full     - Release completo (validate + build + test-perf + docs)'"
	@$(PWSH) -Command "$(YELLOW) ''"

#==============================================================================
# Gerar icone ICO a partir do PNG
#==============================================================================
icon:
	@$(PWSH) -Command "$(CYAN) '=== Gerando Icone ICO ==='"
	@$(PWSH) -Command "Set-Location 'tools/build-icon'; & 'cargo' run"
	@$(PWSH) -Command "$(GREEN) 'Icone gerado em assets/icon.ico'"

#==============================================================================
# Build para Windows (Release otimizado)
#==============================================================================
build-windows: icon
	@$(PWSH) -File "build-windows.ps1" -Profile "$(RELEASE_PROFILE)" -Project "$(PROJECT_NAME)"

#==============================================================================
# Build para Desenvolvimento (Debug rápido)
#==============================================================================
dev:
	@$(PWSH) -Command "$(CYAN) '=== Build Debug (Desenvolvimento) ==='"
	$(CARGO) build
	@$(PWSH) -Command "$(GREEN) '✓ Build debug concluído'"

#==============================================================================
# Executar aplicação
#==============================================================================
run:
	@$(PWSH) -Command "$(CYAN) '=== Executando $(PROJECT_NAME) ==='"
	$(CARGO) run

#==============================================================================
# Instalar executável localmente
#==============================================================================
install:
	@$(PWSH) -Command "$(CYAN) '=== Instalando $(PROJECT_NAME) ==='"
	@$(PWSH) -Command "if (-not (Test-Path 'bin')) { New-Item -ItemType Directory -Path 'bin' | Out-Null }"
	$(CARGO) build --profile $(RELEASE_PROFILE)
	@$(PWSH) -Command "Copy-Item 'target\$(RELEASE_PROFILE)\$(PROJECT_NAME).exe' 'bin\$(PROJECT_NAME).exe' -Force"
	@$(PWSH) -Command "$(GREEN) '✓ Executável copiado para bin\$(PROJECT_NAME).exe'"

#==============================================================================
# Verificação de Compilação (sem build completo)
#==============================================================================
check:
	@$(PWSH) -Command "$(CYAN) '=== Verificando Compilação ==='"
	$(CARGO) check
	@$(PWSH) -Command "$(GREEN) '✓ Verificação concluída'"

#==============================================================================
# Formatar Código
#==============================================================================
fmt:
	@$(PWSH) -Command "$(CYAN) '=== Formatando Código ==='"
	$(CARGO) fmt
	@$(PWSH) -Command "$(GREEN) '✓ Código formatado'"

#==============================================================================
# Linter (Clippy)
#==============================================================================
clippy:
	@$(PWSH) -Command "$(CYAN) '=== Executando Clippy ==='"
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	@$(PWSH) -Command "$(GREEN) '✓ Clippy concluído sem warnings'"

#==============================================================================
# Testes Unitários
#==============================================================================
test:
	@$(PWSH) -Command "$(CYAN) '=== Executando Testes Unitários ==='"
	$(CARGO) test
	@$(PWSH) -Command "$(GREEN) '✓ Testes concluídos'"

#==============================================================================
# Validação Completa (fmt + clippy + check)
#==============================================================================
validate:
	@$(PWSH) -Command "$(CYAN) '=== Validação Completa ==='"
	@$(PWSH) -Command "$(YELLOW) '[1/3] Formatação...'"
	@$(CARGO) fmt --check || ($(PWSH) -Command "$(RED) '✗ Código não formatado. Execute: make fmt'" && exit 1)
	@$(PWSH) -Command "$(GREEN) '  ✓ Código formatado'"
	@$(PWSH) -Command "$(YELLOW) '[2/3] Clippy...'"
	@$(CARGO) clippy --all-targets --all-features -- -D warnings
	@$(PWSH) -Command "$(GREEN) '  ✓ Sem warnings'"
	@$(PWSH) -Command "$(YELLOW) '[3/3] Verificação...'"
	@$(CARGO) check
	@$(PWSH) -Command "$(GREEN) '  ✓ Compilação OK'"
	@$(PWSH) -Command "$(GREEN) '✓ Validação completa passou!'"

#==============================================================================
# Testes de Performance
#==============================================================================
test-perf:
	@$(PWSH) -Command "$(CYAN) '=== Testes de Performance ==='"
	@$(PWSH) -Command "if (-not (Test-Path 'target\$(RELEASE_PROFILE)\$(PROJECT_NAME).exe')) { \
		$(YELLOW) 'Executável não encontrado. Compilando...'; \
		$(CARGO) build --profile $(RELEASE_PROFILE) \
	}"
	@$(PWSH) -Command "cd mydocs; .\performance_test.ps1"

#==============================================================================
# Atualizar Documentação Automática
#==============================================================================
docs-update:
	@$(PWSH) -Command "$(CYAN) '=== Atualizando Documentação ==='"
	@$(PWSH) -Command "$$timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm'; \
		$(YELLOW) \"Gerando documentação em: $$timestamp\"; \
		Write-Host ''; \
		$(YELLOW) 'Arquivos de documentação:'; \
		Write-Host '  - mydocs/ARCHITECTURE.md'; \
		Write-Host '  - mydocs/PERFORMANCE_TESTING.md'; \
		Write-Host '  - mydocs/WORKFLOW_WITH_PERF_TESTING.md'; \
		Write-Host ''; \
		$(GREEN) '✓ Documentação pronta para atualização manual'; \
		Write-Host ''; \
		$(YELLOW) 'LEMBRETE:'; \
		Write-Host '  Adicione entradas no changelog interno de cada arquivo afetado.'; \
		Write-Host '  Formato: [YYYY-MM-DD HH:MM] - Descrição da mudança'; \
		Write-Host ''"

#==============================================================================
# Abrir Documentação
#==============================================================================
docs-serve:
	@$(PWSH) -Command "$(CYAN) '=== Abrindo Documentação ==='"
	@$(PWSH) -Command "Start-Process 'README.md'"
	@$(PWSH) -Command "$(GREEN) '✓ README.md aberto no editor padrão'"

#==============================================================================
# Limpeza de Artefatos
#==============================================================================
clean:
	@$(PWSH) -Command "$(CYAN) '=== Limpando Artefatos de Build ==='"
	$(CARGO) clean
	@$(PWSH) -Command "if (Test-Path 'bin') { Remove-Item -Recurse -Force 'bin' }"
	@$(PWSH) -Command "$(GREEN) '✓ Artefatos removidos'"

#==============================================================================
# Limpeza Completa
#==============================================================================
clean-all: clean
	@$(PWSH) -Command "$(CYAN) '=== Limpeza Completa ==='"
	@$(PWSH) -Command "if (Test-Path 'Cargo.lock') { Remove-Item 'Cargo.lock' }"
	@$(PWSH) -Command "if (Test-Path 'target') { Remove-Item -Recurse -Force 'target' }"
	@$(PWSH) -Command "$(GREEN) '✓ Limpeza completa concluída'"

#==============================================================================
# Atualizar Dependências
#==============================================================================
update-deps:
	@$(PWSH) -Command "$(CYAN) '=== Atualizando Dependências ==='"
	$(CARGO) update
	@$(PWSH) -Command "$(GREEN) '✓ Dependências atualizadas'"
	@$(PWSH) -Command "$(YELLOW) 'Execute make check para verificar compatibilidade'"

#==============================================================================
# Pipeline Completo (Validação + Build)
#==============================================================================
all: validate build-windows
	@$(PWSH) -Command "$(GREEN) '✓ Pipeline completo executado com sucesso!'"

#==============================================================================
# Release Completo (Validação + Build + Testes + Docs)
#==============================================================================
release-full: validate build-windows test-perf
	@$(PWSH) -Command "$(CYAN) '=== Release Completo ==='"
	@$(PWSH) -Command "$(GREEN) '✓ Validação: PASSOU'"
	@$(PWSH) -Command "$(GREEN) '✓ Build Windows: CONCLUÍDO'"
	@$(PWSH) -Command "$(GREEN) '✓ Testes Performance: EXECUTADOS'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(YELLOW) 'PRÓXIMOS PASSOS:'"
	@$(PWSH) -Command "Write-Host '  1. Execute: make docs-update'"
	@$(PWSH) -Command "Write-Host '  2. Atualize changelogs internos em mydocs/'"
	@$(PWSH) -Command "Write-Host '  3. Commit e tag de release'"
	@$(PWSH) -Command "$(YELLOW) ''"
	@$(PWSH) -Command "$(GREEN) '✓ Release pronto para distribuição!'"

#==============================================================================
# Verificar Tamanho do Executável
#==============================================================================
size:
	@$(PWSH) -Command "if (Test-Path 'target\$(RELEASE_PROFILE)\$(PROJECT_NAME).exe') { \
		$$exe = Get-Item 'target\$(RELEASE_PROFILE)\$(PROJECT_NAME).exe'; \
		$$sizeMB = [math]::Round($$exe.Length / 1MB, 2); \
		$$sizeKB = [math]::Round($$exe.Length / 1KB, 2); \
		$(CYAN) '=== Tamanho do Executável ==='; \
		Write-Host \"Arquivo: $$($exe.Name)\"; \
		Write-Host \"Tamanho: $$sizeMB MB ($$sizeKB KB)\"; \
	} else { $(RED) '✗ Executável não encontrado. Execute: make build-windows' }"

#==============================================================================
# Gerar Relatório de Dependências
#==============================================================================
deps-report:
	@$(PWSH) -Command "$(CYAN) '=== Relatório de Dependências ==='"
	$(CARGO) tree --depth 1
	@$(PWSH) -Command "$(GREEN) '✓ Relatório gerado'"

#==============================================================================
# Verificar Atualizações Disponíveis
#==============================================================================
deps-outdated:
	@$(PWSH) -Command "$(CYAN) '=== Verificando Atualizações ==='"
	$(CARGO) update --dry-run
	@$(PWSH) -Command "$(YELLOW) 'Execute make update-deps para atualizar'"

