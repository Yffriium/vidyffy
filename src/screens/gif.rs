use crate::{
    Message, VideoData, VideoKind,
    screens::{DefaultScreen, Viewable},
    style,
};
use gif::Frame;
use iced::{
    Alignment::Center,
    Length::{Fill, Shrink},
    widget::{Image, button, column, container, grid, image::Handle, row, scrollable::{Direction, Scrollbar}, text, text_input},
};

use iced::widget::scrollable;

#[derive(Debug)]
pub struct GifInfo {}

impl DefaultScreen for GifInfo {
    fn from_video(video: &VideoData) -> Self {
        Self {}
    }

    fn fit_to_bounds(&mut self, video: &VideoData, update_text: bool) {
        // do nothing
    }
}

impl Viewable for GifInfo {
    fn view<'a>(&self, counter: &'a crate::Counter) -> iced::Element<'a, crate::Message> {
        if let Some(video_data) = &counter.video
            && let VideoKind::Gif { player_state, .. } = &video_data.kind
        {

            scrollable(row(player_state.frames_iter().enumerate().map(|(idx, elt)| {
                let centiseconds: u32 = elt.duration.as_millis() as u32 / 10u32;
                container(
                    column![
                        text(idx + 1),
                        Image::new(elt.handle.clone()).height(120).width(Shrink).content_fit(iced::ContentFit::Contain),
                        row![
                            button("-").on_press_maybe(if centiseconds > 1 {
                                Some(Message::GifFrameSetDuration { idx, new_duration_centiseconds: centiseconds.saturating_sub(1) })
                            } else {
                                None
                            }),
                            text(format!("{:.2}", elt.duration.as_secs_f32())),
                            button("+").on_press(Message::GifFrameSetDuration { idx, new_duration_centiseconds: centiseconds.saturating_add(1) }),
                        ],
                        row![
                            button("Hide").on_press(Message::GifFrameSetHidden { idx, hidden: !elt.hidden }),
                            button("Copy").on_press(Message::GifFrameCopy { idx }),
                        ]
                    ]
                ).padding(5).into()
                
            })))
            .direction(Direction::Horizontal(Scrollbar::default()))
            .into()
        } else {
            
            // can't do it, not a gif
            container(
                column![
                    container(column![
                        row![text("Select a gif").size(16),].align_y(Center),
                    ])
                    .padding(10),
                    button(text("Not a gif...").size(16).center())
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
}
