use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    widget::{Space, button, column, container, pick_list, row, text, text_input},
};
use iced_video_player::Video;

use crate::{
    Message,
    screens::{DefaultScreen, Viewable},
    style,
};

#[derive(Default, Debug)]
pub struct TrimInfo {
    pub start_frame_text: String,
    pub start_frame_time: f64,
    pub end_frame_text: String,
    pub end_frame_time: f64,
}

impl DefaultScreen for TrimInfo {
    fn from_video(video: &Video) -> Self {
        let vid_duration = video.duration();
        let last_frame_time = vid_duration.as_secs_f64();

        Self {
            start_frame_text: "0".to_string(),
            start_frame_time: 0f64,
            end_frame_text: last_frame_time.to_string(),
            end_frame_time: last_frame_time,
        }
    }

    fn fit_to_bounds(&mut self, video: &Video, update_text: bool) {
        if self.start_frame_time <= 0.0f64 {
            self.start_frame_time = 0.0f64;
            if update_text {
                self.start_frame_text = self.start_frame_time.to_string();
            }
        }
        let max_dur = video.duration().as_secs_f64();
        if self.end_frame_time > max_dur {
            self.end_frame_time = max_dur;
            if update_text {
                self.end_frame_text = self.end_frame_time.to_string();
            }
        }
    }
}

impl Viewable for TrimInfo {
    fn view<'a>(&self, counter: &'a crate::Counter) -> iced::Element<'a, crate::Message> {
        container(
            column![
                container(column![
                    row![
                        text("Trim start frame:"),
                        text_input("", &self.start_frame_text)
                            .width(100)
                            .on_input(Message::TrimStartTimeChanged)
                            .style(style::text_input),
                        button("Set to now")
                            .on_press(Message::SetTrimStartToNow)
                            .style(style::button_full)
                    ]
                    .align_y(Center),
                    row![
                        text("Trim end frame:"),
                        text_input("", &self.end_frame_text)
                            .width(100)
                            .on_input(Message::TrimEndTimeChanged)
                            .style(style::text_input),
                        button("Set to now")
                            .on_press(Message::SetTrimEndToNow)
                            .style(style::button_full)
                    ]
                    .align_y(Center),
                ])
                .padding(10),
                button(text("Do the trimming!").size(16).center())
                    .on_press(Message::TryTrim)
                    .width(Fill)
                    .height(24)
                    .style(style::button_rounded_bottom_full),
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
