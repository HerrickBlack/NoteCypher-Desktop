use iced::widget::container;
use iced::{Background, Border, Color, Shadow, Theme};

// Color palette
const PRIMARY: Color = Color::from_rgb(
    59.0 / 255.0,
    130.0 / 255.0,
    246.0 / 255.0,
);

const PRIMARY_HOVER: Color = Color::from_rgb(
    37.0 / 255.0,
    99.0 / 255.0,
    235.0 / 255.0,
);

const SUCCESS: Color = Color::from_rgb(
    34.0 / 255.0,
    197.0 / 255.0,
    94.0 / 255.0,
);

const DANGER: Color = Color::from_rgb(
    239.0 / 255.0,
    68.0 / 255.0,
    68.0 / 255.0,
);

const EXPORT: Color = Color::from_rgb(
    99.0 / 255.0,
    102.0 / 255.0,
    241.0 / 255.0,
);

// Light theme colors
const LIGHT_BG: Color = Color::from_rgb(
    248.0 / 255.0,
    250.0 / 255.0,
    252.0 / 255.0,
);

const LIGHT_CARD: Color = Color::WHITE;

const LIGHT_TEXT: Color = Color::from_rgb(
    15.0 / 255.0,
    23.0 / 255.0,
    42.0 / 255.0,
);

const LIGHT_TEXT_MUTED: Color = Color::from_rgb(
    100.0 / 255.0,
    116.0 / 255.0,
    139.0 / 255.0,
);

const LIGHT_BORDER: Color = Color::from_rgb(
    226.0 / 255.0,
    232.0 / 255.0,
    240.0 / 255.0,
);

// Dark theme colors
const DARK_BG: Color = Color::from_rgb(
    15.0 / 255.0,
    23.0 / 255.0,
    42.0 / 255.0,
);

const DARK_CARD: Color = Color::from_rgb(
    30.0 / 255.0,
    41.0 / 255.0,
    59.0 / 255.0,
);

const DARK_TEXT: Color = Color::from_rgb(
    248.0 / 255.0,
    250.0 / 255.0,
    252.0 / 255.0,
);

const DARK_TEXT_MUTED: Color = Color::from_rgb(
    148.0 / 255.0,
    163.0 / 255.0,
    184.0 / 255.0,
);

const DARK_BORDER: Color = Color::from_rgb(
    51.0 / 255.0,
    65.0 / 255.0,
    85.0 / 255.0,
);

// ============================================================================
// Container Styles
// ============================================================================

#[derive(Default)]
pub enum Container {
    #[default]
    Default,
    Card,
    Header,
    Success,
    Selected,
    Thumbnail,
    Export,
}

impl container::StyleSheet for Container {
    type Style = Theme;

    fn appearance(&self, theme: &Theme) -> container::Appearance {
        let colors = theme.extended_palette();
        
        match self {
            Container::Default => container::Appearance::default(),
            
            Container::Card => {
                let is_dark = matches!(theme, Theme::Dark);
                container::Appearance {
                    background: Some(Background::Color(if is_dark {
                        DARK_CARD
                    } else {
                        LIGHT_CARD
                    })),
                    border: Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, if is_dark { 0.3 } else { 0.1 }),
                        blur_radius: 4.0,
                    },
                    ..Default::default()
                }
            }
            
