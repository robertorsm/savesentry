# SaveSentry

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)

![SaveSentry Logo](assets/readme_image.png)

**Sistema automático de backup para save games**

Monitore seus arquivos de save em tempo real e crie backups automáticos em formato ZIP.

[Características](#características) • [Instalação](#instalação) • [Uso](#uso) • [Arquitetura](#arquitetura) • [Desenvolvimento](#desenvolvimento)

</div>

---

## 📋 Sobre

**SaveSentry** é uma aplicação desktop nativa desenvolvida em Rust que monitora automaticamente seus arquivos de save games e cria backups compactados sempre que detecta modificações. Com uma interface moderna e intuitiva, permite gerenciar perfis de backup com configurações individualizadas — um perfil ativo por vez.

### ✨ Características

- 🔄 **Monitoramento em Tempo Real**: Detecta mudanças nos arquivos de save automaticamente
- 📦 **Backups Compactados**: Cria arquivos ZIP com timestamp para fácil identificação
- ⏱️ **Controle de Timeout**: Configure intervalos mínimos entre backups para evitar excesso
- 🎮 **Templates Pré-configurados**: 11 jogos populares com paths e processos automáticos
- 🎯 **Filtros de Exclusão**: Use glob patterns para excluir arquivos temporários ou indesejados
- 💾 **Modo Portátil**: Banco de dados local, sem dependências do sistema
- 🖥️ **Interface Nativa**: UI responsiva e moderna com egui
- 🌙 **Tema Escuro**: Interface otimizada para longas sessões
- 🎮 **Monitoramento por Processo**: Só monitora quando o jogo está em execução
- 📸 **Captura de Screenshots**: Salva screenshot do momento do backup com thumbnail e visualização full-res
- 🔄 **Restauração de Backups**: Restaure saves anteriores com backup de segurança automático
- ✏️ **Renomear Backups**: Marque backups importantes com nomes personalizados (não são rotacionados)
- 📊 **Limite de Backups**: Rotação automática com limite configurável (padrão: 50 automáticos)
- 📁 **Diretório de Backup Padrão**: Configure uma pasta base para organizar backups automaticamente em subpastas por jogo
- 🛡️ **Ícone Personalizado**: Logo de sentinela embutido no executável
- 📦 **Build ZIP Automático**: Gera pacote ZIP pronto para distribuição
- ⚡ **Ultra Leve**: Executável otimizado (~5.5 MB), consumo mínimo de recursos

## 🚀 Instalação

### Pré-requisitos

- Windows 10 ou superior
- Nenhuma dependência externa necessária!

### Download

1. Baixe o executável da [última release](../../releases/latest)
2. Execute `SaveSentry.exe` diretamente — não requer instalação

### Build do Código Fonte

```powershell
# Clone o repositório
git clone https://github.com/seu-usuario/SaveSentry.git
cd SaveSentry

# Compile em modo release otimizado para Windows
cargo build --profile release-windows

# Ou use make para build completo com icone e ZIP:
make build-windows

# O executável estará em target/release-windows/SaveSentry.exe
```

## 📖 Uso

### Início Rápido

1. **Abra a aplicação**
   ```powershell
   ./SaveSentry.exe
   ```

2. **Selecione um template (opcional)**
   - Clique em um dos jogos pré-configurados
   - Ou continue para criar um perfil customizado

3. **Configure seu perfil**
   - Nome do jogo
   - Localização do arquivo de save
   - Diretório onde os backups serão salvos (ou use o diretório padrão)
   - Intervalo mínimo entre backups (em minutos)
   - Padrão de exclusão (opcional, para ignorar arquivos temporários)
   - Nome do processo (opcional, para monitorar apenas quando o jogo está rodando)

4. **Inicie o monitoramento**
   - Clique em "Criar Perfil"
   - Clique em "Iniciar" no perfil criado
   - Status mudará para 🟢 Monitorando

### Gerenciamento de Perfis

- **Iniciar/Parar**: Alterna monitoramento do perfil
- **Excluir**: Remove perfil permanentemente
- **Editar**: Altere nome, paths, timeout, exclusões e processo
- **Status**: 
  - 🟢 Monitorando: Backup ativo
  - ⚫ Inativo: Aguardando ativação

### Gerenciamento de Backups

Na aba principal, o painel de histórico mostra todos os backups do perfil ativo:

- **Screenshot**: Thumbnail da tela no momento do backup — clique para visualização full-res
- **Restaurar**: Clique com botão direito no backup → "Restaurar" (cria backup de segurança `BeforeRestore_` automaticamente)
- **Renomear**: Clique com botão direito → "Renomear" para marcar como favorito (não será rotacionado)
- **Excluir**: Clique com botão direito → "Excluir"
- **Contadores**: "Backups:" mostra automáticos vs limite; "Fixados:" mostra renomeados

### Templates Suportados

Templates pré-configurados para 11 jogos populares com expansão automática de variáveis:
- `%APPDATA%` - Dados de aplicação do usuário
- `%USERPROFILE%` - Pasta home do usuário
- `%LOCALAPPDATA%` - Dados locais da aplicação
- `%STEAM_USERDATA%` - Pasta userdata do Steam (detectado automaticamente)
- `%STEAMID%` - ID da conta Steam (primeira pasta numérica em userdata)
- `%PROGRAMFILES%` e `%PROGRAMFILES(X86)%` - Programas
- `%PROGRAMDATA%` - Dados compartilhados entre usuários
- `%PUBLIC%` - Pasta pública
- `%TEMP%` / `%TMP%` - Pasta temporária
- `%HOMEDRIVE%` / `%HOMEPATH%` - Drive e caminho home

### Formato dos Backups

Os backups automáticos usam nomenclatura padronizada:
```
backup_DD-MM-YYYY_HH-MM-SS.zip
```

Exemplo: `backup_24-11-2025_15-30-45.zip`

Backups renomeados mantêm o nome escolhido pelo usuário e não entram na rotação automática.

### Aba Configurações

Acesse a terceira aba para ajustes globais:

- **Diretório padrão de backup**: Pasta base usada quando o perfil não define um diretório específico (organiza automaticamente em subpastas por jogo)
- **Configurações gerais**: Preferências de comportamento do app

## 🏗️ Arquitetura

### Visão Geral

O projeto utiliza **Immediate Mode UI** com arquitetura simples e direta:

```
┌─────────────────────────────────────────┐
│  PRESENTATION (UI)                      │  ← egui
│  - Immediate mode rendering             │
│  - Direct state mutation                │
└──────────────┬──────────────────────────┘
               ↓
┌──────────────────────────────────────────┐
│  APPLICATION (App)                       │  ← Estado e lógica
│  - State management                      │
│  - UI + Logic em um só lugar            │
└──────────────┬──────────────────────────┘
               ↓
┌──────────────────────────────────────────┐
│  DOMAIN (Models + Watcher)              │  ← Lógica de negócio
│  - GameProfile, GameTemplate             │
│  - FileWatcher (backup logic)            │
└──────────────┬──────────────────────────┘
               ↓
┌──────────────────────────────────────────┐
│  INFRASTRUCTURE (Database)               │  ← SQLite
│  - Persistence layer                     │
│  - Migrations                            │
└─────────────────────────────────────────┘
```

### Tecnologias

| Componente | Biblioteca | Versão | Propósito |
|------------|-----------|--------|-----------|
| UI Framework | egui + eframe | 0.35.0 | Interface gráfica immediate mode |
| Database | rusqlite | 0.39.0 | Persistência SQLite |
| Migrations | refinery | 0.9.2 | Schema versioning |
| File Monitoring | notify | 9.0.0-rc.4 | File system watching |
| Compression | zip | 0.6 | Criação de backups ZIP |
| Date/Time | chrono | 0.4.45 | Timestamps |
| Pattern Matching | glob | 0.3 | Filtros de exclusão (glob patterns) |
| File Dialog | rfd | 0.17.2 | Diálogos de arquivo nativos |
| Process Info | sysinfo | 0.39.5 | Detecção de processos do jogo |
| Screenshots | screenshots + image | 0.8 + 0.24 | Captura de tela no backup |
| Error Handling | anyhow | 1.0.103 | Tratamento de erros Rust |
| Icon Resource | winresource | 0.1 | Ícone no executável Windows |

### Padrões de Projeto

- **Repository Pattern**: Abstração do acesso a dados
- **Immediate Mode UI**: Renderização e lógica unificadas
- **Observer Pattern**: File watching com threads
- **Factory Method**: Criação de perfis e templates
- **Strategy Pattern**: Filtros configuráveis com glob patterns
- **Thread-based Background**: Watchers em threads separadas
- **Component Pattern**: UI modular com componentes reutilizáveis
- **Pure Functions**: Views sem side effects para testabilidade
- **LRU Cache**: Cache de screenshots com limite de memória GPU


## 💻 Desenvolvimento

### Estrutura do Projeto

```
SaveSentry/
├── src/
│   ├── main.rs                 # Entry point
│   ├── ui/                     # Presentation layer
│   │   ├── mod.rs              # Módulo UI
│   │   ├── app.rs              # Orquestração (~70 linhas)
│   │   ├── state.rs            # Estado centralizado
│   │   ├── actions/            # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── monitoring.rs   # Monitoramento, backup e restore
│   │   │   └── templates.rs    # CRUD de templates
│   │   ├── components/         # Componentes compartilhados
│   │   │   ├── mod.rs
│   │   │   ├── tab_bar.rs      # Barra de navegação
│   │   │   └── messages.rs     # Mensagens de notificação
│   │   └── pages/              # Páginas das 3 abas
│   │       ├── mod.rs
│   │       ├── main/           # Aba Principal
│   │       │   ├── mod.rs
│   │       │   ├── config_panel.rs   # Painel de configuração
│   │       │   ├── save_info.rs      # Informações do save
│   │       │   └── backup_history.rs # Histórico de backups
│   │       ├── templates/      # Aba Templates
│   │       │   ├── mod.rs
│   │       │   └── manager.rs        # Gerenciador de templates
│   │       └── settings/       # Aba Configurações
│   │           ├── mod.rs
│   │           └── panel.rs          # Painel de settings
│   ├── models/                 # Domain layer
│   │   ├── mod.rs
│   │   ├── game_profile.rs     # Perfil de jogo
│   │   └── game_template.rs    # Template de jogo
│   ├── db/                     # Infrastructure layer
│   │   ├── mod.rs
│   │   ├── database.rs         # Repository
│   │   └── migrations/         # SQL migrations (V1 consolidado)
│   └── watcher/                # Background processing
│       ├── mod.rs
│       ├── file_watcher.rs     # Lógica de backup, ZIP e screenshots
│       ├── simple_watcher.rs   # Thread-based watching
│       └── process_monitor.rs  # Monitoramento de processos
├── assets/                     # Ícones e imagens
│   ├── logo.svg                # Logo vetorial
│   ├── exec_icon.png           # Ícone do executável
│   ├── readme_image.png        # Imagem do README
│   └── icon.ico                # Ícone embutido no .exe
├── tools/                      # Ferramentas de build
│   └── build-icon/             # Conversor PNG -> ICO
├── build.rs                    # Build script (migrations + icone)
├── build-windows.ps1           # Script de build para Windows
├── Makefile                    # Comandos de automação
└── Cargo.toml                  # Dependências
```

> **Nota sobre Arquitetura UI**: O projeto usa **egui** com paradigma Immediate Mode:
> - `app.rs`: Apenas orquestração (composição de componentes)
> - `state.rs`: Estado centralizado organizado em sub-structs
> - `actions/`: Lógica de negócio separada da UI
> - `pages/` e `components/`: UI modular e reutilizável
> - Ideal para utilitários leves como este

### Comandos Úteis

```powershell
# Verificar código
cargo check
make check

# Compilar (debug)
cargo build
make dev

# Compilar (release otimizado para Windows)
cargo build --profile release-windows
make build-windows    # Gera icone + compila + cria ZIP

# Executar
cargo run
make run

# Instalar executável localmente (copia para bin/)
make install

# Verificar qualidade (linter)
cargo clippy --all-targets --all-features -- -D warnings
make clippy

# Formatar código
cargo fmt
make fmt

# Executar testes
cargo test
make test

# Validação completa (fmt --check + clippy + check)
make validate

# Pipeline completo (validate + build-windows)
make all

# Release completo (validate + build + test-perf)
make release-full

# Testes de performance
make test-perf

# Gerar icone ICO a partir do PNG
make icon

# Verificar tamanho do executável
make size

# Relatório de dependências
make deps-report

# Verificar atualizações disponíveis
make deps-outdated

# Atualizar dependências
make update-deps

# Limpeza de artefatos
make clean
make clean-all    # Limpeza completa (inclui Cargo.lock)

# Ajuda com todos os comandos
make help
```

### Build de Release

O projeto usa o profile `release-windows` otimizado para distribuição:

```toml
[profile.release-windows]
opt-level = "z"        # Otimizar para tamanho mínimo
lto = "fat"            # Link Time Optimization máximo
codegen-units = 1      # Melhor otimização cross-function
strip = true           # Remove símbolos de debug
panic = "abort"        # Reduz unwinding code
incremental = false    # Desabilitar compilação incremental
overflow-checks = false # Desabilitar checks de overflow
debug = false          # Sem informações de debug
debug-assertions = false # Desabilitar assertions de debug
rpath = false          # Não incluir rpath
inherits = "release"   # Herdar configurações base do perfil release
```

Resultado: Executável compacto (~5.5-6 MB) e altamente otimizado.

### Build com Make (Recomendado)

```powershell
# Build release completo (gera icone + compila + cria ZIP)
make build-windows

# O pacote ZIP estará em: dist/SaveSentry.zip
```

### Adicionar Novo Template

1. Edite a migration `V1__initial_schema.sql` na seção de seeds (apenas em desenvolvimento local):
```sql
INSERT INTO game_templates (
    name, save_directory, process_name, save_pattern, exclude_pattern,
    default_exclude_pattern, backup_dir, backup_delay_minutes, backup_max_count,
    version, is_official, created_at
)
VALUES (
    'Meu Jogo', '%APPDATA%\MeuJogo\saves', 'jogo.exe', '*.sav', NULL,
    'steam_autocloud.vdf', '%USERPROFILE%\SaveSentry\Meu Jogo',
    5, 50, 1, 1, datetime('now')
);
```

> **Nota:** Em produção, crie uma nova migration (ex: `V2__add_meujogo_template.sql`). Nunca edite migrations já aplicadas em bancos de dados existentes.

2. Recompile - migrations são aplicadas automaticamente

### Adicionar Nova Feature

1. **Lógica de negócio:** Adicione em `src/ui/actions/` (monitoring.rs ou templates.rs) ou crie um novo arquivo:
```rust
// src/ui/actions/minha_feature.rs
impl AppState {
    pub fn minha_feature(&mut self) -> anyhow::Result<()> {
        // Implementar feature aqui
        Ok(())
    }
}
```

2. **Estado:** Adicione campos necessários em `src/ui/state.rs`:
```rust
pub struct AppState {
    // ... campos existentes ...
    pub meu_campo: String,
}
```

3. **UI:** Adicione componente em `src/ui/pages/` ou `src/ui/components/`. **Nunca adicione lógica em `app.rs`** — ele é apenas para orquestração.

4. Teste e compile

## 🔧 Configuração Avançada

### Variáveis de Ambiente

O sistema expande automaticamente variáveis do Windows e paths do Steam:

| Variável | Exemplo Expandido | Fonte |
|----------|------------------|-------|
| `%APPDATA%` | `C:\Users\User\AppData\Roaming` | Windows env |
| `%LOCALAPPDATA%` | `C:\Users\User\AppData\Local` | Windows env |
| `%USERPROFILE%` | `C:\Users\User` | Windows env |
| `%USERNAME%` | `User` | Windows env |
| `%HOMEDRIVE%` | `C:` | Windows env |
| `%HOMEPATH%` | `\Users\User` | Windows env |
| `%PROGRAMFILES%` | `C:\Program Files` | Windows env |
| `%PROGRAMFILES(X86)%` | `C:\Program Files (x86)` | Windows env |
| `%PROGRAMDATA%` | `C:\ProgramData` | Windows env |
| `%PUBLIC%` | `C:\Users\Public` | Windows env |
| `%TEMP%` / `%TMP%` | `C:\Users\User\AppData\Local\Temp` | Windows env |
| `%STEAM_USERDATA%` | `C:\Program Files (x86)\Steam\userdata` | Detectado em runtime |
| `%STEAMID%` | `76561198000000000` | Primeira pasta numérica em `userdata/` |

> **Detecção Steam**: O app busca `Steam\userdata` em `%LOCALAPPDATA%`, `%ProgramFiles(x86)%`, `%ProgramFiles%` e `%USERPROFILE%`, usando a primeira subpasta totalmente numérica como SteamID.

### Filtros de Exclusão (Glob Patterns)

Use padrões glob (não regex) para excluir arquivos:

```
*.tmp              # Exclui arquivos .tmp
*_backup*          # Exclui arquivos com "_backup"
temp*              # Exclui arquivos começando com "temp"
*cache*            # Exclui arquivos com "cache"
session.lock       # Exclui arquivo específico
```

Use `|` para múltiplos padrões: `*.tmp|*.lock|temp*`

### Modo Debug

Para logs detalhados durante desenvolvimento:

```powershell
$env:RUST_LOG="debug"
cargo run
```

## 📊 Performance

### Métricas

- **Tempo de startup**: ~2-3 segundos
- **Tamanho do executável**: ~5.5 MB
- **Consumo de memória**: ~30-50 MB
- **CPU (idle)**: < 1%
- **CPU (durante backup)**: ~30-50% (temporário)
- **Detecção de mudanças**: < 100ms (via notify/inotify)

### Otimizações

- Cache de templates em memória
- LRU cache de screenshots (máx. 1 full-res + 2 thumbs na GPU)
- Reutilização de `sysinfo::System` (evita recriação a cada poll)
- SQLite cache reduzido para 512KB
- Threads de watcher com stack de 512KB
- Stream lazy para processamento de eventos
- HashMap para lookup O(1) de watchers
- Zero-cost abstractions do Rust
- Iteradores em vez de loops

## 🛡️ Segurança

- ✅ Sem coleta de telemetria
- ✅ Sem conexão com internet
- ✅ Dados armazenados localmente
- ✅ Modo portátil (não usa registry)
- ✅ Type-safe (verificações em tempo de compilação)

## 🐛 Troubleshooting

### Banco de dados não inicializa

**Solução**: Verifique permissões de escrita no diretório do executável.

```powershell
# Verificar permissões
icacls SaveSentry.exe
```

### Backup não está sendo criado

**Causas possíveis**:
1. Perfil não está ativo (status ⚫)
2. Timeout ainda não expirou
3. Arquivo não foi modificado
4. Filtro de exclusão está bloqueando
5. Processo do jogo não está em execução (se configurado)
6. Limite de backups foi atingido e rotação falhou

**Verificação**:
```powershell
# Confirmar que arquivo foi modificado
Get-Item "caminho\do\save.sav" | Select-Object LastWriteTime

# Verificar se processo está rodando (ex: StardewValley.exe)
Get-Process | Where-Object { $_.ProcessName -like "*Stardew*" }
```

### Aplicação lenta

**Soluções**:
1. Use versão release (não debug)
2. Reduza número de perfis ativos
3. Aumente timeout (menos backups)

## 🤝 Contribuindo

Contribuições são bem-vindas! Por favor:

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/MinhaFeature`)
3. Commit suas mudanças (`git commit -m 'Adiciona MinhaFeature'`)
4. Push para a branch (`git push origin feature/MinhaFeature`)
5. Abra um Pull Request

### Guidelines

- Siga convenções de código Rust (use `cargo fmt`)
- Adicione testes quando aplicável
- Documente APIs públicas
- Mantenha commits atômicos e descritivos

## 📄 Licença

Este projeto está sob a licença MIT. Veja o arquivo [LICENSE](LICENSE) para detalhes.

## 👨‍💻 Autor

Desenvolvido com ❤️ em Rust

## 🙏 Agradecimentos

- [egui](https://github.com/emilk/egui) - Immediate-mode GUI (eframe)
- [notify](https://github.com/notify-rs/notify) - Cross-platform file watching
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- Comunidade Rust Brasil

## 📮 Contato

- Issues: [GitHub Issues](../../issues)
- Discussions: [GitHub Discussions](../../discussions)

---

<div align="center">

**Se este projeto foi útil, considere dar uma ⭐!**

</div>
