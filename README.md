# SaveSentry

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

**SaveSentry** é uma aplicação desktop nativa desenvolvida em Rust que monitora automaticamente seus arquivos de save games e cria backups compactados sempre que detecta modificações. Com uma interface moderna e intuitiva, permite gerenciar múltiplos perfis de backup com configurações individualizadas.

### ✨ Características

- 🔄 **Monitoramento em Tempo Real**: Detecta mudanças nos arquivos de save automaticamente
- 📦 **Backups Compactados**: Cria arquivos ZIP com timestamp para fácil identificação
- ⏱️ **Controle de Timeout**: Configure intervalos mínimos entre backups para evitar excesso
- 🎮 **Templates Pré-configurados**: Suporte para jogos populares com paths automáticos
- 🎯 **Filtros de Exclusão**: Use regex para excluir arquivos temporários ou indesejados
- 💾 **Modo Portátil**: Banco de dados local, sem dependências do sistema
- 🖥️ **Interface Nativa**: UI responsiva e moderna com egui
- 🌙 **Tema Escuro**: Interface otimizada para longas sessões
- ⚡ **Ultra Leve**: Apenas 5.3 MB, consumo mínimo de recursos

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

# Compile em modo release
cargo build --release

# O executável estará em target/release/SaveSentry.exe
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
| UI Framework | egui + eframe | 0.33 | Interface gráfica immediate mode |
| Database | rusqlite | 0.37 | Persistência SQLite |
| Migrations | refinery | 0.9 | Schema versioning |
| File Monitoring | notify | 8.2 | File system watching |
| Compression | zip | 6.0 | Criação de backups |
| Date/Time | chrono | 0.4 | Timestamps |
| Pattern Matching | regex | 1.12 | Filtros de exclusão |

### Padrões de Projeto

- **Repository Pattern**: Abstração do acesso a dados
- **Immediate Mode UI**: Renderização e lógica unificadas
- **Observer Pattern**: File watching com threads
- **Factory Method**: Criação de perfis e templates
- **Strategy Pattern**: Filtros configuráveis com regex
- **Thread-based Background**: Watchers em threads separadas
- **Component Pattern**: UI modular com componentes reutilizáveis
- **Pure Functions**: Views sem side effects para testabilidade


## 💻 Desenvolvimento

### Estrutura do Projeto

```
SaveSentry/
├── src/
│   ├── main.rs                 # Entry point
│   ├── ui/                     # Presentation layer
│   │   ├── app.rs              # Orquestração (~70 linhas)
│   │   ├── state.rs            # Estado centralizado
│   │   ├── actions/            # Business logic
│   │   │   ├── monitoring.rs   # Monitoramento e backup
│   │   │   └── templates.rs    # CRUD de templates
│   │   ├── components/         # Componentes compartilhados
│   │   │   ├── tab_bar.rs      # Barra de navegação
│   │   │   └── messages.rs     # Mensagens de notificação
│   │   └── pages/              # Páginas das 3 abas
│   │       ├── main/           # Aba Principal
│   │       ├── templates/      # Aba Templates
│   │       └── settings/       # Aba Configurações
│   ├── models/                 # Domain layer
│   │   ├── game_profile.rs     # Perfil de jogo
│   │   └── game_template.rs    # Template de jogo
│   ├── db/                     # Infrastructure layer
│   │   ├── database.rs         # Repository
│   │   └── migrations/         # SQL migrations
│   └── watcher/                # Background processing
│       ├── file_watcher.rs     # Lógica de backup
│       ├── simple_watcher.rs   # Thread-based watching
│       └── process_monitor.rs  # Monitoramento de processos
├── build.rs                    # Build script
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
opt-level = "s"        # Balanço tamanho/performance
lto = true             # Link Time Optimization
codegen-units = 1      # Otimização cross-function
strip = true           # Remove símbolos de debug
panic = "abort"        # Reduz unwinding code
incremental = false    # Remove overhead
```

Resultado: Executável compacto (**5.3 MB**) e altamente otimizado.

### Adicionar Novo Template

1. Edite `src/db/migrations/V2__seed_game_templates.sql`:
```sql
INSERT INTO game_templates (name, save_directory, process_name, save_pattern, ...)
VALUES ('Meu Jogo', '%APPDATA%\MeuJogo\saves', 'jogo.exe', '*.sav', ...);
```

2. Recompile - migrations são aplicadas automaticamente

### Adicionar Nova Feature

1. Adicione lógica em `src/ui/app.rs` no método `update()`:
```rust
// Dentro do método update() da trait eframe::App
egui::CentralPanel::default().show(ctx, |ui| {
    if ui.button("Nova Ação").clicked() {
        // Implementar feature aqui
        self.meu_campo = novo_valor;
    }
});
```

2. Se necessário, adicione novo campo no `struct App`
3. Teste e compile

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
icacls SaveSentry.exe
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
