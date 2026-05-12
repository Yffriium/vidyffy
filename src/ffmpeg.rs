use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn save_video_to_wav(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-c:a",
            "pcm_s16le",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_flac(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-c:a",
            "flac",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_aiff(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-c:a",
            "pcm_s16be",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_mp3(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-c:a",
            "libmp3lame",
            "-q:a",
            "2",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_ogg(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-c:a",
            "libvorbis",
            "-q:a",
            "4",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_raw(input_vid: &Path, output_audio: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vn",
            "-f",
            "s16le",
            "-c:a",
            "pcm_s16le",
            output_audio.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_default(input_vid: &Path, output_vid: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-c:v",
            "copy",
            "-c:a",
            "copy",
            "-movflags",
            "+faststart",
            output_vid.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn save_video_to_avi(input_vid: &Path, output_vid: &Path) -> bool {
    Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-c:v",
            "mpeg4",
            "-q:v",
            "3",
            "-c:a",
            "mp3",
            output_vid.to_str().unwrap(),
        ])
        .status()
        .is_ok()
}

pub fn trim_vid(
    tempdir: PathBuf,
    input_vid: &Path,
    start: f64,
    end: f64,
    pipeline_step: usize,
) -> Result<PathBuf, String> {
    let duration = end - start;

    let output_path = tempdir.join(format!("trim_{}.mkv", pipeline_step));

    let output = match Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-ss",
            &start.to_string(),
            "-t",
            &duration.to_string(),
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-preset",
            "veryfast",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap(),
        ])
        .output()
    {
        Ok(r) => r,
        Err(e) => return Err(format!("Could not start ffmpeg during trim: {:?}", e)),
    };

    if output.status.success() {
        Ok(output_path)
    } else {
        Err("ffmpeg ran during trim, but could not process. Invalid input.".to_string())
    }
}

pub fn crop_vid(
    tempdir: PathBuf,
    input_vid: &Path,
    x_left: u32,
    y_top: u32,
    width: u32,
    height: u32,
    pipeline_step: usize,
) -> Result<PathBuf, String> {
    let output_path = tempdir.join(format!("crop_{}.mkv", pipeline_step));

    let crop_filter = format!("crop={}:{}:{}:{}", width, height, x_left, y_top);

    let output = match Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vf",
            &crop_filter,
            // video encode (required for filter accuracy)
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            // keep audio lossless copy
            "-c:a",
            "copy",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap(),
        ])
        .output()
    {
        Ok(r) => r,
        Err(e) => return Err(format!("Could not start ffmpeg during crop: {:?}", e)),
    };

    if output.status.success() {
        Ok(output_path)
    } else {
        Err("ffmpeg ran during crop, but could not process. Invalid input.".to_string())
    }
}

pub fn scale_vid_size(
    tempdir: PathBuf,
    input_vid: &Path,
    new_width: u32,
    new_height: u32,
    pipeline_step: usize,
) -> Result<PathBuf, String> {
    let output_path = tempdir.join(format!("scale_{}.mkv", pipeline_step));

    let even_width = new_width & !1; // TODO this is hacky. Not reported to user.
    let even_height = new_height & !1;

    let scale_filter = format!("scale={}:{}", even_width, even_height);

    let output = match Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vf",
            &scale_filter,
            // video encode (required for scaling)
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            // keep audio lossless copy
            "-c:a",
            "copy",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap(),
        ])
        .output()
    {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "Could not start ffmpeg during absolute scale: {:?}",
                e
            ));
        }
    };

    if output.status.success() {
        Ok(output_path)
    } else {
        Err(
            "ffmpeg ran during scale by abs size, but could not process. Invalid input."
                .to_string(),
        )
    }
}

pub fn scale_vid_prop(
    tempdir: PathBuf,
    input_vid: &Path,
    scale_factor: f32,
    pipeline_step: usize,
) -> Result<PathBuf, String> {
    let output_path = tempdir.join(format!("scale_{}.mkv", pipeline_step));

    let scale_filter = format!("scale=trunc(iw*{0}/2)*2:trunc(ih*{0}/2)*2", scale_factor);

    let output = match Command::new("ffmpeg")
        .args([
            "-i",
            input_vid.to_str().unwrap(),
            "-vf",
            &scale_filter,
            // video encode (required for scaling)
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            // keep audio copy
            "-c:a",
            "copy",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap(),
        ])
        .output()
    {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "Could not start ffmpeg during proportional scale: {:?}",
                e
            ));
        }
    };

    if output.status.success() {
        Ok(output_path)
    } else {
        Err(
            "ffmpeg ran during scale by proportion, but could not process. Invalid input."
                .to_string(),
        )
    }
}
