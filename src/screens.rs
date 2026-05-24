use iced::{
    Element,
    Length::{Fill, Shrink},
    widget::{Container, Space, container, row, text, text_input},
};
use iced_video_player::Video;

use crate::{Counter, Message, VideoData, style};

pub mod crop;
pub mod gif;
pub mod scale;
pub mod trim;
pub mod volume;

#[derive(Debug)]
pub enum Screen {
    Trim(trim::TrimInfo),
    Crop(crop::CropInfo),
    Scale(scale::ScaleInfo),
    Pipeline,
    Volume(volume::VolumeInfo),
    GifFrames(gif::GifInfo),
}

#[derive(Debug, Clone)]
pub enum ScreenName {
    Trim,
    Crop,
    Scale,
    Pipeline,
    Volume,
    GifFrames,
}

pub trait DefaultScreen: Sized {
    fn from_video(video: &VideoData) -> Self;
    fn fit_to_bounds(&mut self, video: &VideoData, update_text: bool);
}

impl ScreenName {
    pub fn to_screen(self, video: &VideoData) -> Screen {
        match self {
            ScreenName::Trim => Screen::Trim(trim::TrimInfo::from_video(video)),
            ScreenName::Crop => Screen::Crop(crop::CropInfo::from_video(video)),
            ScreenName::Scale => Screen::Scale(scale::ScaleInfo::from_video(video)),
            ScreenName::Pipeline => Screen::Pipeline,
            ScreenName::Volume => Screen::Volume(volume::VolumeInfo::from_video(video)),
            ScreenName::GifFrames => Screen::GifFrames(gif::GifInfo::from_video(video)),
        }
    }
}

pub trait Viewable: Sized {
    fn view<'a>(&self, counter: &'a Counter) -> Element<'a, Message>;
}

pub fn view_screen<'a>(counter: &'a Counter) -> Container<'a, Message> {
    let elt: Element<'a, Message> = match counter.screen.as_ref() {
        Some(Screen::Crop(crop_info)) => crop_info.view(counter),
        Some(Screen::Trim(trim_info)) => trim_info.view(counter),
        Some(Screen::Scale(scale_info)) => scale_info.view(counter),
        Some(Screen::Pipeline) => {
            if let Some(pipeline) = counter.pipeline.as_ref() {
                pipeline.view()
            } else {
                text("Can't show pipeline because something horrible is happening").into()
            }
        }
        Some(Screen::Volume(volume_info)) => volume_info.view(counter),
        Some(Screen::GifFrames(frames_info)) => frames_info.view(counter),
        None => text("Select an operation").into(),
    };
    container(elt)
        .height(Shrink)
        .width(Fill)
        .style(style::container_color)
        .padding(10)
}
