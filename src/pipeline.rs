use std::{fs, path::PathBuf};

use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    wgpu::wgc::pipeline,
    widget::{
        Column, Row, button, column, container,
        scrollable::{Direction, Scrollbar},
        text,
    },
};
use iced_video_player::Video;
use tempfile::{TempDir, tempdir};

use crate::{Message, style};

pub struct Pipeline {
    pipeline: Vec<PipelineEntry>,
    tempdir: TempDir,
}

impl Pipeline {
    pub fn get_tempdir_path(&self) -> PathBuf {
        self.tempdir.path().to_path_buf()
    }

    pub fn get_next_pipeline_step(&self) -> usize {
        self.pipeline.len()
    }

    pub fn new(path: PathBuf, video: &Video) -> Self {
        let size = video.size();

        Self {
            pipeline: vec![PipelineEntry {
                step_num: 0,
                path,
                operation: Operation::Default,
                width: size.0 as u32,
                height: size.1 as u32,
                duration: video.duration().as_secs_f64(),
            }],
            tempdir: tempdir().expect("Could not make a tempdir. Need more space."),
        }
    }

    pub fn add(
        &mut self,
        path: PathBuf,
        operation: Operation,
        width: u32,
        height: u32,
        duration: f64,
    ) {
        let pipeline_size = self.pipeline.len();
        self.pipeline.push(PipelineEntry {
            step_num: pipeline_size,
            path,
            operation,
            width,
            height,
            duration,
        });
    }

    pub fn add_from_vid(&mut self, path: PathBuf, operation: Operation, video: &Video) {
        let pipeline_size = self.pipeline.len();
        let vid_size = video.size();
        let vid_duration = video.duration().as_secs_f64();
        self.pipeline.push(PipelineEntry {
            step_num: pipeline_size,
            path,
            operation,
            width: vid_size.0 as u32,
            height: vid_size.1 as u32,
            duration: vid_duration,
        });
    }

    pub fn destroy_steps_after(&mut self, step_num: usize) {
        let mut rest = self.pipeline.split_off(step_num + 1);

        rest.drain(..).for_each(|elt| {
            // delete the invalid files
            fs::remove_file(elt.path).expect("Could not remove file??");
        });
    }

    pub fn get(&self, step_num: usize) -> Option<PathBuf> {
        self.pipeline.get(step_num).map(|elt| elt.path.clone())
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let elts = Row::from_iter(self.pipeline.iter().enumerate().map(|(idx, elt)| {
            // turn into a card
            container(
                Column::new()
                    .push(
                        button(
                            text(format!("{}", elt.step_num))
                                .align_y(Center)
                                .align_x(Center),
                        )
                        .padding(5)
                        .width(Fill)
                        .height(24)
                        .style(style::button_rounded_top_full)
                        .on_press(Message::RevertToPipeline(idx)),
                    )
                    .push(container(
                        column![
                            text(elt.operation.display()),
                            text(format!("{}x{}", elt.width, elt.height)),
                            text(format!("{:.1}s", elt.duration))
                        ]
                        .height(Shrink)
                        .width(Shrink)
                        .padding(5),
                    ))
                    .height(Shrink)
                    .width(Shrink),
            )
            .height(Shrink)
            .width(Shrink)
            .style(style::container_medium_rounded)
            .into()
            // column![
            //     text(elt.operation.display()),
            //     text(elt.path.to_string),
            //     text(elt.step_num),

            // ].into()
        }))
        .spacing(10)
        .height(Shrink)
        .width(Shrink);

        let scroll = iced::widget::scrollable(elts)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .height(Shrink) // must constrain height??
            .width(Shrink);

        container(scroll).width(Shrink).height(Shrink).into()
    }
}

pub struct PipelineEntry {
    step_num: usize,
    path: PathBuf,
    operation: Operation,
    width: u32,
    height: u32,
    duration: f64,
}

#[derive(Clone, Copy)]
pub enum Operation {
    Default,
    Trim,
    Crop,
    Scale,
    Volume,
}

impl Operation {
    pub fn display(self) -> &'static str {
        match self {
            Operation::Default => "Default",
            Operation::Trim => "Trim",
            Operation::Crop => "Crop",
            Operation::Scale => "Scale",
            Operation::Volume => "Volume",
        }
    }
}
