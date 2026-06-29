//! Componentes compartilhados entre páginas
//!
//! Apenas componentes globais:
//! - tab_bar: Barra de navegação entre abas
//! - messages: Exibição de mensagens de erro/sucesso
//!
//! Componentes específicos de cada página estão em pages/

mod messages;
mod tab_bar;

pub use messages::render_messages_banner;
pub use tab_bar::render_tab_bar;
