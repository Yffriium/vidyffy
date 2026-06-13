use std::{ops::{Deref, DerefMut}, sync::{Arc, RwLock}, time::{Duration, Instant}};

use iced::{ContentFit, Event, Length, Renderer, Size, Theme, advanced::{Widget, widget::tree}, mouse::Cursor, widget::image::Handle, window};

use iced::advanced::image::{Renderer as ImageRenderer};
use iced::Rectangle;

use crate::Message;


struct GifInternalState {
    playing: bool,

    current_frame: u32,
    time_from_start: Duration,

    last_update_time: Instant, // used for dt

    current_duration_in_frame: Duration,
    goal_duration_in_frame: Duration,
}

pub struct GifState{
    internals: RwLock<GifInternalState>,
    frames: Vec<GifFrame>,
    num_visible_frames: u32,
    stored_duration: Duration
}

impl GifState {
    fn write(&self) -> impl DerefMut<Target = GifInternalState> + '_ {
        self.internals.write().expect("Write lock")
    }

    fn read(&self) -> impl Deref<Target = GifInternalState> + '_ {
        self.internals.read().expect("Read lock")
    }

    /// Based on stored values, recalculates the time from start.
    /// TODO put this in the right places
    fn calc_time_from_start(frames: &Vec<GifFrame>, current_frame: usize, current_duration_in_frame: Duration) -> Duration {
        current_duration_in_frame + frames.iter().take(current_frame).fold(Duration::ZERO, |acc, elt| acc + elt.duration)
    }


    pub fn new(frames: Vec<GifFrame>) -> Self {
        let num_visible_frames = frames.len() as u32;
        let stored_duration = frames.iter().fold(Duration::ZERO, |acc, elt| acc + elt.duration);
        Self {
            internals: RwLock::new(GifInternalState { playing: true, current_frame: 0, last_update_time: Instant::now(), goal_duration_in_frame: Duration::from_millis(100), time_from_start: Duration::ZERO, current_duration_in_frame: Duration::ZERO }), // TODO use first frame data, not 100
            frames,
            num_visible_frames,
            stored_duration
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

    pub fn get_frame(&self, idx: usize) -> Option<&GifFrame> {
        self.frames.get(idx)
    }

    pub fn set_frame_duration(&mut self, idx: usize, duration: Duration) {
        self.frames.get_mut(idx).map(|elt| {
            // update stored val
            self.stored_duration += duration;
            self.stored_duration -= elt.duration;

            // modify elt
            elt.duration = duration
        });
    }

    pub fn set_frame_hidden(&mut self, idx: usize, hidden: bool) {
        self.frames.get_mut(idx).map(|elt| {
            // update stored val
            if elt.hidden && !hidden {
                self.num_visible_frames += 1;
            }
            if !elt.hidden && hidden {
                self.num_visible_frames -= 1;

                // TODO need to go to start
                // self.write().current_frame = 0; // go to start, just for safety
            }

            // modify elt
            elt.hidden = hidden
        });
    }

    

    // dont expose this bc we need to track changes
    // pub fn get_frame_mut(&mut self, idx: usize) -> Option<&mut GifFrame> {
    //     self.frames.get_mut(idx)
    // }

    pub fn copy_frame_at(&mut self, idx: usize) -> bool {
        let found_frame = self.frames.get(idx);
        if found_frame.is_none() {
            return false;
        }
        let found_frame = found_frame.unwrap();
        
        self.frames.insert(idx, GifFrame { handle: found_frame.handle.clone(), duration: found_frame.duration.clone(), hidden: found_frame.hidden.clone() });
        true
    }

    pub fn swap_frames(&mut self, source_idx: usize, dest_idx: usize) {
        self.frames.swap(source_idx, dest_idx);
    }

    pub fn frames_iter(&self) -> std::slice::Iter<'_, GifFrame> {
        self.frames.iter()
    }


    pub fn set_current_frame(&mut self, value: u32) {
        let mut inner = self.write();
        let frame = self.frames.get(value as usize);
        if frame.is_none() {
            return;
        }
        let frame = frame.unwrap();

        inner.current_duration_in_frame = Duration::ZERO;
        inner.goal_duration_in_frame = frame.duration;
        inner.current_frame = value;
        inner.last_update_time = Instant::now();
        inner.time_from_start = Self::calc_time_from_start(&self.frames, value as usize, Duration::ZERO); // TODO do this
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
                inner.goal_duration_in_frame = current_frame.duration;
                inner.current_duration_in_frame = remaining_dur;
                inner.time_from_start = duration; // TODO not very good at all
                inner.last_update_time = Instant::now();
                break;
            } else {
                remaining_dur -= current_frame.duration;
                inner.current_frame = (inner.current_frame + 1) % num_frames as u32;
            }
        }
    }

    pub fn total_frames(&self) -> u32 {
        self.frames.len() as u32
    }

    pub fn num_visible_frames(&self) -> u32 {
        self.num_visible_frames
    }

    pub fn duration(&self) -> Duration {
        self.stored_duration
    }

    pub fn time_from_start(&self) -> Duration {
        self.read().time_from_start
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
    on_advance_frame: Option<Box<dyn Fn(u32, Duration) -> Message + 'a>>,
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

    pub fn on_advance_frame(mut self, on_advance_frame: impl Fn(u32, Duration) -> Message + 'a) -> Self {
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
        
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let mut gif_state = self.gif_state.write();

            // push next 
            let dt = *now - gif_state.last_update_time;
            gif_state.last_update_time = *now;

            // eprintln!("Update {}", dt.as_secs_f64());

            if gif_state.playing {

                
                

                gif_state.current_duration_in_frame += dt;
                gif_state.time_from_start += dt;

            
                while gif_state.current_duration_in_frame >= gif_state.goal_duration_in_frame {
                    let sub_amt = gif_state.goal_duration_in_frame;
                    gif_state.current_duration_in_frame -= sub_amt;
                    


                    // this block does one of the following.
                    // 1. most common case, just advances to the next non-hidden frame, skipping all that are hidden.
                    //  usually, none will be hidden so it will just advance to next.
                    // 2. unusual case. if all frames are hidden, then it loops back around and just ends up back at
                    //  the original spot, so it does not advance to the next frame.
                    //  maybe this case can be made impossible by disallowing all hiddens
                    let orig = gif_state.current_frame;
                    loop {
                        gif_state.current_frame = gif_state.current_frame + 1;
                        if gif_state.current_frame as usize >= self.gif_state.frames.len() {
                            gif_state.current_frame = 0;
                            gif_state.time_from_start = gif_state.current_duration_in_frame;
                        }
                        if !self.gif_state.frames[gif_state.current_frame as usize].hidden || gif_state.current_frame == orig {
                            break;
                        }
                    }

                
                    gif_state.goal_duration_in_frame = self.gif_state.frames[gif_state.current_frame as usize].duration;
                    if let Some(on_advance_frame) = &self.on_advance_frame {
                        let msg = on_advance_frame(gif_state.current_frame, gif_state.time_from_start);
                        shell.publish(msg);
                    }
                }
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