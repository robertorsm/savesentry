use crate::models::GameProfile;
use crate::ui::Message;
use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length,
};

/// Renderiza a lista de perfis
pub fn render_profiles_list(profiles: &[GameProfile]) -> Element<'_, Message> {
    let profiles_column = profiles
        .iter()
        .fold(column![].spacing(10), |col, profile| {
            let status_text = if profile.is_active {
                "🟢 Monitorando"
            } else {
                "⚫ Inativo"
            };

            let toggle_button_text = if profile.is_active {
                "Parar"
            } else {
                "Iniciar"
            };

            let profile_row = row![
                column![
                    text(&profile.name).size(18),
                    text(&profile.save_path).size(12),
                ]
                .width(Length::Fill),
                text(status_text),
                button(text(toggle_button_text))
                    .on_press(Message::ToggleMonitoring(profile.id)),
                button(text("Excluir"))
                    .on_press(Message::DeleteProfile(profile.id))
                    .style(button::danger),
            ]
            .spacing(20)
            .align_y(iced::Alignment::Center);

            col.push(container(profile_row).padding(10).style(|_| {
                container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }
            }))
        });

    scrollable(profiles_column).height(Length::Fill).into()
}

