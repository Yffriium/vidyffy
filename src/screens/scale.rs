use iced_video_player::Video;

use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    widget::{Space, button, column, container, pick_list, row, text, text_input},
};

use crate::{
    Message, VideoData,
    screens::{DefaultScreen, Viewable},
    style,
};

#[derive(Debug)]
pub struct ScaleInfo {
    pub width: u32,
    pub text_width: String,
    pub height: u32,
    pub text_height: String,
    pub prop: f32,
    pub text_prop: String,
    pub keep_aspect_ratio: bool,
}

impl DefaultScreen for ScaleInfo {
    fn from_video(video: &VideoData) -> Self {
        let vid_width = video.width;
        let vid_height = video.height;

        Self {
            width: vid_width as u32,
            text_width: vid_width.to_string(),
            height: vid_height as u32,
            text_height: vid_height.to_string(),
            prop: 1.0f32,
            text_prop: "1.0".to_string(),
            keep_aspect_ratio: true,
        }
    }

    fn fit_to_bounds(&mut self, video: &VideoData, update_text: bool) {
        if self.keep_aspect_ratio {
            if self.prop <= 0.0f32 {
                self.prop = 1.0f32;
                if update_text {
                    self.text_prop = "1.0".to_string();
                }
            }
            self.width = (video.width as f32 * self.prop).round() as u32;
            self.height = (video.height as f32 * self.prop).round() as u32;
            if update_text {
                self.text_width = self.width.to_string();
                self.text_height = self.height.to_string();
            }
        } else {
            if self.width <= 1 {
                self.width = 1;
                if update_text {
                    self.text_width = 1.to_string();
                }
            }
            if self.height <= 1 {
                self.height = 1;
                if update_text {
                    self.text_height = 1.to_string();
                }
            }
        }
    }
}

impl Viewable for ScaleInfo {
    fn view<'a>(&self, counter: &'a crate::Counter) -> iced::Element<'a, crate::Message> {
        container(
            column![
                container(column![
                    row![
                        text("Target scale: ").size(16).center(),
                        text_input("1", &self.text_width)
                            .width(100)
                            .on_input_maybe(if self.keep_aspect_ratio {
                                None
                            } else {
                                Some(Message::ScaleWidthChanged)
                            })
                            .size(16)
                            .style(style::text_input),
                        Space::new().width(20),
                        text_input("1", &self.text_height)
                            .width(100)
                            .on_input_maybe(if self.keep_aspect_ratio {
                                None
                            } else {
                                Some(Message::ScaleHeightChanged)
                            })
                            .size(16)
                            .style(style::text_input),
                    ]
                    .align_y(Center),
                    row![
                        button(text("Keep aspect ratio").size(13))
                            .on_press(Message::KeepAspectRatioToggled(!self.keep_aspect_ratio)),
                        text("Proportion: ").size(16).center(),
                        text_input("1", &self.text_prop)
                            .width(100)
                            .on_input_maybe(if self.keep_aspect_ratio {
                                Some(Message::ScaleProportionChanged)
                            } else {
                                None
                            })
                            .size(16)
                            .style(style::text_input),
                    ]
                    .align_y(Center),
                ])
                .padding(10),
                button(text("Do the scaling!").size(16).center())
                    .on_press(Message::TryScale)
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
