mod communication;
mod video;
mod navdata;
mod droneconfig;
mod internal_config;
mod format;

use format::*;

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
        match self.communication.start_connection(&self.i_config.show_commands) {
            Ok(()) => { return Ok(()); }
            Err(s) => { return Err(s); }
        }
    }

    pub fn shutdown(self) {
        self.communication.shutdown_connection();
    }

    /// Tells the drone that it is horizontal (parallel to the ground)
    /// Do this only when the drone is on the ground!
    pub fn trim(&mut self) {
        self.communication.command("FTRIM", Vec::new());
    }

    pub fn mtrim(&mut self) {
        self.communication.command("CALIB", vec![String::from("0")]);
    }

    pub fn mantrim(&mut self, theta: f32, phi: f32, yaw: f32) {
        self.communication.command(
            "MTRIM", 
            vec![format_float(theta),
            format_float(phi),
            format_float(yaw)]
        );
    }

    /// Sorry, move was a keyword so I couldn't set it as a method name
    /// Parameters: Speed from left ([-1.0, 0.0)) to right ((0.0, 1.0]) or none (0.0)
    /// Speed from back ([-1.0, 0.0)) to front ((0.0, 1.0]) or none (0.0)
    /// Speed from down ([-1.0, 0.0)) to up ((0.0, 1.0]) or none (0.0)
    /// Turn rate from left ([-1.0, 0.0)) to right ((0.0, 1.0]) or none (0.0)
    pub fn mov(&mut self, left_right: f32, back_front: f32, down_up: f32, turn_left_right: f32) {
        let mut l_r = left_right;
        let mut b_f = back_front;
        let mut d_u = down_up;
        let mut t_l_r = turn_left_right;
        if left_right.abs() > 1.0 {
            l_r = left_right / left_right.abs();
        }
        if back_front.abs() > 1.0 {
            b_f = back_front / back_front.abs();
        }
        if down_up.abs() > 1.0 {
            d_u = down_up / down_up.abs();
        }
        if t_l_r.abs() > 1.0 {
            t_l_r = turn_left_right / turn_left_right.abs();
        }

        self.communication.command("PCMD",
                                   vec![
                                   format_int(3),
                                   format_float(l_r),
                                   format_float(-b_f),
                                   format_float(d_u),
                                   format_float(t_l_r)
                                   ]);
    }
    
    /// Move relative to the controller
    pub fn rel_mov(&mut self, left_right: f32, back_front: f32, down_up: f32, turn_left_right: f32, east_west: f32, north_ta_accuracy: f32) {
        let mut l_r = left_right;
        let mut b_f = back_front;
        let mut d_u = down_up;
        let mut t_l_r = turn_left_right;
        let mut e_w = east_west;
        let mut n_ta_a = north_ta_accuracy;
        if left_right.abs() > 1.0 {
            l_r = left_right / left_right.abs();
        }
        if back_front.abs() > 1.0 {
            b_f = back_front / back_front.abs();
        }
        if down_up.abs() > 1.0 {
            d_u = down_up / down_up.abs();
        }
        if turn_left_right.abs() > 1.0 {
            t_l_r = turn_left_right / turn_left_right.abs();
        }
        if east_west.abs() > 1.0 {
            e_w = east_west / east_west.abs();
        }
        if north_ta_accuracy.abs() > 1.0 {
            n_ta_a = north_ta_accuracy / north_ta_accuracy.abs();
        }

        self.communication.command("PCMD_MAG",
                                   vec![
                                   format_int(1),
                                   format_float(l_r),
                                   format_float(-b_f),
                                   format_float(d_u),
                                   format_float(t_l_r),
                                   format_float(e_w),
                                   format_float(n_ta_a)
                                   ]);
    }

    /// Stops all movement and turns
    pub fn hover(&mut self) {
        self.mov(0.0, 0.0, 0.0, 0.0);
    }
    
    /// Same a hover
    pub fn stop(&mut self) {
        self.hover();
    }

    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_right()
    pub fn mov_right(&mut self, speed: f32) {
        self.mov(speed, 0.0, 0.0, 0.0);
    }

    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_left()
    pub fn mov_left(&mut self, speed: f32) {
        self.mov(-speed, 0.0, 0.0, 0.0);
    }
    
    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_forward()
    pub fn mov_forward(&mut self, speed: f32) {
        self.mov(0.0, speed, 0.0, 0.0);
    }
    
    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_backward()
    pub fn mov_backward(&mut self, speed: f32) {
        self.mov(0.0, -speed, 0.0, 0.0);
    }
    
    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_up()
    pub fn mov_up(&mut self, speed: f32) {
        self.mov(0.0, 0.0, speed, 0.0);
    }
    
    /// This method requires explicit speed to be given to it from [-1.0, 1.0]
    /// If you want to use the drones default speed use move_down()
    pub fn mov_down(&mut self, speed: f32) {
        self.mov(0.0, 0.0, -speed, 0.0);
    }
    
    /// This method uses the drones default speed
    /// If you want to give an explicit speed use move_right()
    pub fn move_right(&mut self) {
        self.mov(self.i_config.speed, 0.0, 0.0, 0.0);
    }

    /// This method uses the drones default speed
    /// If you want to give an explicit speed use mov_left()
    pub fn move_left(&mut self) {
        self.mov(-self.i_config.speed, 0.0, 0.0, 0.0);
    }    

    /// This method uses the drones default speed
    /// If you want to give an explicit speed use mov_left()
    pub fn move_forward(&mut self) {
        self.mov(0.0, self.i_config.speed, 0.0, 0.0);
    }
    
    /// This method uses the drones default speed
    /// If you want to give an explicit speed use mov_left()
    pub fn move_backward(&mut self) {
        self.mov(0.0, -self.i_config.speed, 0.0, 0.0);
    }
    
    /// This method uses the drones default speed
    /// If you want to give an explicit speed use mov_left()
    pub fn move_up(&mut self) {
        self.mov(0.0, 0.0, self.i_config.speed, 0.0);
    }
    
    /// This method uses the drones default speed
    /// If you want to give an explicit speed use mov_left()
    pub fn move_down(&mut self) {
        self.mov(0.0, 0.0, -self.i_config.speed, 0.0);
    }

    /// This method requires explicit turn rate to be given to it from [-1.0, 1.0]
    pub fn turn_right(&mut self, turn_rate: f32) {
        self.mov(0.0, 0.0, 0.0, turn_rate);
    }

    /// This method requires explicit turn rate to be given to it from [-1.0, 1.0]
    pub fn turn_left(&mut self, turn_rate: f32) {
        self.mov(0.0, 0.0, 0.0, -turn_rate);
    }

    /// Message conforms SDK documentation
    /// 290718208=10001010101000000001000000000
    pub fn takeoff(&mut self) {
        self.communication.command("REF", vec![String::from("290718208")]);
    }

    /// Message conforms SDK documentation
    /// 290717696=10001010101000000000000000000
    pub fn land(&mut self) {
        self.communication.command("REF", vec![String::from("290717696")]);
    }

    /// Do a preset led animation (anim < 21; duration in seconds)
    pub fn led(&mut self, anim: usize, frequency: f32, duration: i32) {
        if anim < 21 && frequency > 0.0 && duration > 0 {
            self.communication.command("LED", vec![
                                       format_int(anim as i32),
                                       format_float(frequency),
                                       format_int(duration)]);
        }
    }
    
    /// Execute a preset movement (anim < 20; duration in seconds)
    pub fn anim(&mut self, anim: usize, duration: i32) {
        if anim < 20 && duration > 0 {
            self.communication.command("ANIM", vec![
                                       format_int(anim as i32),
                                       format_int(duration)]);
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        if speed.abs() > 1.0 {
            self.i_config.speed = 1.0;
        } else {
            self.i_config.speed = speed.abs();
        }
    }
    
    pub fn get_config(&mut self) {
        self.communication.command_str("CTRL", vec!["5", "0"]);
        self.communication.command_str("CTRL", vec!["4", "0"]);
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
                drone.takeoff();
                thread::sleep(time::Duration::from_secs(5));
                drone.land();
                thread::sleep(time::Duration::from_secs(5));
                drone.shutdown();
            }
            _ => {}
        }
        assert_eq!(test_result, Ok(()));
    }
}
