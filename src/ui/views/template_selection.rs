use crate::models::GameTemplate;
use crate::ui::Message;
use iced::{
    widget::{button, column, container, scrollable, text},
    Element, Length,
};

/// Renderiza a seção de seleção de templates
pub fn render_template_selection<'a>(
    templates: &'a [GameTemplate],
    selected_template: &'a Option<GameTemplate>,
) -> Element<'a, Message> {
    let templates_list = templates
        .iter()
        .fold(column![].spacing(10), |col, template| {
            let is_selected = selected_template.as_ref().map(|t| t.id) == Some(template.id);

            let btn_text = if is_selected {
                format!("✅ {}", template.name)
            } else {
                template.name.clone()
            };

            col.push(
                button(text(btn_text))
                    .on_press(Message::SelectTemplate(template.id))
                    .padding(5)
                    .width(Length::Fill),
            )
        });

    let content = column![
        text("Templates Disponíveis:").size(14),
        container(scrollable(templates_list).height(Length::Fixed(150.0)))
            .style(|_| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            })
            .padding(10),
        if selected_template.is_some() {
            button(text("Limpar Seleção (Criar Customizado)")).on_press(Message::ClearTemplate)
        } else {
            button(text("Nenhum template selecionado (Modo Customizado)"))
        }
    ]
    .spacing(10);

    content.into()
}

