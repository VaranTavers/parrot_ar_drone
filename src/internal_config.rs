use std::time::{SystemTime};

pub struct InternalConfig {
    pub version: String,
    pub start_time: SystemTime,
    /// Shows all sent commands except keepalives
    pub show_commands: bool,
    /// Shows additional debug information
    pub debug: bool,
    /// Should the drone land when there is a communication problem
    pub stop_on_com_loss: bool,
    /// Default drone speed in percent
    pub speed: f32,
    pub value_correction: bool,
    pub self_rotation: f32,
    pub navdata_process: String,
    pub video_process: String,
    pub v_decode_process: String,
    pub network_suicide: bool,
    pub recieve_data_running: bool,
    pub send_config_running: bool,
    pub shutdown: bool
}

impl InternalConfig {
    /// Returns a default set of internal configs
    pub fn new() -> InternalConfig {
        return InternalConfig {
            version: String::from("0.0.1 (2.1.4)"),
            start_time: SystemTime::now(),
            show_commands: true,
            debug: true,
            stop_on_com_loss: false,
            speed: 0.2,
            value_correction: false,
            self_rotation: 0.0185,
            navdata_process: String::new(),
            video_process: String::new(),
            v_decode_process: String::new(),
            network_suicide: false,
            recieve_data_running: false,
            send_config_running: false,
            shutdown: false
        }
    }
}
