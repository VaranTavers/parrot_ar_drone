use opencv::core;

pub struct Video {
    pub video_image: Option<core::Mat>,
    pub video_image_count: usize,
    pub video_decode_timestamp: u32,
    pub video_decode_time: i32,
    pub video_ready: bool,
    pub v_key: char,
    pub save_video: bool
}
 
pub fn get_default_settings() -> Video {
    return Video {
        video_image: None,
        video_image_count: 0,
        video_decode_timestamp: 0,
        video_decode_time: 0,
        video_ready: false,
        v_key: ' ',
        save_video: false
    };
}
