mod ffmpeg;
mod pipeline;
mod screens;
mod style;
mod gifplayer;

use iced::widget::image::Handle;
use screens::ScreenName;

use iced::Length::{Fill, Shrink};
use iced::widget::slider;
use iced::{
    Alignment, Color, Element, Point, Rectangle, Renderer, Size, Task,
    widget::{
        Container, Space, button,
        canvas::{self, Canvas, Frame, Path},
        column, container, pick_list, row, stack, text, text_input,
    },
};
use iced::{Alignment::Center, ContentFit};
use iced_video_player::{Video, VideoPlayer};
use rfd::FileDialog;
use screens::DefaultScreen;
use std::time::Duration;
use std::{fmt::Display, path::PathBuf};
use url::Url;

use crate::gifplayer::{GifFrame, GifPlayer};
use crate::pipeline::Pipeline;
use crate::screens::Screen;
use crate::screens::crop::CropMode;
use crate::screens::crop::CropModeKind;

enum BottomBarState {
    Normal,
    InProgress,
    Error,
}

struct BottomBar {
    message: String,
    state: BottomBarState,
}

impl BottomBar {
    pub fn set_error(&mut self, msg: String) {
        self.message = msg;
        self.state = BottomBarState::Error;
    }
    pub fn set_normal(&mut self) {
        self.message = "".to_string();
        self.state = BottomBarState::Normal;
    }
    pub fn set_info(&mut self, msg: String) {
        self.message = msg;
        self.state = BottomBarState::Normal;
    }
    pub fn set_in_progress(&mut self, msg: String) {
        self.message = msg;
        self.state = BottomBarState::InProgress;
    }
}

struct NormRect {
    x_left: f32,
    y_top: f32,
    width: f32,
    height: f32,
}
impl NormRect {
    pub fn to_rect(&self, bounds: Rectangle) -> Rectangle {
        let w = bounds.width;
        let h = bounds.height;

        Rectangle {
            x: bounds.x + self.x_left * w,
            y: bounds.y + self.y_top * h,
            width: self.width * w,
            height: self.height * h,
        }
    }
}
struct RectOverlay {
    rect: NormRect,
    video_aspect_ratio: f32,
}
impl<Message> canvas::Program<Message> for RectOverlay {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let video_rect = contain_rect(bounds.size(), self.video_aspect_ratio);
        let scaled_rect = self.rect.to_rect(video_rect);

        let path_rect = Path::rectangle(
            Point::new(scaled_rect.x, scaled_rect.y),
            Size::new(scaled_rect.width, scaled_rect.height),
        );

        frame.stroke(
            &path_rect,
            canvas::Stroke {
                width: 2.0,
                style: canvas::Style::Solid(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 0.0,
                    a: 0.8,
                }),
                ..Default::default()
            },
        );

        vec![frame.into_geometry()]
    }
}

pub struct VideoData {
    path: PathBuf,
    duration: f64,
    current_time: f64,
    frames: u32,
    width: u16,
    height: u16,
    kind: VideoKind,
}

pub enum VideoKind {
    Video { player: Video },
    Gif { player_state: gifplayer::GifState },
}

