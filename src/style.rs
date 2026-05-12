use iced::{
    Background, Border, Color, Theme,
    border::Radius,
    widget::{
        button,
        slider::{Handle, Rail},
    },
};

pub struct Palette;

impl Palette {
    pub const DARKEST_BACKGROUND: Color = Color::from_rgb8(34, 37, 48);
    pub const DARK_BACKGROUND: Color = Color::from_rgb8(74, 74, 74);
    pub const DARK_BORDER: Color = Color::from_rgb8(30, 30, 30);
    pub const TEXT_ON_DARK: Color = Color::from_rgb8(237, 237, 237);
    pub const DARK_BUTTON: Color = Color::from_rgb8(122, 130, 153);
    pub const RAIL_ACTIVE: Color = Color::from_rgb8(240, 234, 129);
    pub const RAIL_DEACTIVE: Color = Color::from_rgb8(184, 181, 129);
    pub const BLUE_BACKGROUND: Color = Color::from_rgb8(84, 91, 133);
    pub const LIGHT_GRAY: Color = Color::from_rgb8(209, 209, 209);
    pub const PROGRESS_ORANGE: Color = Color::from_rgb8(252, 123, 3);
    pub const ERROR_RED: Color = Color::from_rgb8(176, 23, 0);
}

pub fn container_dark(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::DARKEST_BACKGROUND)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 0.0,
            ..Default::default()
        },
        snap: true,
        ..Default::default()
    }
}

pub fn container_medium(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::DARK_BACKGROUND)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 0.0,
            ..Default::default()
        },
        snap: true,
        ..Default::default()
    }
}

pub fn bottom_bar_in_progress(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::PROGRESS_ORANGE)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 0.0,
            ..Default::default()
        },
        snap: true,
        ..Default::default()
    }
}

pub fn bottom_bar_error(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::ERROR_RED)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 0.0,
            ..Default::default()
        },
        snap: true,
        ..Default::default()
    }
}
pub fn container_color(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::BLUE_BACKGROUND)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 0.0,
            ..Default::default()
        },
        snap: true,
        ..Default::default()
    }
}

pub fn container_medium_rounded(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Palette::TEXT_ON_DARK),
        background: Some(iced::Background::Color(Palette::DARK_BACKGROUND)),
        border: Border {
            color: Palette::DARK_BORDER,
            width: 1.0,
            radius: Radius {
                top_left: 10.0,
                top_right: 10.0,
                bottom_right: 10.0,
                bottom_left: 10.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn button_full(theme: &Theme, status: button::Status) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(Palette::DARK_BUTTON)),
        text_color: Palette::TEXT_ON_DARK,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn button_rounded_options_deselected(
    theme: &Theme,
    status: button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(Palette::DARK_BACKGROUND)),
        text_color: Palette::TEXT_ON_DARK,
        border: Border {
            color: Palette::DARK_BORDER,
            width: 1.0,
            radius: Radius {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn button_rounded_options_selected(
    theme: &Theme,
    status: button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(Palette::RAIL_ACTIVE)),
        text_color: Palette::DARKEST_BACKGROUND,
        border: Border {
            color: Palette::DARK_BORDER,
            width: 1.0,
            radius: Radius {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn button_rounded_top_full(
    theme: &Theme,
    status: button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(Palette::DARK_BUTTON)),
        text_color: Palette::TEXT_ON_DARK,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn button_rounded_bottom_full(
    theme: &Theme,
    status: button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(Palette::DARK_BUTTON)),
        text_color: Palette::TEXT_ON_DARK,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
        },
        snap: true,
        ..Default::default()
    }
}

pub fn slider_full(
    theme: &Theme,
    status: iced::widget::slider::Status,
) -> iced::widget::slider::Style {
    iced::widget::slider::Style {
        rail: Rail {
            backgrounds: (
                Background::Color(Palette::RAIL_ACTIVE),
                Background::Color(Palette::RAIL_DEACTIVE),
            ),
            width: 4.0,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::default(),
            },
        },
        handle: Handle {
            shape: iced::widget::slider::HandleShape::Rectangle {
                width: 2,
                border_radius: Radius::default(),
            },
            background: Background::Color(Palette::RAIL_ACTIVE),
            border_width: 2.0f32,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub fn text_input(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let background = Background::Color(match status {
        iced::widget::text_input::Status::Active => Palette::LIGHT_GRAY,
        iced::widget::text_input::Status::Disabled => Palette::DARK_BACKGROUND,
        _ => Palette::LIGHT_GRAY,
    });
    iced::widget::text_input::Style {
        background,
        border: Border {
            color: Palette::DARK_BORDER,
            width: 1.0,
            radius: Radius {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
        },
        icon: Palette::DARK_BACKGROUND,
        placeholder: Palette::DARK_BACKGROUND,
        value: Palette::DARKEST_BACKGROUND,
        selection: Palette::RAIL_ACTIVE,
    }
}
