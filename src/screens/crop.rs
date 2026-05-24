use std::fmt::Display;

use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    widget::{Space, button, column, container, pick_list, row, text, text_input},
};
use iced_video_player::Video;

use crate::{
    Message, VideoData,
    screens::{DefaultScreen, Viewable},
    style,
};

// TODO make these fields private
#[derive(Default, Debug)]
pub struct CropInfo {
    pub crop_mode: CropMode,
    pub width: u16,
    pub height: u16,
    pub text_width: String,
    pub text_height: String,
}

impl CropInfo {
    pub fn get_top_left_pixel(&self, video: &VideoData) -> (u16, u16) {
        match self.crop_mode {
            CropMode::TopLeft { x, y, .. } => (x, y),
            CropMode::PixelCenter { x, y, .. } => (
                x.saturating_sub(self.width / 2),
                y.saturating_sub(self.height / 2),
            ),
            CropMode::VideoCenter => {
                let center_loc = (video.width / 2, video.height / 2);
                (
                    center_loc.0.saturating_sub(self.width / 2),
                    center_loc.1.saturating_sub(self.height / 2),
                )
            }
        }
    }
    /// From whatever crop_type we have, changes to CropType::TopLeft
    /// Also returns top left pixel
    pub fn convert_top_left(&mut self, video: &VideoData) -> (u16, u16) {
        let top_left = self.get_top_left_pixel(video);
        self.crop_mode = CropMode::TopLeft {
            x: top_left.0,
            y: top_left.1,
            text_x: top_left.0.to_string(),
            text_y: top_left.1.to_string(),
        };
        top_left
    }

    pub fn convert_to(&mut self, video: &VideoData, target: CropModeKind) {
        // first, go to top left
        let top_left_pixel = self.convert_top_left(video);
        // then, convert from top left to target
        match target {
            CropModeKind::TopLeft => { /* done */ }
            CropModeKind::VideoCenter => self.crop_mode = CropMode::VideoCenter,
            CropModeKind::PixelCenter => {
                let new_x = top_left_pixel.0.saturating_add(self.width / 2);
                let new_y = top_left_pixel.1.saturating_add(self.height / 2);
                self.crop_mode = CropMode::PixelCenter {
                    x: new_x,
                    y: new_y,
                    text_x: new_x.to_string(),
                    text_y: new_y.to_string(),
                };
            }
        }
    }
}

#[derive(Debug)]
pub enum CropMode {
    TopLeft {
        x: u16,
        y: u16,
        text_x: String,
        text_y: String,
    },
    PixelCenter {
        x: u16,
        y: u16,
        text_x: String,
        text_y: String,
    },
    VideoCenter,
}
impl Default for CropMode {
    fn default() -> Self {
        Self::TopLeft {
            x: 0,
            y: 0,
            text_x: "0".to_string(),
            text_y: "0".to_string(),
        }
    }
}
impl CropMode {
    pub fn to_kind(&self) -> CropModeKind {
        match self {
            CropMode::TopLeft { .. } => CropModeKind::TopLeft,
            CropMode::PixelCenter { .. } => CropModeKind::PixelCenter,
            CropMode::VideoCenter => CropModeKind::VideoCenter,
        }
    }
}
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum CropModeKind {
    #[default]
    TopLeft,
    VideoCenter,
    PixelCenter,
}

impl Display for CropModeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CropModeKind::TopLeft => write!(f, "Top left pixel"),
            CropModeKind::VideoCenter => write!(f, "Center of video"),
            CropModeKind::PixelCenter => write!(f, "Center at pixel"),
        }
    }
}

impl DefaultScreen for CropInfo {
    fn from_video(video: &VideoData) -> Self {
        Self {
            crop_mode: CropMode::default(),

            width: video.width,
            height: video.height,
            text_width: video.width.to_string(),
            text_height: video.height.to_string(),
        }
    }

