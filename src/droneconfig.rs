pub struct DroneConfig {
    pub config_data: Vec<i32>,
    pub config_data_count: usize,
    pub config_data_timestamp: u32,
    pub config_sending: bool,
    pub config_session_id: String,
    pub config_user_id: String,
    pub config_application_id: String,
    pub send_config_save_mode: bool,
    pub config_queue: Vec<(String, String, bool)>,
}

pub fn get_default_settings() -> DroneConfig {
    return DroneConfig {
        config_data: Vec::new(),
        config_data_count: 0,
        config_data_timestamp: 0,
        config_sending: false,
        config_session_id: String::from("03016321"),
        config_user_id: String::from("0a100407"),
        config_application_id: String::from("03016321"),
        send_config_save_mode: false,
        config_queue: Vec::new()
    };
}


impl DroneConfig {
    pub fn set_config(&mut self, name: &str, value: &str) {
        self.config_queue.push((String::from(name), String::from(value), true));
    }
}