impl VideoData {
    pub fn new(path_buf: PathBuf) -> Result<Self, String> {
        match path_buf.extension().map(|elt| elt.to_str()) {
            Some(Some("gif")) => {
                // make gif player
                let mut opts = gif::DecodeOptions::new();
                opts.set_color_output(gif::ColorOutput::RGBA);

                let file = match std::fs::File::open(path_buf.clone()) {
                    Ok(f) => f,
                    Err(e) => return Err(format!("Could not open file. {}", e)),
                };

                let decoder = match opts.read_info(file) {
                    Ok(f) => f,
                    Err(e) => return Err(format!("Could not decode gif info. {}", e)),
                };

                let mut width = 0;
                let mut height = 0;

                let frames: Vec<gifplayer::GifFrame> = decoder
                    .into_iter()
                    .filter_map(|elt| {
                        if elt.is_ok() {
                            Some(elt.unwrap())
                        } else {
                            None
                        }
                    })
                    // .map(|elt| elt.to_owned())
                    .map(|elt| {
                        width = elt.width;
                        height = elt.height;
                            gifplayer::GifFrame {
                                handle: Handle::from_rgba(elt.width as u32, elt.height as u32, elt.buffer.to_vec()),
                                duration: Duration::from_millis(10 * elt.delay as u64),
                                hidden: false,
                            }})
                    .collect();

                let num_frames = frames.len() as u32;

                let gif_state = crate::gifplayer::GifState::new(frames);
                
                let duration = gif_state.duration().as_secs_f64(); // TODO duration!!
                
                Ok(Self {
                    path: path_buf,
                    kind: VideoKind::Gif { player_state: gif_state},
                    duration,
                    width,
                    height,
                    frames: num_frames,
                    current_time: 0f64,
                })
            }
            _ => {
                // make video player
                let url = match Url::from_file_path(&path_buf) {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(format!(
                            "Could not load URL from path_buf {}",
                            path_buf.to_str().unwrap()
                        ));
                    }
                };
                let player = match Video::new(&url) {
                    Ok(r) => r,
                    Err(e) => {
                        println!(
                            "Exists? {}",
                            url.to_file_path()
                                .expect("URL file path couldnt be made?!")
                                .exists()
                        );
                        println!("URL: {:?}", url);
                        panic!("Could not make video player from URL {:?}", e);
                    }
                };
                let duration = player.duration().as_secs_f64();
                let frames = f64::floor(player.framerate() * duration) as u32;
                let size = player.size();
                let width = size.0 as u16;
                let height = size.1 as u16;
                Ok(Self {
                    path: path_buf,
                    kind: VideoKind::Video { player },
                    duration,
                    frames,
                    width,
                    height,
                    current_time: 0f64,
                })
            }
        }
    }
}
struct Counter {
    video: Option<VideoData>,
    screen: Option<Screen>,
    pipeline: Option<Pipeline>,
    bottom_bar: BottomBar,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            video: Default::default(),
            screen: Default::default(),
            pipeline: None,
            bottom_bar: BottomBar {
                message: String::new(),
                state: BottomBarState::Normal,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// When the user clicks the "Open" button to start a new pipeline on a file.
    OpenFile,
    /// Callback for when the user selects a file from the system explorer.
    FileSelected(PathBuf),
    /// Callback for when the user deselects a file?
    /// Seems to not be used currently.
    FileDeselected,

    /// Called when VideoPlayer ends (should loop video)
    VideoPlayerEnds,
    /// Called when the VideoPlayer advances to a new frame
    VideoPlayerNewFrame,
    /// Sets whether the current video is playing or not
    SetPlaying(bool),
    /// Sets whether the current video is muted or not
    SetMute(bool),
    /// Sets the volume of the current player
    VideoPlayerSetVolume(f64),
    /// Goes to default screen by screen name, or goes to no screen,
    /// closing the panel.
    GoToScreen(Option<ScreenName>),
    /// Callback whenever the slider on the player is changed by the user.
    PlayerSliderChanged(f64),
    /// Sets the trim start to the current time for the video.
    SetTrimStartToNow,
    /// Sets the trim end to the current time for the video.
    SetTrimEndToNow,
    /// Callback whenever user enters their own trim start time.
    TrimStartTimeChanged(String),
    /// Callback whenever user enters their own trim end time.
    TrimEndTimeChanged(String),
    /// Steps forward or backward by the given number of frames on the current
    /// player. Gives best-effort seek with video player when going backwards.
    StepFrames(i32),
    /// Steps forward or backward by the given number of centiseconds on the
    /// current player.
    StepCentiseconds(i32),
    /// Tries to conduct a trim.
    TryTrim,
    /// Callback when trim is finished.
    TrimFinished(Result<PathBuf, String>),
    /// Tries to save.
    TrySave,
    /// Callback when user selects a save path.
    SaveTo(PathBuf),
    CropLeftChanged(String),
    CropTopChanged(String),
    CropWidthChanged(String),
    CropHeightChanged(String),
    TryCrop,
    CropFinished(Result<PathBuf, String>),
    ScaleWidthChanged(String),
    ScaleHeightChanged(String),
    KeepAspectRatioToggled(bool),
    ScaleProportionChanged(String),
    TryScale,
    ScaleFinished(Result<PathBuf, String>),
    CropTypeSelected(CropModeKind),
    /// Reverts pipeline to given index
    RevertToPipeline(usize),
    VolumePropChanged(String),
    TryVolume,
    VolumeFinished(Result<PathBuf, String>),

    /// Converts frames in gif video to a gif
    FramesToGif,
    OnNewGifFrame(u32, Duration),
    GifFrameSetDuration{idx: usize, new_duration_centiseconds: u32},
    GifFrameSetHidden{idx: usize, hidden: bool},
    GifFrameCopy{idx: usize},
    GifFrameSwap{source_idx: usize, dest_idx: usize},
}

fn pick_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Video", &["mp4", "mkv", "avi", "mov", "gif"])
        .pick_file()
}

fn contain_rect(outer: iced::Size, video_aspect: f32) -> Rectangle {
    let outer_aspect = outer.width / outer.height;
    if outer_aspect > video_aspect {
        let h = outer.height;
        let w = h * video_aspect;
        let x = (outer.width - w) / 2.0;

        Rectangle {
            x,
            y: 0.0,
            width: w,
            height: h,
        }
    } else {
        let w = outer.width;
        let h = w / video_aspect;
        let y = (outer.height - h) / 2.0;

        Rectangle {
            x: 0.0,
            y,
            width: w,
            height: h,
        }
    }
}