            Container::Header => {
                let is_dark = matches!(theme, Theme::Dark);
                container::Appearance {
                    background: Some(Background::Color(if is_dark {
                        DARK_CARD
                    } else {
                        LIGHT_CARD
                    })),
                    border: Border {
                        radius: 0.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            
            Container::Success => {
                container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(
                        34.0 / 255.0,
                        197.0 / 255.0,
                        94.0 / 255.0,
                        0.1,
                    ))),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            
            Container::Selected => {
                container::Appearance {
                    background: Some(Background::Color(Color::from_rgba(
                        59.0 / 255.0,
                        130.0 / 255.0,
                        246.0 / 255.0,
                        0.2,
                    ))),
                    border: Border {
                        radius: 8.0.into(),
                        color: PRIMARY,
                        width: 2.0,
                    },
                    ..Default::default()
                }
            }
            
            Container::Thumbnail => {
                let is_dark = matches!(theme, Theme::Dark);
                container::Appearance {
                    background: Some(Background::Color(if is_dark {
                        Color::from_rgb(0.2, 0.2, 0.2)
                    } else {
                        Color::WHITE
                    })),
                    border: Border {
                        radius: 8.0.into(),
                        color: if is_dark {
                            Color::from_rgb(0.3, 0.3, 0.3)
                        } else {
                            Color::from_rgb(0.8, 0.8, 0.8)
                        },
                        width: 2.0,
                    },
                    ..Default::default()
                }
            }
            
            Container::Export => {
                container::Appearance {
                    background: Some(Background::Color(PRIMARY)),
                    border: Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        }
    }
}

// ============================================================================
// Button Styles
// ============================================================================

#[derive(Default)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Danger,
    Success,
    Export,
    Link,
    Icon,
    Upload,
    Thumbnail,
    Disabled,
}

impl iced::widget::button::StyleSheet for Button {
    type Style = Theme;

    fn active(&self, theme: &Theme) -> iced::widget::button::Appearance {
        let is_dark = matches!(theme, Theme::Dark);
        
        match self {
            Button::Primary => iced::widget::button::Appearance {
                background: Some(Background::Color(PRIMARY)),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            
            Button::Secondary => iced::widget::button::Appearance {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgb(0.2, 0.2, 0.2)
                } else {
                    Color::from_rgb(0.95, 0.95, 0.95)
                })),
                border: Border {
                    radius: 8.0.into(),
                    color: if is_dark {
                        Color::from_rgb(0.3, 0.3, 0.3)
                    } else {
                        Color::from_rgb(0.8, 0.8, 0.8)
                    },
                    width: 1.0,
                },
                text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                ..Default::default()
            },
            
            Button::Danger => iced::widget::button::Appearance {
                background: Some(Background::Color(Color::from_rgba(
                    239.0 / 255.0,
                    68.0 / 255.0,
                    68.0 / 255.0,
                    0.1,
                ))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                text_color: DANGER,
                ..Default::default()
            },
            
            Button::Success => iced::widget::button::Appearance {
                background: Some(Background::Color(SUCCESS)),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            
            Button::Export => iced::widget::button::Appearance {
                background: Some(Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                text_color: PRIMARY,
                ..Default::default()
            },
            
            Button::Link => iced::widget::button::Appearance {
                background: None,
                border: Border::default(),
                text_color: PRIMARY,
                ..Default::default()
            },
            
            Button::Icon => iced::widget::button::Appearance {
                background: None,
                border: Border::default(),
                text_color: if is_dark { DARK_TEXT_MUTED } else { LIGHT_TEXT_MUTED },
                ..Default::default()
            },
            
            Button::Upload => iced::widget::button::Appearance {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.1)
                } else {
                    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.05)
                })),
                border: Border {
                    radius: 16.0.into(),
                    color: if is_dark {
                        Color::from_rgb(0.3, 0.3, 0.3)
                    } else {
                        Color::from_rgb(0.8, 0.8, 0.8)
                    },
                    width: 2.0,
                },
                text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                ..Default::default()
            },
            
            Button::Thumbnail => iced::widget::button::Appearance {
                background: None,
                border: Border::default(),
                text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                ..Default::default()
            },
            
            Button::Disabled => iced::widget::button::Appearance {
                background: Some(Background::Color(Color::from_rgba(
                    0.5, 0.5, 0.5, 0.3
                ))),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.5),
                ..Default::default()
            },
        }
    }

    fn hovered(&self, theme: &Theme) -> iced::widget::button::Appearance {
        let is_dark = matches!(theme, Theme::Dark);
        
        match self {
            Button::Primary => iced::widget::button::Appearance {
                background: Some(Background::Color(PRIMARY_HOVER)),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            
            Button::Secondary => iced::widget::button::Appearance {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgb(0.25, 0.25, 0.25)
                } else {
                    Color::from_rgb(0.9, 0.9, 0.9)
                })),
                border: Border {
                    radius: 8.0.into(),
                    color: PRIMARY,
                    width: 1.0,
                },
                text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                ..Default::default()
            },
            
            Button::Danger => iced::widget::button::Appearance {
                background: Some(Background::Color(Color::from_rgba(
                    239.0 / 255.0,
                    68.0 / 255.0,
                    68.0 / 255.0,
                    0.2,
                ))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                text_color: DANGER,
                ..Default::default()
            },
            
            Button::Export => iced::widget::button::Appearance {
                background: Some(Background::Color(Color::from_rgb(
                    243.0 / 255.0,
                    244.0 / 255.0,
                    246.0 / 255.0,
                ))),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                text_color: PRIMARY,
                ..Default::default()
            },
            
            Button::Link => iced::widget::button::Appearance {
                background: None,
                border: Border::default(),
                text_color: PRIMARY_HOVER,
                ..Default::default()
            },
            
            Button::Icon => iced::widget::button::Appearance {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.1)
                } else {
                    Color::from_rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.05)
                })),
                border: Border::default(),
                text_color: DANGER,
                ..Default::default()
            },
            
            Button::Upload => iced::widget::button::Appearance {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.15)
                } else {
                    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.1)
                })),
                border: Border {
                    radius: 16.0.into(),
                    color: PRIMARY,
                    width: 2.0,
                },
                text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                ..Default::default()
            },
            
            _ => self.active(theme),
        }
    }
}

