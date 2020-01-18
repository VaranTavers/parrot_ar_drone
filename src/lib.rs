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
    fn startup(&self) -> Result<(), String> {
        if !self.communication.try_connection() {
            return Err(String::from("Drone is not online!"));
        }
        // rest of startup
        Ok(())
    }

}