impl Counter {
    fn view_video_player<'a>(&'a self, video: &'a VideoData) -> Container<'a, Message> {
        match &video.kind {
            VideoKind::Video { player } => {
                let play_pause_button = if player.paused() {
                    button(text("Play").center().size(15)).on_press(Message::SetPlaying(true))
                } else {
                    button(text("Pause").center().size(15)).on_press(Message::SetPlaying(false))
                }
                .style(style::button_full);

                let mute_unmute_button = if player.muted() {
                    button(text("Unmute").center().size(13)).on_press(Message::SetMute(false))
                } else {
                    button(text("Mute").center().size(13)).on_press(Message::SetMute(true))
                }
                .width(Shrink)
                .height(Fill)
                .style(style::button_full);

                let player_bar = container(
                    row![
                        play_pause_button,
                        slider(
                            0f64..=video.duration,
                            video.current_time,
                            Message::PlayerSliderChanged
                        )
                        .step(1f64 / player.framerate())
                        .width(Fill)
                        .style(style::slider_full),
                        text(format!("{:.1} / {:.1}", video.current_time, video.duration))
                            .center()
                            .size(15),
                        button(text("<").size(15).center())
                            .on_press(Message::StepFrames(-1))
                            .width(Shrink)
                            .height(Fill)
                            .style(style::button_full),
                        button(text(">").size(15).center())
                            .on_press(Message::StepFrames(1))
                            .width(Shrink)
                            .height(Fill)
                            .style(style::button_full),
                    ]
                    .spacing(5)
                    .height(25)
                    .width(Fill),
                )
                .width(Fill)
                .height(Shrink)
                .style(style::container_medium);

                let video_size = player.size();
                let video_aspect = video_size.0 as f32 / video_size.1 as f32;

                let stats_bar = container(
                    row![
                        text(format!("{}x{}", video_size.0, video_size.1))
                            .size(13)
                            .center()
                            .width(Shrink)
                            .height(Fill),
                        mute_unmute_button,
                    ]
                    .height(20)
                    .spacing(3)
                    .width(Fill),
                )
                .style(style::container_medium)
                .width(Fill)
                .height(Shrink);

                let mut stack_children: Vec<Element<Message>> = vec![
                    VideoPlayer::new(player)
                        .width(Fill)
                        .height(Fill)
                        .content_fit(ContentFit::Contain)
                        .on_end_of_stream(Message::VideoPlayerEnds)
                        .on_new_frame(Message::VideoPlayerNewFrame)
                        .into(),
                ];

                // TODO extract this out of here,
                // move to screens/crop.rs
                if let Some(Screen::Crop(crop_info)) = self.screen.as_ref() {
                    let top_left_pixel = crop_info.get_top_left_pixel(&video);
                    let x_left_prop =
                        (top_left_pixel.0 as f32 / video_size.0 as f32).clamp(0.0f32, 1.0f32);
                    let y_top_prop =
                        (top_left_pixel.1 as f32 / video_size.1 as f32).clamp(0.0f32, 1.0f32);

                    let mut width_prop =
                        (crop_info.width as f32 / video_size.0 as f32).clamp(0.0f32, 1.0f32);
                    let mut height_prop =
                        (crop_info.height as f32 / video_size.1 as f32).clamp(0.0f32, 1.0f32);

                    width_prop = f32::min(width_prop, 1.0f32 - x_left_prop);
                    height_prop = f32::min(height_prop, 1.0f32 - y_top_prop);

                    let overlay = Canvas::new(RectOverlay {
                        rect: NormRect {
                            x_left: x_left_prop,
                            y_top: y_top_prop,
                            width: width_prop,
                            height: height_prop,
                        },
                        video_aspect_ratio: video_aspect,
                    });
                    stack_children.push(overlay.width(Fill).height(Fill).into());
                }

                let video_stack = stack(stack_children);

                let video_container = container(video_stack)
                    .width(Fill)
                    .height(Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center);

                container(column![
                    video_container,
                    stats_bar,
                    player_bar.width(Fill).height(Shrink),
                ])
                .style(style::container_dark)
            }
            VideoKind::Gif { player_state, .. } => {
                let play_pause_button = if !player_state.playing() {
                    button(text("Play").center().size(15)).on_press(Message::SetPlaying(true))
                } else {
                    button(text("Pause").center().size(15)).on_press(Message::SetPlaying(false))
                }
                .height(Fill).width(Shrink).style(style::button_full);

                let player_bar = container(
                    row![
                        play_pause_button,
                        slider(
                            0f64..=video.duration,
                            video.current_time,
                            Message::PlayerSliderChanged
                        )
                        .step(0.01f64) // gifs step by centiseconds
                        .width(Fill)
                        .style(style::slider_full),
                        text(format!("{:.1} / {:.1}", video.current_time, video.duration))
                            .center()
                            .size(15),
                        button(text("<").size(15).center())
                            .on_press(Message::StepCentiseconds(-1)) // gifs step by centiseconds
                            .width(Shrink)
                            .height(Fill)
                            .style(style::button_full),
                        button(text(">").size(15).center())
                            .on_press(Message::StepCentiseconds(1)) // gifs step by centiseconds
                            .width(Shrink)
                            .height(Fill)
                            .style(style::button_full),
                    ]
                    .spacing(5)
                    .height(25)
                    .width(Fill),
                )
                .width(Fill)
                .height(Shrink)
                .style(style::container_medium);

                let video_aspect = video.width as f32 / video.height as f32;

                let stats_bar = container(
                    row![
                        text(format!("{}x{}", video.width, video.height))
                            .size(13)
                            .center()
                            .width(Shrink)
                            .height(Fill),
                        // mute_unmute_button,
                    ]
                    .height(20)
                    .spacing(3)
                    .width(Fill),
                )
                .style(style::container_medium)
                .width(Fill)
                .height(Shrink);

                let mut stack_children: Vec<Element<Message>> = vec![
                    Element::new(GifPlayer::new(player_state, video.width, video.height)
                        .width(Fill)
                        .height(Fill)
                        .content_fit(ContentFit::Contain)
                        .on_advance_frame(Message::OnNewGifFrame)),
                ];

                // TODO extract this out of here,
                // move to screens/crop.rs
                if let Some(Screen::Crop(crop_info)) = self.screen.as_ref() {
                    let top_left_pixel = crop_info.get_top_left_pixel(&video);
                    let x_left_prop =
                        (top_left_pixel.0 as f32 / video.width as f32).clamp(0.0f32, 1.0f32);
                    let y_top_prop =
                        (top_left_pixel.1 as f32 / video.height as f32).clamp(0.0f32, 1.0f32);

                    let mut width_prop =
                        (crop_info.width as f32 / video.width as f32).clamp(0.0f32, 1.0f32);
                    let mut height_prop =
                        (crop_info.height as f32 / video.height as f32).clamp(0.0f32, 1.0f32);

                    width_prop = f32::min(width_prop, 1.0f32 - x_left_prop);
                    height_prop = f32::min(height_prop, 1.0f32 - y_top_prop);

                    let overlay = Canvas::new(RectOverlay {
                        rect: NormRect {
                            x_left: x_left_prop,
                            y_top: y_top_prop,
                            width: width_prop,
                            height: height_prop,
                        },
                        video_aspect_ratio: video_aspect,
                    });
                    stack_children.push(overlay.width(Fill).height(Fill).into());
                }

                let video_stack = stack(stack_children);

                let video_container = container(video_stack)
                    .width(Fill)
                    .height(Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center);

                container(column![
                    video_container,
                    stats_bar,
                    player_bar.width(Fill).height(Shrink),
                ])
                .style(style::container_dark)
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // We use a column: a simple vertical layout

        let top_bar = container(
            row![
                button(text("Open").center().width(Fill).height(Fill).size(13))
                    .width(Shrink)
                    .height(Fill)
                    .on_press(Message::OpenFile)
                    .style(style::button_full),
                text(match self.video.as_ref() {
                    None => "No file selected".to_string(),
                    Some(i) => i.path.to_str().unwrap().to_owned(),
                })
                .size(13)
                .width(Fill)
                .height(Fill)
                .center()
            ]
            .spacing(2),
        )
        .width(Fill)
        .height(Shrink)
        .style(style::container_medium);

        let mut col = column![top_bar.width(Fill).height(20)];

        if let Some(video) = self.video.as_ref() {
            col = col.push(
                container(
                    row![
                        button("Trim")
                            .on_press(Message::GoToScreen(Some(ScreenName::Trim)))
                            .style(if matches!(self.screen, Some(Screen::Trim(_))) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Crop")
                            .on_press(Message::GoToScreen(Some(ScreenName::Crop)))
                            .style(if matches!(self.screen, Some(Screen::Crop(_))) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Scale")
                            .on_press(Message::GoToScreen(Some(ScreenName::Scale)))
                            .style(if matches!(self.screen, Some(Screen::Scale(_))) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Volume")
                            .on_press(Message::GoToScreen(Some(ScreenName::Volume)))
                            .style(if matches!(self.screen, Some(Screen::Volume(_))) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Pipeline")
                            .on_press(Message::GoToScreen(Some(ScreenName::Pipeline)))
                            .style(if matches!(self.screen, Some(Screen::Pipeline)) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Gif")
                            .on_press(Message::GoToScreen(Some(ScreenName::GifFrames)))
                            .style(if matches!(self.screen, Some(Screen::GifFrames(_))) {
                                style::button_rounded_options_selected
                            } else {
                                style::button_rounded_options_deselected
                            }),
                        button("Save")
                            .on_press(Message::TrySave)
                            .style(style::button_rounded_options_deselected),
                    ]
                    .width(Fill)
                    .height(Shrink)
                    .padding(5)
                    .spacing(5),
                )
                .width(Fill)
                .height(Shrink)
                .style(style::container_color),
            );

            col = col.push(screens::view_screen(self));

            col = col.push(self.view_video_player(video));

            col = col.push(
                Container::new(text(self.bottom_bar.message.to_string()).size(13))
                    .width(Fill)
                    .height(15)
                    .style(match self.bottom_bar.state {
                        BottomBarState::Normal => style::container_medium,
                        BottomBarState::InProgress => style::bottom_bar_in_progress,
                        BottomBarState::Error => style::bottom_bar_error,
                    }),
            );
        } else {
            col = col.push(
                container(
                    button("Open a video file")
                        .on_press(Message::OpenFile)
                        .style(style::button_rounded_options_deselected),
                )
                .width(Fill)
                .height(Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(style::container_dark),
            )
        }

        col.width(Fill).height(Fill).into()
    }
}

impl Counter {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => match pick_file() {
                Some(buf) => Task::done(Message::FileSelected(buf)),
                None => Task::none(),
            },
            Message::FileSelected(path_buf) => {
                let new_vid_data = match VideoData::new(path_buf.clone()) {
                    Err(e) => {
                        self.bottom_bar.set_error(e);
                        return Task::none();
                    }
                    Ok(vid) => vid,
                };
                let width = new_vid_data.width;
                let height = new_vid_data.height;
                let duration = new_vid_data.duration;
                self.video = Some(new_vid_data); // move
                self.pipeline = Some(Pipeline::new(path_buf, width, height, duration));

                Task::none()
            }
            Message::FileDeselected => {
                self.video = None;
                self.pipeline = None;
                Task::none()
            }
            Message::VideoPlayerEnds => Task::none(),
            Message::VideoPlayerNewFrame => {
                if let Some(vid) = self.video.as_mut() {
                    match &mut vid.kind {
                        VideoKind::Video { player } => {
                            vid.current_time = player.position().as_secs_f64();
                        }
                        _ => { /* TODO OnNewFrame for gif */ }
                    }
                }
                Task::none()
            }
            Message::SetPlaying(playing) => {
                if let Some(vid) = self.video.as_mut() {
                    match &mut vid.kind {
                        VideoKind::Video { player } => player.set_paused(!playing),
                        VideoKind::Gif { player_state, .. } => player_state.set_playing(playing),
                    }
                }
                Task::none()
            }
            Message::SetMute(val) => {
                if let Some(vid) = self.video.as_mut()
                    && let VideoKind::Video { player, .. } = &mut vid.kind
                {
                    player.set_muted(val);
                }
                Task::none()
            }
            Message::VideoPlayerSetVolume(new_volume) => {
                if let Some(vid) = self.video.as_mut()
                    && let VideoKind::Video { player, .. } = &mut vid.kind
                {
                    player.set_volume(new_volume);
                }
                Task::none()
            }
            Message::GoToScreen(screen_name) => {
                self.screen = if let Some(vid) = self.video.as_ref()
                    && let Some(screen_name) = screen_name
                {
                    Some(screen_name.to_screen(&vid))
                } else {
                    None
                };
                Task::none()
            }
            Message::PlayerSliderChanged(new_val) => {
                
                if let Some(vid) = self.video.as_mut()
                    && let VideoKind::Video { player, .. } = &mut vid.kind
                {
                    vid.current_time = new_val;
                    player
                        .seek(std::time::Duration::from_secs_f64(new_val), false)
                        .unwrap(); // TODO deal with this error somehow
                }
                Task::none()
            }
            Message::SetTrimStartToNow => {
                if let Some(vid) = self.video.as_mut() {
                    if let Some(Screen::Trim(trim_info)) = self.screen.as_mut() {
                        trim_info.start_frame_time = vid.current_time;
                        trim_info.start_frame_text = format!("{:.4}", vid.current_time);
                    }
                }
                Task::none()
            }
            Message::SetTrimEndToNow => {
                if let Some(vid) = self.video.as_mut() {
                    if let Some(Screen::Trim(trim_info)) = self.screen.as_mut() {
                        trim_info.end_frame_time = vid.current_time;
                        trim_info.end_frame_text = format!("{:.4}", vid.current_time);
                    }
                } else {
                    // ERROR!!! TODO
                }
                Task::none()
            }
            Message::TrimStartTimeChanged(new_val) => {
                if let Some(Screen::Trim(trim_info)) = self.screen.as_mut() {
                    let string_to_set: String = new_val
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    match string_to_set.parse::<f64>() {
                        Ok(parse_res) => {
                            trim_info.start_frame_text = string_to_set;
                            trim_info.start_frame_time = parse_res;
                        }
                        Err(_) => {
                            // TODO put in error
                            trim_info.start_frame_text = "0".to_string();
                            trim_info.start_frame_time = 0f64;
                        }
                    }
                }

                Task::none()
            }
            Message::TrimEndTimeChanged(new_val) => {
                if let Some(Screen::Trim(trim_info)) = self.screen.as_mut() {
                    let string_to_set: String = new_val
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    match string_to_set.parse::<f64>() {
                        Ok(parse_res) => {
                            trim_info.end_frame_text = string_to_set;
                            trim_info.end_frame_time = parse_res;
                        }
                        Err(_) => {
                            // TODO put in error
                            trim_info.end_frame_text = "0".to_string();
                            trim_info.end_frame_time = 0f64;
                        }
                    }
                }

                Task::none()
            }
            Message::StepFrames(i) => {
                if let Some(vid) = self.video.as_mut()
                    && let VideoKind::Video { player, .. } = &mut vid.kind
                {
                    if i > 0 {
                        for _ in 0..i {
                            player.step_one_frame();
                        }
                        vid.current_time = player.position().as_secs_f64();
                    } else if i < 0 {
                        let target_loc: Duration = player.position().saturating_sub(
                            Duration::from_secs_f64(-i as f64 / player.framerate()),
                        );
                        player.seek(target_loc, true).unwrap(); // TODO deal w error
                        vid.current_time = player.position().as_secs_f64();
                    } else {
                        // do nothing
                    }
                }
                Task::none()
            }
            Message::TryTrim => {
                if let Some(video) = self.video.as_ref()
                    && let Some(Screen::Trim(trim_info)) = self.screen.as_mut()
                    && let Some(pipeline) = self.pipeline.as_ref()
                {
                    self.bottom_bar
                        .set_in_progress("Trimming video...".to_string());
                    trim_info.fit_to_bounds(&video, true);

                    let video_path = video.path.clone();
                    let tempdir_path = pipeline.get_tempdir_path();
                    let start_time = trim_info.start_frame_time;
                    let end_time = trim_info.end_frame_time;
                    let pipeline_step = pipeline.get_next_pipeline_step();
                    Task::perform(
                        async move {
                            ffmpeg::trim_vid(
                                tempdir_path,
                                &video_path,
                                start_time,
                                end_time,
                                pipeline_step,
                            )
                        },
                        Message::TrimFinished,
                    )
                } else {
                    Task::none()
                }
            }
            Message::TrimFinished(res) => {
                match res {
                    Ok(path_buf) => {
                        let vid_result = VideoData::new(path_buf.clone());
                        if let Err(e) = vid_result {
                            self.bottom_bar.set_error(e);
                            return Task::none();
                        };
                        let vid = vid_result.unwrap();
                        let width = vid.width;
                        let height = vid.height;
                        let duration = vid.duration;

                        self.video = Some(vid); // move vid
                        let pipeline = self
                            .pipeline
                            .as_mut()
                            .expect("Need pipeline to exist here, otherwise issues.");

                        pipeline.add(path_buf, pipeline::Operation::Trim, width, height, duration);

                        self.bottom_bar.set_normal();
                    }
                    Err(e) => {
                        self.bottom_bar.set_error(e);
                    }
                }

                Task::none()
            }
            Message::TrySave => {
                match FileDialog::new()
                    .set_title("Save Video")
                    .add_filter("Video", &["mp4", "mkv", "avi", "mov"])
                    .add_filter("Sound", &["wav", "flac", "aiff", "mp3", "ogg", "raw"])
                    .add_filter("All Files", &["*"])
                    .save_file()
                {
                    Some(path) => Task::done(Message::SaveTo(path)),
                    None => Task::none(),
                }
            }
            Message::SaveTo(path_buf) => {
                // copy from temp directory, then flush temp directory
                if let Some(video) = self.video.as_mut() {
                    // None => good! success
                    // Some(s) => error, with s message
                    self.bottom_bar
                        .set_in_progress("Saving video...".to_string());

                    let outcome: Option<String> = match path_buf
                        .extension()
                        .and_then(|f| f.to_str())
                    {
                        Some("mkv") => {
                            let stored_path_buf = path_buf.clone();
                            if std::fs::copy(&video.path, path_buf).is_err() {
                                // TODO deal with error
                                Some("Could not copy to new path".to_string())
                            } else {
                                None
                            }
                        }
                        Some("mp4") => {
                            if ffmpeg::save_video_default(&video.path, &path_buf) {
                                None
                            } else {
                                Some(
                                    "Could not save video using mp4 (default) encoding".to_string(),
                                )
                            }
                        }
                        Some("mov") => {
                            if ffmpeg::save_video_default(&video.path, &path_buf) {
                                None
                            } else {
                                Some(
                                    "Could not save video using mov (default) encoding".to_string(),
                                )
                            }
                        }
                        Some("avi") => {
                            if ffmpeg::save_video_to_avi(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using avi encoding".to_string())
                            }
                        }
                        Some("aiff") => {
                            if ffmpeg::save_video_to_aiff(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using aiff encoding".to_string())
                            }
                        }
                        Some("flac") => {
                            if ffmpeg::save_video_to_flac(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using flac encoding".to_string())
                            }
                        }
                        Some("mp3") => {
                            if ffmpeg::save_video_to_mp3(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using mp3 encoding".to_string())
                            }
                        }
                        Some("ogg") => {
                            if ffmpeg::save_video_to_ogg(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using ogg encoding".to_string())
                            }
                        }
                        Some("raw") => {
                            if ffmpeg::save_video_to_raw(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using raw encoding".to_string())
                            }
                        }
                        Some("wav") => {
                            if ffmpeg::save_video_to_wav(&video.path, &path_buf) {
                                None
                            } else {
                                Some("Could not save video using wav encoding".to_string())
                            }
                        }
                        Some(ext) => Some(format!("File extension {} invalid for saving", ext)),
                        None => Some("No extension provided in save path".to_string()),
                    };

                    match outcome {
                        None => self.bottom_bar.set_normal(),
                        Some(e) => self.bottom_bar.set_error(e),
                    }
                } else {
                    self.bottom_bar
                        .set_error("No video loaded, cannot save".to_string());
                }
                Task::none()
            }
            Message::CropLeftChanged(new_val) => {
                if let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    match &mut crop_info.crop_mode {
                        CropMode::TopLeft { x, text_x, .. } => {
                            restrict_to_u16_input(new_val, text_x, x, 0u16, video.width);
                        }
                        CropMode::PixelCenter { x, text_x, .. } => {
                            restrict_to_u16_input(new_val, text_x, x, 0u16, video.width);
                        }
                        CropMode::VideoCenter => { /* not possible */ }
                    }
                }
                Task::none()
            }
            Message::CropTopChanged(new_val) => {
                if let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    match &mut crop_info.crop_mode {
                        CropMode::TopLeft { y, text_y, .. } => {
                            restrict_to_u16_input(new_val, text_y, y, 0u16, video.height);
                        }
                        CropMode::PixelCenter { y, text_y, .. } => {
                            restrict_to_u16_input(new_val, text_y, y, 0u16, video.height);
                        }
                        CropMode::VideoCenter => { /* not possible */ }
                    }
                }
                Task::none()
            }
            Message::CropWidthChanged(new_val) => {
                if let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    restrict_to_u16_input(
                        new_val,
                        &mut crop_info.text_width,
                        &mut crop_info.width,
                        0,
                        video.width,
                    );
                }
                Task::none()
            }
            Message::CropHeightChanged(new_val) => {
                if let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    restrict_to_u16_input(
                        new_val,
                        &mut crop_info.text_height,
                        &mut crop_info.height,
                        0,
                        video.height,
                    );
                }
                Task::none()
            }
            Message::TryCrop => {
                if let Some(video) = self.video.as_ref()
                    && let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(pipeline) = self.pipeline.as_ref()
                {
                    self.bottom_bar
                        .set_in_progress("Cropping video...".to_string());
                    let top_left_pixel = crop_info.convert_top_left(&video);
                    crop_info.fit_to_bounds(&video, true);

                    let video_path = video.path.clone();
                    let tempdir_path = pipeline.get_tempdir_path();
                    let x_pixel = top_left_pixel.0;
                    let y_pixel = top_left_pixel.1;
                    let width = crop_info.width;
                    let height = crop_info.height;
                    let pipeline_step = pipeline.get_next_pipeline_step();
                    Task::perform(
                        async move {
                            ffmpeg::crop_vid(
                                tempdir_path,
                                &video_path,
                                x_pixel,
                                y_pixel,
                                width,
                                height,
                                pipeline_step,
                            )
                        },
                        Message::CropFinished,
                    )
                } else {
                    Task::none()
                }
            }
            Message::CropFinished(res) => {
                match res {
                    Ok(path_buf) => {
                        let pipeline = self
                            .pipeline
                            .as_mut()
                            .expect("Need pipeline to exist here, otherwise issues.");

                        let vid = VideoData::new(path_buf.clone());
                        if let Err(e) = vid {
                            self.bottom_bar.set_error(e);
                            return Task::none();
                        }
                        let vid = vid.unwrap();
                        let width = vid.width;
                        let height = vid.height;
                        let duration = vid.duration;

                        self.video = Some(vid);

                        pipeline.add(path_buf, pipeline::Operation::Crop, width, height, duration);
                        self.bottom_bar.set_normal();
                    }
                    Err(e) => self.bottom_bar.set_error(e),
                }

                Task::none()
            }
            Message::ScaleWidthChanged(new_val) => {
                if let Some(Screen::Scale(scale_info)) = self.screen.as_mut() {
                    restrict_to_u32_input(
                        new_val,
                        &mut scale_info.text_width,
                        &mut scale_info.width,
                        0,
                        u32::MAX,
                    );
                }
                Task::none()
            }
            Message::ScaleHeightChanged(new_val) => {
                if let Some(Screen::Scale(scale_info)) = self.screen.as_mut() {
                    restrict_to_u32_input(
                        new_val,
                        &mut scale_info.text_height,
                        &mut scale_info.height,
                        0,
                        u32::MAX,
                    );
                }
                Task::none()
            }
            Message::KeepAspectRatioToggled(new_val) => {
                if let Some(Screen::Scale(scale_info)) = self.screen.as_mut() {
                    scale_info.keep_aspect_ratio = new_val;
                }
                Task::none()
            }
            Message::ScaleProportionChanged(new_val) => {
                if let Some(Screen::Scale(scale_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    restrict_to_f32_input(
                        new_val,
                        &mut scale_info.text_prop,
                        &mut scale_info.prop,
                        0f32,
                        f32::MAX,
                    );
                }
                Task::none()
            }
            Message::TryScale => {
                if let Some(video) = self.video.as_ref()
                    && let Some(Screen::Scale(scale_info)) = self.screen.as_mut()
                    && let Some(pipeline) = self.pipeline.as_ref()
                {
                    self.bottom_bar
                        .set_in_progress("Scaling video...".to_string());
                    scale_info.fit_to_bounds(&video, true);
                    let video_path = video.path.clone();
                    let tempdir_path = pipeline.get_tempdir_path();
                    let pipeline_step = pipeline.get_next_pipeline_step();

                    if scale_info.keep_aspect_ratio {
                        let scale_factor = scale_info.prop;

                        Task::perform(
                            async move {
                                ffmpeg::scale_vid_prop(
                                    tempdir_path,
                                    &video_path,
                                    scale_factor,
                                    pipeline_step,
                                )
                            },
                            Message::ScaleFinished,
                        )
                    } else {
                        let new_w = scale_info.width;
                        let new_h = scale_info.height;
                        Task::perform(
                            async move {
                                ffmpeg::scale_vid_size(
                                    tempdir_path,
                                    &video_path,
                                    new_w,
                                    new_h,
                                    pipeline_step,
                                )
                            },
                            Message::ScaleFinished,
                        )
                    }
                } else {
                    Task::none()
                }
            }
            Message::ScaleFinished(res) => {
                match res {
                    Ok(path_buf) => {
                        let pipeline = self
                            .pipeline
                            .as_mut()
                            .expect("Need pipeline to exist here, otherwise issues.");

                        let vid = VideoData::new(path_buf.clone());
                        if let Err(e) = vid {
                            self.bottom_bar.set_error(e);
                            return Task::none();
                        }
                        let vid = vid.unwrap();
                        let width = vid.width;
                        let height = vid.height;
                        let duration = vid.duration;

                        self.video = Some(vid);
                        pipeline.add(
                            path_buf,
                            pipeline::Operation::Scale,
                            width,
                            height,
                            duration,
                        );
                        self.bottom_bar.set_normal();
                    }
                    Err(e) => self.bottom_bar.set_error(e),
                }

                Task::none()
            }
            Message::CropTypeSelected(crop_type) => {
                if let Some(Screen::Crop(crop_info)) = self.screen.as_mut()
                    && let Some(video) = self.video.as_ref()
                {
                    crop_info.convert_to(&video, crop_type);
                }
                Task::none()
            }
            Message::RevertToPipeline(idx) => {
                if let Some(pipeline) = self.pipeline.as_mut() {
                    match pipeline.get(idx) {
                        Some(new_buf) => {
                            let vid = VideoData::new(new_buf);
                            if let Err(e) = vid {
                                self.bottom_bar.set_error(e);
                                return Task::none();
                            }

                            pipeline.destroy_steps_after(idx);
                        }
                        None => self
                            .bottom_bar
                            .set_error(format!("Pipeline lacked step {}", idx)),
                    }
                } else {
                    self.bottom_bar
                        .set_error("Tried to revert to pipeline when none exists?".to_string());
                }
                Task::none()
            }
            Message::VolumePropChanged(new_val) => {
                if let Some(Screen::Volume(volume_info)) = self.screen.as_mut() {
                    restrict_to_f32_input(
                        new_val,
                        &mut volume_info.text_prop,
                        &mut volume_info.prop,
                        0f32,
                        f32::MAX,
                    );
                }
                Task::none()
            }
            Message::TryVolume => {
                if let Some(video) = self.video.as_ref()
                    && let Some(Screen::Volume(volume_info)) = self.screen.as_mut()
                    && let Some(pipeline) = self.pipeline.as_ref()
                {
                    self.bottom_bar
                        .set_in_progress("Changing volume of video...".to_string());
                    volume_info.fit_to_bounds(&video, true);
                    let video_path = video.path.clone();
                    let tempdir_path = pipeline.get_tempdir_path();
                    let pipeline_step = pipeline.get_next_pipeline_step();

                    let new_volume = volume_info.prop;

                    Task::perform(
                        async move {
                            ffmpeg::change_vid_volume(
                                tempdir_path,
                                &video_path,
                                new_volume,
                                pipeline_step,
                            )
                        },
                        Message::VolumeFinished,
                    )
                } else {
                    Task::none()
                }
            }
            Message::VolumeFinished(res) => {
                match res {
                    Ok(path_buf) => {
                        let pipeline = self
                            .pipeline
                            .as_mut()
                            .expect("Need pipeline to exist here, otherwise issues.");

                        let vid = VideoData::new(path_buf.clone());
                        if let Err(e) = vid {
                            self.bottom_bar.set_error(e);
                            return Task::none();
                        }
                        let vid = vid.unwrap();
                        let width = vid.width;
                        let height = vid.height;
                        let duration = vid.duration;

                        self.video = Some(vid);

                        pipeline.add(
                            path_buf,
                            pipeline::Operation::Volume,
                            width,
                            height,
                            duration,
                        );
                        self.bottom_bar.set_normal();
                    }
                    Err(e) => self.bottom_bar.set_error(e),
                }
                Task::none()
            }
            Message::FramesToGif => todo!(),
            Message::OnNewGifFrame(next_frame_idx, time_from_start) => {
                if let Some(VideoData { current_time, kind, .. }) = self.video.as_mut() {
                    *current_time = time_from_start.as_secs_f64();
                    if let VideoKind::Gif {player_state, ..} = kind {
                        // ??? what todo here?

                    }
                }

                Task::none()
            },
            Message::GifFrameSetDuration { idx, new_duration_centiseconds } => {
                if let Some(VideoData {kind, ..}) = self.video.as_mut() && let VideoKind::Gif { player_state, ..} = kind {
                    if let Some(frame) = player_state.frames.get_mut(idx) {
                        frame.duration = Duration::from_millis(new_duration_centiseconds as u64 * 10u64);
                    }
                }
                Task::none()
            },
            Message::GifFrameSetHidden { idx, hidden } => {
                if let Some(VideoData {kind, ..}) = self.video.as_mut() && let VideoKind::Gif { player_state, .. } = kind {
                    if let Some(frame) = player_state.frames.get_mut(idx) {
                        frame.hidden = hidden;
                    }
                }
                Task::none()
            },
            Message::GifFrameCopy { idx } => {
                if let Some(VideoData {kind, ..}) = self.video.as_mut() && let VideoKind::Gif { player_state, .. } = kind {
                    if let Some(frame) = player_state.frames.get(idx) {
                        player_state.frames.insert(idx, GifFrame{
                            handle: frame.handle.clone(),
                            duration: frame.duration.clone(),
                            hidden: frame.hidden.clone(),
                        });
                    }
                }
                Task::none()
            },
            Message::GifFrameSwap { source_idx, dest_idx } => {
                if let Some(VideoData {kind, ..}) = self.video.as_mut() && let VideoKind::Gif { player_state, .. } = kind {
                    player_state.frames.swap(source_idx, dest_idx);
                }
                Task::none()
            },
            Message::StepCentiseconds(_) => todo!(),
        }
    }
}

pub fn restrict_to_f32_input(
    string: String,
    restricted_string: &mut String,
    restricted_f32: &mut f32,
    clamp_min: f32,
    clamp_max: f32,
) {
    let string_to_set: String = string
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    match string_to_set.parse::<f32>() {
        Ok(parsed_val) => {
            let clamped_val = parsed_val.clamp(clamp_min, clamp_max);
            *restricted_string = string_to_set;
            *restricted_f32 = clamped_val;
        }
        Err(_) => {
            // TODO print error msg
            *restricted_string = "0.0".to_string();
            *restricted_f32 = 0.0f32;
        }
    }
}

pub fn restrict_to_u16_input(
    string: String,
    restricted_string: &mut String,
    restricted_value: &mut u16,
    clamp_min: u16,
    clamp_max: u16,
) {
    let string_to_set: String = string.chars().filter(|c| c.is_ascii_digit()).collect();
    match string_to_set.parse::<u16>() {
        Ok(parsed_val) => {
            let clamped_val = parsed_val.clamp(clamp_min, clamp_max);
            *restricted_string = parsed_val.to_string();
            *restricted_value = clamped_val;
        }
        Err(_) => {
            // TODO print error msg
            *restricted_string = "0".to_string();
            *restricted_value = 0;
        }
    }
}

pub fn restrict_to_u32_input(
    string: String,
    restricted_string: &mut String,
    restricted_u32: &mut u32,
    clamp_min: u32,
    clamp_max: u32,
) {
    let string_to_set: String = string.chars().filter(|c| c.is_ascii_digit()).collect();
    match string_to_set.parse::<u32>() {
        Ok(parsed_val) => {
            let clamped_val = parsed_val.clamp(clamp_min, clamp_max);
            *restricted_string = parsed_val.to_string();
            *restricted_u32 = clamped_val;
        }
        Err(_) => {
            // TODO print error msg
            *restricted_string = "0".to_string();
            *restricted_u32 = 0;
        }
    }
}

fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}