    fn fit_to_bounds(&mut self, video: &VideoData, update_text: bool) {
        let vid_width = video.width;
        let vid_height = video.height;
        match &mut self.crop_mode {
            CropMode::VideoCenter => {
                if self.width > vid_width {
                    self.width = vid_width;
                    if update_text {
                        self.text_width = self.width.to_string();
                    }
                }
                if self.height > vid_height {
                    self.height = vid_height;
                    if update_text {
                        self.text_height = self.height.to_string();
                    }
                }
            }
            CropMode::TopLeft {
                x,
                y,
                text_x,
                text_y,
            } => {
                if *x >= vid_width {
                    *x = vid_width - 1;
                    if update_text {
                        *text_x = x.to_string();
                    }
                }

                if *y >= vid_height {
                    *y = vid_height - 1;
                    if update_text {
                        *text_y = y.to_string();
                    }
                }

                let max_width_value = vid_width - *x;
                let max_height_value = vid_height - *y;

                if self.width == 0 {
                    self.width = 1;
                    if update_text {
                        self.text_width = self.width.to_string();
                    }
                }
                if self.width > max_width_value {
                    self.width = max_width_value;
                    if update_text {
                        self.text_width = self.width.to_string();
                    }
                }

                if self.height == 0 {
                    self.height = 1;
                    if update_text {
                        self.text_height = self.height.to_string();
                    }
                }
                if self.height > max_height_value {
                    self.height = max_height_value;
                    if update_text {
                        self.text_height = self.height.to_string();
                    }
                }
            }
            CropMode::PixelCenter {
                x,
                y,
                text_x,
                text_y,
            } => {
                if *x >= vid_width {
                    *x = vid_width - 1;
                    if update_text {
                        *text_x = x.to_string();
                    }
                } else if *x == 0 {
                    *x = 1;
                    if update_text {
                        *text_x = x.to_string();
                    }
                }

                // TODO there is a problem here. Imagine a 1x1 video.
                // It will be cropped to nothing. Need to disallow this.

                if *y >= vid_height {
                    *y = vid_height - 1;
                    if update_text {
                        *text_y = y.to_string();
                    }
                } else if *y == 0 {
                    *y = 1;
                    if update_text {
                        *text_y = y.to_string();
                    }
                }

                let max_width_value = u16::min(*x, vid_width.saturating_sub(*x));
                let max_height_value = u16::min(*y, vid_height.saturating_sub(*y));

                if self.width == 0 {
                    self.width = 1;
                    if update_text {
                        self.text_width = self.width.to_string();
                    }
                }
                if self.width > max_width_value {
                    self.width = max_width_value;
                    if update_text {
                        self.text_width = self.width.to_string();
                    }
                }

                if self.height == 0 {
                    self.height = 1;
                    if update_text {
                        self.text_height = self.height.to_string();
                    }
                }
                if self.height > max_height_value {
                    self.height = max_height_value;
                    if update_text {
                        self.text_height = self.height.to_string();
                    }
                }
            }
        }
    }
}

impl Viewable for CropInfo {
    fn view<'a>(&self, counter: &'a crate::Counter) -> Element<'a, Message> {
        let position_row: Element<Message> = match &self.crop_mode {
            CropMode::TopLeft { text_x, text_y, .. } => row![
                text("Crop top left pixel: ").size(16),
                text_input("0", &text_x)
                    .width(100)
                    .on_input(Message::CropLeftChanged)
                    .size(16)
                    .style(style::text_input),
                Space::new().width(20),
                text_input("0", &text_y)
                    .width(100)
                    .on_input(Message::CropTopChanged)
                    .size(16)
                    .style(style::text_input),
            ]
            .align_y(Center)
            .into(),

            CropMode::PixelCenter { text_x, text_y, .. } => row![
                text("Crop center pixel: ").size(16),
                text_input("0", &text_x)
                    .width(100)
                    .on_input(Message::CropLeftChanged)
                    .size(16)
                    .style(style::text_input),
                Space::new().width(20),
                text_input("0", &text_y)
                    .width(100)
                    .on_input(Message::CropTopChanged)
                    .size(16)
                    .style(style::text_input),
            ]
            .align_y(Center)
            .into(),

            CropMode::VideoCenter => Space::new().into(),
        };

        container(
            column![
                container(column![
                    pick_list(
                        [
                            CropModeKind::TopLeft,
                            CropModeKind::PixelCenter,
                            CropModeKind::VideoCenter
                        ],
                        Some(self.crop_mode.to_kind()),
                        Message::CropTypeSelected
                    ),
                    position_row,
                    row![
                        text("Crop width and height: ").size(16),
                        text_input("1", &self.text_width)
                            .width(100)
                            .on_input(Message::CropWidthChanged)
                            .size(16)
                            .style(style::text_input),
                        Space::new().width(20),
                        text_input("1", &self.text_height)
                            .width(100)
                            .on_input(Message::CropHeightChanged)
                            .size(16)
                            .style(style::text_input),
                    ]
                    .align_y(Center),
                ])
                .padding(10),
                button(text("Do the cropping!").size(16).center())
                    .on_press(Message::TryCrop)
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
