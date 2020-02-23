mod communication;
mod navdata;
mod droneconfig;
mod internal_config;
mod format;

pub use format::*;
pub use communication::*;
pub use internal_config::*;

/// First is the codec used for streaming on UDP 5555, second (if exists) is for
/// recording on TCP 5553.
pub enum VideoCodec {
    MP4_360p,
    H264_360p,
    MP4_360pH264_720p,
    MP4_360pH264_360p,
    H264_720p,
}

pub struct Drone {
    communication: communication::Communication,
    navdata: navdata::NavData,
    config: droneconfig::DroneConfig,
    i_config: internal_config::InternalConfig,
}

impl Drone {
    /// Returns a Drone object with default settings.
    pub fn new() -> Drone {
        Drone {
            communication: communication::Communication::new(),
            navdata: navdata::NavData::new(),
            config: droneconfig::DroneConfig::new(),
            i_config: internal_config::InternalConfig::new(),
        }

    }

    /// Initializes connection to the drone, starts navdata, control, and config
    /// threads. Sends basic commands to the drone to initialize it.
    pub fn startup(&mut self) -> Result<(), String> {
        if !self.communication.try_connection() {
            return Err(String::from("Drone is not online!"));
        }
        match self.communication.start_connection(&self.i_config.show_commands) {
            Ok(()) => { }
            Err(s) => { return Err(s); }
        }
        match self.communication.get_ctl_tcp_connection() {
            Ok(stream) => { self.config.start_config_listening_thread(stream); }
            Err(s) => { return Err(s); }
        }

        match self.communication.get_navdata_udp_connection() {
            Ok(stream) => { self.navdata.start_navdata_listening_thread(stream, self.i_config.debug)}
            Err(s) => { return Err(s); }
        }

        // Is necessary in order to get full NavData back
        self.use_demo_mode(true);
        self.communication.command_str("CTRL", vec!["5", "0"]);
        self.set_config_str("custom:session_id", "-all");

        self.communication.command_str("CTRL", vec!["5", "0"]);
        self.update_config();

        Ok(())
    }

