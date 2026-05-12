use iced::{Alignment::Center, Length::{Fill, Shrink}, widget::{button, column, container, row, text, text_input}};

use crate::{Message, screens::{DefaultScreen, Viewable}, style};

#[derive(Debug)]
pub struct VolumeInfo {
    pub prop: f32,
    pub text_prop: String,
}

impl DefaultScreen for VolumeInfo {
    fn from_video(_video: &iced_video_player::Video) -> Self {
        Self { prop: 1.0f32, text_prop: "1.0".to_string() }
    }

    fn fit_to_bounds(&mut self, _video: &iced_video_player::Video, update_text: bool) {
        if self.prop < 0f32 {
            self.prop = 0f32;
            if update_text {
                self.text_prop = self.prop.to_string();
            }
        }
        else if self.prop > 4f32 {
            self.prop = 4f32;
            if update_text {
                self.text_prop = self.prop.to_string();
            }
        }
    }
}
impl Viewable for VolumeInfo {
    fn view<'a>(&self, counter: &'a crate::Counter) -> iced::Element<'a, crate::Message> {
        container(
            column![
                container(column![
                    row![
                        text("Multiply volume by: ").size(16).center(),
                        text_input("1", &self.text_prop)
                            .width(100)
                            .on_input(Message::VolumePropChanged)
                            .size(16)
                            .style(style::text_input)
                    ]
                    .align_y(Center)
                ])
                .padding(10),
                button(text("Do the volume changing!").size(16).center())
                    .on_press(Message::TryVolume)
                    .width(Fill)
                    .height(24)
                    .style(style::button_rounded_bottom_full)
            ]
            .height(Shrink)
            .width(Fill),
        )
        .style(style::container_medium_rounded)
        .width(Shrink)
        .height(Shrink)
        .into()
    }
}