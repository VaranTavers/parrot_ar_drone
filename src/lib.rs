mod communication;
mod video;
mod navdata;
mod droneconfig;
mod internal_config;

pub struct Drone {
    communication: communication::Communication,
    navdata: navdata::NavData,
    video: video::Video,
    config: droneconfig::DroneConfig,
    i_config: internal_config::InternalConfig,

}

pub fn get_drone() -> Drone {
    return Drone {
        communication: communication::get_default_settings(),
        video: video::get_default_settings(),
        navdata: navdata::get_default_settings(),
        config: droneconfig::get_default_settings(),
        i_config: internal_config::get_default_settings(),
    }
}

impl Drone {
    pub fn startup(&mut self) -> Result<(), String> {
        if !self.communication.try_connection() {
            return Err(String::from("Drone is not online!"));
        }
        match self.communication.start_connection() {
            Ok(()) => { return Ok(()); }
            Err(s) => { return Err(s); }
        }
    }

    pub fn test_led(&mut self) {
        self.communication.command("LED", vec![String::from("2"), String::from("1065353216"), String::from("10")]);
    }

    pub fn shutdown(self) {
        self.communication.shutdown_connection();
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{time, thread};

    #[test]
    fn test_connect() {
        let mut drone = get_drone();
        let test_result = drone.startup();
        match test_result {
            Ok(()) => {
                thread::sleep(time::Duration::from_secs(2));
                drone.test_led();
                thread::sleep(time::Duration::from_secs(5));
                drone.shutdown();
            }
            _ => {}
        }
        assert_eq!(test_result, Ok(()));
    }
}