    fn shutdown(&mut self) {
        self.navdata.stop_navdata_listening_thread();
        self.config.stop_config_listening_thread();
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

    /// The most basic move command.
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

    /// Same as hover
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

    /// Makes the drone take off
    /// Message conforms SDK documentation
    /// 290718208=10001010101000000001000000000
    pub fn takeoff(&mut self) {
        self.communication.command("REF", vec![String::from("290718208")]);
    }

    /// Makes the drone land
    /// Message conforms SDK documentation
    /// 290717696=10001010101000000000000000000
    pub fn land(&mut self) {
        self.communication.command("REF", vec![String::from("290717696")]);
    }

    /// Resets the drone in case the last landing was crashlanding.
    /// Message conforms SDK documentation
    /// 290717952=10001010101000000000100000000
    pub fn reset(&mut self) {
        self.communication.command("REF", vec![String::from("290717952")]);
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

    /// Control engines thrust manually (could be potentially dangerous)
    ///
    /// Parameters in order are: front-left, front-right, rear-left, rear-right
    ///
    /// All values should be between in [0, 1023], use at
    pub fn manual_engine(&mut self, fl: u32, fr: u32, rl: u32, rr: u32) {
        let mut fl = fl; 
        if fl > 1023 {
            fl = 1023;
        }
        let mut fr = fr; 
        if fr > 1023 {
            fr = 1023;
        }
        let mut rl = rl; 
        if rl > 1023 {
            rl = 1023;
        }
        let mut rr = rr; 
        if rr > 1023 {
            rr = 1023;
        }

        self.communication.command("PWM", vec![
                                   format_int(fl as i32),
                                   format_int(fr as i32),
                                   format_int(rl as i32),
                                   format_int(rr as i32)
        ])
    }

    /// This makes the drone fly around and follow 2D tags detected by it's camera
    pub fn aflight(&mut self, flag: bool) {
        if flag {
            self.communication.command_str("AFLIGHT", vec!["1"]);
        } else {
            self.communication.command_str("AFLIGHT", vec!["0"]);
        }
    }

    /// Set the default seed of the drone that will be used in the move functions.
    /// This value should be in the [0, 1.0] range 
    pub fn set_speed(&mut self, speed: f32) {
        if speed.abs() > 1.0 {
            self.i_config.speed = 1.0;
        } else {
            self.i_config.speed = speed.abs();
        }
    }

    /// Requests an updated config from the drone
    pub fn update_config(&mut self) {
        self.communication.command_str("CTRL", vec!["5", "0"]);
        self.communication.command_str("CTRL", vec!["4", "0"]);
    }

    /// This function doesn't guarantee that the config read is up to date!
    /// To be sure please use the update_config function before this and
    /// wait a little, to give time to the config thread to process the changes
    pub fn get_offline_config(&mut self, config_name: &str) -> Option<String> {
        self.config.get_config_str(config_name)
    }

    pub fn send_config_ids(&mut self) {
        self.communication.command("CONFIG_IDS",
                                   vec![
                                   format_string(self.config.session_id.clone()),
                                   format_string(self.config.user_id.clone()),
                                   format_string(self.config.application_id.clone()),
                                   ]);
    }

    /// This function sends a config to the drone, however it does not check if
    /// the drone has gotten the command or not.
    pub fn set_config(&mut self, config_name: &str, config_value: String) {
        // self.send_config_ids();
        self.communication.command("CONFIG",
                                   vec![
                                   format!("\"{}\"", config_name),
                                   format!("\"{}\"", config_value)
                                   ]);
    }

    /// Same as set_config but this uses &str for config_value
    pub fn set_config_str(&mut self, config_name: &str, config_value: &str) {
        // self.send_config_ids();
        self.communication.command("CONFIG",
                                   vec![
                                   format!("\"{}\"", config_name),
                                   format!("\"{}\"", config_value)
                                   ]);
    }

    /// Enters the drone into demo mode
    pub fn use_demo_mode(&mut self, value: bool) {
        if value {
            self.set_config_str("general:navdata_demo", "TRUE");
        } else {
            self.set_config_str("general:navdata_demo", "FALSE");
        }
    }

    /// Sets the codec that will be used by the drone for streaming and recording.
    pub fn set_video_codec(&mut self, codec: VideoCodec) {
        let s;
        match codec { 
            VideoCodec::MP4_360p => {
                s = "128";
            }
            VideoCodec::H264_360p => {
                s = "129";
            }
            VideoCodec::H264_720p => {
                s = "131";
            }
            VideoCodec::MP4_360pH264_720p => {
                s = "130";
            }
            VideoCodec::MP4_360pH264_360p => {
                s = "136";
            }
        }

        self.set_config_str("video:video_codec", s);
    }

    /// Stream (UDP 5555) will be in HD (H264_720p) and there will be nothing
    /// sent to the recording port (TCP 5553)
    pub fn set_hd_video_stream(&mut self) {
        self.set_video_codec(VideoCodec::H264_720p);
    }

    /// Stream (UDP 5555) will be in SD (H264_360p) and there will be nothing
    /// sent to the recording port (TCP 5553)
    pub fn set_sd_video_stream(&mut self) {
        self.set_video_codec(VideoCodec::H264_360p);
    }

    /// Stream (UDP 5555) will be in SD (MP4_360p) and there will be nothing
    /// sent to the recording port (TCP 5553)
    pub fn set_mp4_video_stream(&mut self) {
        self.set_video_codec(VideoCodec::MP4_360p);
    }

    /// Stream (UDP 5555) will be in SD (MP4_360p) and a HD (H264_720p) capture 
    /// will be sent to the recording port (TCP 5553)
    pub fn set_hd_video_capture(&mut self) {
        self.set_video_codec(VideoCodec::MP4_360pH264_720p);
    }

    /// Stream (UDP 5555) will be in SD (MP4_360p) and a SD (H264_360p) capture 
    /// will be sent to the recording port (TCP 5553)
    pub fn set_sd_video_capture(&mut self) {
        self.set_video_codec(VideoCodec::MP4_360pH264_360p);
    }

    /// Set the FPS for the video on (UDP 5555)
    pub fn set_video_fps(&mut self, fps: u32) {
        let mut real_fps = fps;
        if fps > 60 || fps == 0 {
            real_fps = 60;
        }
        self.set_config("video:codec_fps", format!("{}", real_fps));
    }

    /// Set the FPS for the video on (UDP 5555)
    pub fn set_video_bitrate(&mut self, bitrate: u32) {
        let mut real_bitrate = bitrate;
        if bitrate > 20000 {
            real_bitrate = 20000;
        } else if bitrate < 250 {
            real_bitrate = 250;
        }
        self.set_config("video:bitrate", format!("{}", real_bitrate));
    }

    /// Tells the drone to use it's front cam, for recording and streaming
    pub fn use_front_cam(&mut self) {
        self.set_config_str("video:video_channel", "0");
    }

    /// Tells the drone to use it's ground cam, for recording and streaming
    pub fn use_ground_cam(&mut self) {
        self.set_config_str("video:video_channel", "1");
    }

    /// Get Navdata from the drone (currently only supports DEMO mode)
    pub fn get_navdata(&mut self, name: &str) -> Option<navdata::NavDataValue> {
        self.navdata.get_navdata_str(name)
    }
}

impl Drop for Drone {
    fn drop(&mut self) {
        self.shutdown();
    }
}
