//! Camada de UI (apresentação)
//!
//! Estrutura:
//! - app: Orquestração principal
//! - state: Estado centralizado da aplicação
//! - actions: Ações organizadas por contexto
//!   - monitoring: Ações de monitoramento e backup
//!   - templates: CRUD de templates
//! - pages: Páginas das 3 abas
//!   - main: Aba Principal
//!   - templates: Aba Templates
//!   - settings: Aba Configurações
//! - components: Componentes compartilhados
//!   - tab_bar: Barra de navegação
//!   - messages: Mensagens de notificação

mod actions;
mod app;
pub mod components;
pub mod pages;
mod state;

pub use app::App;