// ============================================================================
// Text Styles
// ============================================================================

#[derive(Default)]
pub enum Text {
    #[default]
    Default,
    Bold,
    Muted,
}

impl iced::widget::text::StyleSheet for Text {
    type Style = Theme;

    fn appearance(&self, theme: &Theme) -> iced::widget::text::Appearance {
        let is_dark = matches!(theme, Theme::Dark);
        
        match self {
            Text::Default => iced::widget::text::Appearance {
                color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
            },
            Text::Bold => iced::widget::text::Appearance {
                color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
            },
            Text::Muted => iced::widget::text::Appearance {
                color: Some(if is_dark { DARK_TEXT_MUTED } else { LIGHT_TEXT_MUTED }),
            },
        }
    }
}

// ============================================================================
// Checkbox Styles
// ============================================================================

#[derive(Default)]
pub enum Checkbox {
    #[default]
    Default,
}

impl iced::widget::checkbox::StyleSheet for Checkbox {
    type Style = Theme;

    fn active(&self, theme: &Theme, is_checked: bool) -> iced::widget::checkbox::Appearance {
        match is_checked {
            true => iced::widget::checkbox::Appearance {
                background: Background::Color(PRIMARY),
                icon_color: Color::WHITE,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: None,
            },
            false => iced::widget::checkbox::Appearance {
                background: Background::Color(Color::TRANSPARENT),
                icon_color: Color::TRANSPARENT,
                border: Border {
                    radius: 4.0.into(),
                    color: if matches!(theme, Theme::Dark) {
                        Color::from_rgb(0.4, 0.4, 0.4)
                    } else {
                        Color::from_rgb(0.7, 0.7, 0.7)
                    },
                    width: 2.0,
                },
                text_color: None,
            },
        }
    }

    fn hovered(&self, theme: &Theme, is_checked: bool) -> iced::widget::checkbox::Appearance {
        self.active(theme, is_checked)
    }
}

// ============================================================================
// Scrollbar Styles
// ============================================================================

#[derive(Default)]
pub enum Scrollbar {
    #[default]
    Default,
}

impl iced::widget::scrollable::StyleSheet for Scrollbar {
    type Style = Theme;

    fn active(&self, theme: &Theme) -> iced::widget::scrollable::Appearance {
        let is_dark = matches!(theme, Theme::Dark);
        
        iced::widget::scrollable::Appearance {
            container: iced::widget::scrollable::Appearance::default().container,
            vertical_rail: iced::widget::scrollable::Rail {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.05))),
                border: Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.2)
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                },
            },
            horizontal_rail: iced::widget::scrollable::Rail {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.05))),
                border: Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.2)
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, theme: &Theme, is_mouse_over_x: bool, is_mouse_over_y: bool) -> iced::widget::scrollable::Appearance {
        self.active(theme)
    }

    fn dragging(&self, theme: &Theme) -> iced::widget::scrollable::Appearance {
        self.active(theme)
    }
}
