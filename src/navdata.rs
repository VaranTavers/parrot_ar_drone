pub struct NavData {
    pub navdata: String,
    pub state: Vec<i32>,
    pub navdata_count: usize,
    pub navdata_timestamp: u32,
    pub navdata_decoding_time: f64,
    pub no_navdata: bool
}

pub fn get_default_settings() -> NavData {
    return NavData {
        navdata: String::new(),
        state: vec![32; 0],
        navdata_count: 0,
        navdata_timestamp: 0,
        navdata_decoding_time: 0.0,
        no_navdata: false
    };
}
