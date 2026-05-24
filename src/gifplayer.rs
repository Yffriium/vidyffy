use std::{ops::{Deref, DerefMut}, sync::{Arc, RwLock}, time::{Duration, Instant}};

use iced::{ContentFit, Event, Length, Renderer, Size, Theme, advanced::{Widget, widget::tree}, mouse::Cursor, widget::image::Handle, window};

use iced::advanced::image::{Renderer as ImageRenderer};
use iced::Rectangle;

use crate::Message;


struct GifInternalState {
    playing: bool,

    current_frame: u32,

    frame_start_playing: Instant,
    frame_duration: Duration,
}

pub struct GifState{
    internals: RwLock<GifInternalState>,
    pub frames: Vec<GifFrame>
}

impl GifState {
    fn write(&self) -> impl DerefMut<Target = GifInternalState> + '_ {
        self.internals.write().expect("Write lock")
    }

    fn read(&self) -> impl Deref<Target = GifInternalState> + '_ {
        self.internals.read().expect("Read lock")
    }


    pub fn new(frames: Vec<GifFrame>) -> Self {
        Self {
            internals: RwLock::new(GifInternalState { playing: true, current_frame: 0, frame_start_playing: Instant::now(), frame_duration: Duration::from_millis(100) }), // TODO use first frame data, not 100
            frames,
        }
    }

    pub fn playing(&self) -> bool {
        self.read().playing
    }

    pub fn current_frame(&self) -> u32 {
        self.read().current_frame
    }

    pub fn set_playing(&mut self, value: bool) {
        self.write().playing = value;
    }


    pub fn set_current_frame(&mut self, value: u32) {
        let mut inner = self.write();
        let frame = self.frames.get(value as usize);
        if frame.is_none() {
            return;
        }
        let frame = frame.unwrap();

        inner.frame_duration = frame.duration;
        inner.current_frame = value;
        inner.frame_start_playing = Instant::now();
    }

    /// expensive seek!
    /// TODO has infinite loop potential if durations are 0, fix this
    pub fn seek_to_time(&mut self, duration: Duration) {
        let mut inner = self.write();

        inner.current_frame = 0;
        let num_frames = self.frames.len();
        let mut remaining_dur = duration;



        loop {
            let current_frame = &self.frames[inner.current_frame as usize];
            if remaining_dur < current_frame.duration {
                inner.frame_duration = current_frame.duration;
                inner.frame_start_playing = Instant::now() - remaining_dur;
                break;
            } else {
                remaining_dur -= current_frame.duration;
                inner.current_frame = (inner.current_frame + 1) % num_frames as u32;
            }
        }
    }
    

}



pub struct GifFrame {
    pub handle: Handle,
    pub duration: Duration,
    pub hidden: bool,
}

pub struct GifPlayer<'a, Message> {
    gif_state: &'a GifState,
    width: Length,
    height: Length,
    on_advance_frame: Option<Box<dyn Fn(u32) -> Message + 'a>>,
    content_fit: ContentFit,
    frame_width: u16,
    frame_height: u16,
}

impl<'a> GifPlayer<'a, Message> {
    pub fn new(gif_state: &'a GifState, frame_width: u16, frame_height: u16) -> Self {
        Self { 
            gif_state,
            width: Length::Fill, 
            height: Length::Fill, 
            on_advance_frame: None, 
            content_fit: ContentFit::Contain,
            frame_width,
            frame_height
        }
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn on_advance_frame(mut self, on_advance_frame: impl Fn(u32) -> Message + 'a) -> Self {
        self.on_advance_frame = Some(Box::new(on_advance_frame));
        self
    }

    pub fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }
}


impl <'a> Widget<Message, Theme, Renderer> for GifPlayer<'a, Message> {
    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {

        
            
        let image_size = iced::Size::new(self.frame_width as f32, self.frame_height as f32);
        let max_size =limits.width(self.width).height(self.height).max();

        let final_size = match self.content_fit {
            ContentFit::Contain => {
                let scale_x = max_size.width / image_size.width;
                let scale_y = max_size.height / image_size.height;

                let scale = scale_x.min(scale_y).min(1.0);

                Size::new(image_size.width * scale, image_size.height * scale)
            },
            _ => image_size
        };
            
        
        
    
        iced::advanced::layout::Node::new(final_size)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {

        let frames = &self.gif_state.frames;
        if frames.is_empty() {
            return;
        }

        let state = self.gif_state.write();


        

        let bounds = layout.bounds();

        let frame_index = state.current_frame as usize;

        let frame = &frames[frame_index.min(frames.len() - 1)];

        let image = iced::advanced::image::Image::new(frame.handle.clone());

        renderer.draw_image(image, bounds, bounds);
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }
    
    
    fn children(&self) -> Vec<tree::Tree> {
        Vec::new()
    }

    fn update(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: &Event,
        layout: iced::advanced::Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {

        let mut gif_state = self.gif_state.write();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            // push next 
            if gif_state.playing {

                let frame_dur = gif_state.frame_duration;
            
                if *now - gif_state.frame_start_playing >= frame_dur {
                    
                    gif_state.frame_start_playing += frame_dur;


                    // this block does one of the following.
                    // 1. most common case, just advances to the next non-hidden frame, skipping all that are hidden.
                    //  usually, none will be hidden so it will just advance to next.
                    // 2. unusual case. if all frames are hidden, then it loops back around and just ends up back at
                    //  the original spot, so it does not advance to the next frame.
                    //  maybe this case can be made impossible by disallowing all hiddens
                    let orig = gif_state.current_frame;
                    loop {
                        gif_state.current_frame = (gif_state.current_frame + 1) % self.gif_state.frames.len() as u32;
                        if !self.gif_state.frames[gif_state.current_frame as usize].hidden || gif_state.current_frame == orig {
                            break;
                        }
                    }

                
                    gif_state.frame_duration = self.gif_state.frames[gif_state.current_frame as usize].duration;
                    if let Some(on_advance_frame) = &self.on_advance_frame {
                        let msg = on_advance_frame(gif_state.current_frame);
                        shell.publish(msg);
                    }
                }
            } else {
                // not playing, just set start time to now
                // gif_state.frame_start_playing = *now;
            }


            shell.request_redraw_at(*now + Duration::from_millis(16)); // max framerate is 60 for gif rendering 
        }
    }

    
}

fn next_frame_values(frames: &[GifFrame], current_frame: usize, time_elapsed: f64) -> (usize, f64) {
    if frames.is_empty() {
        return (current_frame, time_elapsed);
    }

    let expected_time = frames[current_frame].duration.as_secs_f64();

    if expected_time <= time_elapsed {
        ((current_frame + 1) % frames.len(), time_elapsed - expected_time)
    } else {
        (current_frame, time_elapsed)
    }
}