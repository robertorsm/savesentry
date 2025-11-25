# SaveGameWatcher

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)

**Sistema automático de backup para save games**

Monitore seus arquivos de save em tempo real e crie backups automáticos em formato ZIP.

[Características](#características) • [Instalação](#instalação) • [Uso](#uso) • [Arquitetura](#arquitetura) • [Desenvolvimento](#desenvolvimento)

</div>

---

## 📋 Sobre

**SaveGameWatcher** é uma aplicação desktop nativa desenvolvida em Rust que monitora automaticamente seus arquivos de save games e cria backups compactados sempre que detecta modificações. Com uma interface moderna e intuitiva, permite gerenciar múltiplos perfis de backup com configurações individualizadas.

### ✨ Características

- 🔄 **Monitoramento em Tempo Real**: Detecta mudanças nos arquivos de save automaticamente
- 📦 **Backups Compactados**: Cria arquivos ZIP com timestamp para fácil identificação
- ⏱️ **Controle de Timeout**: Configure intervalos mínimos entre backups para evitar excesso
- 🎮 **Templates Pré-configurados**: Suporte para jogos populares com paths automáticos
- 🎯 **Filtros de Exclusão**: Use regex para excluir arquivos temporários ou indesejados
- 💾 **Modo Portátil**: Banco de dados local, sem dependências do sistema
- 🖥️ **Interface Nativa**: UI responsiva e moderna com iced.rs
- 🌙 **Tema Escuro**: Interface otimizada para longas sessões

## 🚀 Instalação

### Pré-requisitos

- Windows 10 ou superior
- Nenhuma dependência externa necessária!

### Download

1. Baixe o instalador da [última release](../../releases/latest)
2. Execute `SaveGameWatcher_Setup.exe`
3. Siga as instruções do instalador

### Build do Código Fonte

```powershell
# Clone o repositório
git clone https://github.com/seu-usuario/SaveGameWatcher.git
cd SaveGameWatcher

# Compile em modo release
cargo build --release

# O executável estará em target/release/SaveGameWatcher.exe
```

## 📖 Uso

### Início Rápido

1. **Abra a aplicação**
   ```powershell
   ./SaveGameWatcher.exe
   ```

2. **Selecione um template (opcional)**
   - Clique em um dos jogos pré-configurados
   - Ou continue para criar um perfil customizado

3. **Configure seu perfil**
   - Nome do jogo
   - Localização do arquivo de save
   - Diretório onde os backups serão salvos
   - Intervalo mínimo entre backups (em minutos)

4. **Inicie o monitoramento**
   - Clique em "Criar Perfil"
   - Clique em "Iniciar" no perfil criado
   - Status mudará para 🟢 Monitorando

### Gerenciamento de Perfis

- **Iniciar/Parar**: Alterna monitoramento do perfil
- **Excluir**: Remove perfil permanentemente
- **Status**: 
  - 🟢 Monitorando: Backup ativo
  - ⚫ Inativo: Aguardando ativação

### Templates Suportados

Templates pré-configurados para jogos populares incluem expansão automática de variáveis:
- `%APPDATA%` - Dados de aplicação do usuário
- `%USERPROFILE%` - Pasta home do usuário
- `%LOCALAPPDATA%` - Dados locais da aplicação

### Formato dos Backups

Os backups são criados com nomenclatura padronizada:
```
backup_DD-MM-YYYY_HH-MM-SS.zip
```

Exemplo: `backup_24-11-2025_15-30-45.zip`

## 🏗️ Arquitetura

### Visão Geral

O projeto segue a **Elm Architecture (TEA)** com separação clara de responsabilidades:

```
┌─────────────────────────────────────────┐
│  PRESENTATION (UI)                      │  ← iced.rs
│  - Widgets reativos                     │
│  - Message-driven updates               │
└──────────────┬──────────────────────────┘
               ↓
┌──────────────────────────────────────────┐
│  APPLICATION (App)                       │  ← Orquestração
│  - State management                      │
│  - Message handlers                      │
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
| UI Framework | iced | 0.13 | Interface gráfica reativa |
| Database | rusqlite | 0.32 | Persistência SQLite |
| Migrations | refinery | 0.8 | Schema versioning |
| File Monitoring | notify | 6.1 | File system watching |
| Async Runtime | tokio | 1.0 | Operações assíncronas |
| Compression | zip | 0.6 | Criação de backups |
| Date/Time | chrono | 0.4 | Timestamps |
| Pattern Matching | regex | 1.10 | Filtros de exclusão |

### Padrões de Projeto

- **Repository Pattern**: Abstração do acesso a dados
- **Command Pattern**: Sistema de mensagens tipadas
- **Observer Pattern**: Subscription para file watching
- **Factory Method**: Criação de perfis e templates
- **Strategy Pattern**: Filtros configuráveis com regex
- **State Machine**: Gerenciamento de estado reativo
- **Component Pattern**: UI modular com componentes reutilizáveis
- **Pure Functions**: Views sem side effects para testabilidade

Para detalhes completos, consulte [`.copilot-memory.md`](.copilot-memory.md).

## 💻 Desenvolvimento

### Estrutura do Projeto

```
SaveGameWatcher/
├── src/
│   ├── main.rs                 # Entry point
│   ├── ui/                     # Presentation layer
│   │   ├── mod.rs              # Módulo UI
│   │   ├── app.rs              # Estado e lógica (95 linhas)
│   │   ├── view.rs             # View principal (49 linhas)
│   │   ├── messages.rs         # Message types
│   │   └── views/              # Componentes UI
│   │       ├── mod.rs          # Re-exports
│   │       ├── template_selection.rs  # Seleção de templates
│   │       ├── profile_form.rs        # Formulário de perfil
│   │       └── profile_list.rs        # Lista de perfis
│   ├── models/                 # Domain layer
│   │   ├── game_profile.rs     # Perfil de jogo
│   │   └── game_template.rs    # Template de jogo
│   ├── db/                     # Infrastructure layer
│   │   ├── database.rs         # Repository
│   │   └── migrations/         # SQL migrations
│   └── watcher/                # Background processing
│       ├── file_watcher.rs     # Lógica de backup
│       └── subscription.rs     # Stream reativa
├── resources/
│   └── sgw.db                  # Banco embarcado
├── build.rs                    # Build script
├── Cargo.toml                  # Dependências
└── installer.iss               # Inno Setup script
```

> **Nota sobre Arquitetura UI**: O módulo `ui/` foi refatorado seguindo princípios SOLID:
> - `app.rs`: Core da aplicação (estado + lógica)
> - `view.rs`: Orquestração de componentes
> - `views/`: Componentes UI reutilizáveis e isolados
> 
> Cada componente é uma **pure function**, facilitando testes e manutenção.

### Comandos Úteis

```powershell
# Verificar código
cargo check

# Compilar (debug)
cargo build

# Compilar (release otimizado)
cargo build --release

# Executar
cargo run

# Verificar qualidade (linter)
cargo clippy

# Formatar código
cargo fmt

# Executar testes
cargo test
```

### Build de Release

O profile de release está otimizado para distribuição:

```toml
[profile.release]
opt-level = "z"        # Tamanho mínimo
lto = true             # Link Time Optimization
codegen-units = 1      # Otimização cross-function
strip = true           # Remove símbolos de debug
panic = "abort"        # Reduz unwinding code
```

Resultado: Executável compacto (~10-20 MB) e altamente otimizado.

### Adicionar Novo Template

1. Edite `src/db/migrations/V2__seed_game_templates.sql`:
```sql
INSERT INTO game_templates (name, save_directory, process_name, save_pattern, ...)
VALUES ('Meu Jogo', '%APPDATA%\MeuJogo\saves', 'jogo.exe', '*.sav', ...);
```

2. Recompile - migrations são aplicadas automaticamente

### Adicionar Nova Feature

1. Adicione variant em `src/ui/messages.rs`:
```rust
pub enum Message {
    // ...existing variants...
    MinhaNovaAcao(String),
}
```

2. Implemente handler em `src/ui/app.rs`:
```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        // ...existing matches...
        Message::MinhaNovaAcao(valor) => {
            // Lógica aqui
        }
    }
    Task::none()
}
```

3. Adicione trigger na UI em `view()`:
```rust
button("Ação").on_press(Message::MinhaNovaAcao(String::from("valor")))
```

## 🔧 Configuração Avançada

### Variáveis de Ambiente

O sistema expande automaticamente:
- `%APPDATA%` → `C:\Users\[User]\AppData\Roaming`
- `%LOCALAPPDATA%` → `C:\Users\[User]\AppData\Local`
- `%USERPROFILE%` → `C:\Users\[User]`

### Regex de Exclusão

Exemplos de padrões úteis:

```regex
.*\.tmp$           # Exclui arquivos .tmp
.*_backup.*        # Exclui arquivos com "_backup"
^temp.*            # Exclui arquivos começando com "temp"
.*(cache|log).*    # Exclui arquivos com "cache" ou "log"
```

### Modo Debug

Para logs detalhados durante desenvolvimento:

```powershell
$env:RUST_LOG="debug"
cargo run
```

## 📊 Performance

### Métricas

- **Tempo de startup**: ~2-3 segundos
- **Consumo de memória**: ~50-100 MB
- **CPU (idle)**: < 1%
- **CPU (durante backup)**: ~30-50% (temporário)
- **Detecção de mudanças**: < 100ms (via notify/inotify)

### Otimizações

- Cache de templates em memória
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
icacls SaveGameWatcher.exe
```

### Backup não está sendo criado

**Causas possíveis**:
1. Perfil não está ativo (status ⚫)
2. Timeout ainda não expirou
3. Arquivo não foi modificado
4. Filtro de exclusão está bloqueando

**Verificação**:
```powershell
# Confirmar que arquivo foi modificado
Get-Item "caminho\do\save.sav" | Select-Object LastWriteTime
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

- [iced](https://github.com/iced-rs/iced) - Framework GUI reativo
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

