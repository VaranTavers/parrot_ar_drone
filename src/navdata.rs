use std::collections::HashMap;
use std::net::UdpSocket;
use std::thread;
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

pub enum NavDataValue {
    Int(i32),
    Uint(u32),
    Float(f32),
    Bool(bool)
}

impl NavDataValue {
    pub fn copy(&self) -> NavDataValue {
        match self {
            NavDataValue::Int(a) => NavDataValue::Int(*a),
            NavDataValue::Uint(a) => NavDataValue::Uint(*a),
            NavDataValue::Float(a) => NavDataValue::Float(*a),
            NavDataValue::Bool(a) => NavDataValue::Bool(*a),
        }
    }
}

pub struct NavData {
    pub navdata: String,
    pub state: Vec<i32>,
    pub navdata_count: usize,
    pub navdata_timestamp: u32,
    pub navdata_decoding_time: f64,
    pub no_navdata: bool,
    command_sender: Option<Sender<String>>,
    result_receiver: Option<Receiver<Option<NavDataValue>>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

fn decode_id0<'a, I: AsRef<[u8]>>(crs: &mut Cursor<I>,
                                  options_map: &mut HashMap<String, NavDataValue>,
                                  print_error: bool) {
    let size = crs.read_u16::<LittleEndian>().unwrap(); 
    if size != 148 && print_error {
        println!("Navdata-Demo-Packet has wrong size: {}", size);
    }
    let flags = crs.read_u32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_default"), NavDataValue::Bool(flags >> 15 & 1 == 1));
    options_map.insert(String::from("demo_init"), NavDataValue::Bool(flags >> 16 & 1 == 1));
    options_map.insert(String::from("demo_landed"), NavDataValue::Bool(flags >> 17 & 1 == 1));
    options_map.insert(String::from("demo_flying"), NavDataValue::Bool(flags >> 18 & 1 == 1));
    options_map.insert(String::from("demo_hovering"), NavDataValue::Bool(flags >> 19 & 1 == 1));
    options_map.insert(String::from("demo_test"), NavDataValue::Bool(flags >> 20 & 1 == 1));
    options_map.insert(String::from("demo_trans_takeoff"), NavDataValue::Bool(flags >> 21 & 1 == 1));
    options_map.insert(String::from("demo_trans_gofix"), NavDataValue::Bool(flags >> 22 & 1 == 1));
    options_map.insert(String::from("demo_trans_landing"), NavDataValue::Bool(flags >> 23 & 1 == 1));
    options_map.insert(String::from("demo_trans_looping"), NavDataValue::Bool(flags >> 24 & 1 == 1));
    options_map.insert(String::from("demo_trans_no_vision"), NavDataValue::Bool(flags >> 25 & 1 == 1));
    options_map.insert(String::from("demo_num_state"),NavDataValue::Bool(flags >> 26 & 1 == 1));

    let battery = crs.read_u32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_battery"),NavDataValue::Uint(battery));

    let theta = crs.read_f32::<LittleEndian>().unwrap();
    let phi = crs.read_f32::<LittleEndian>().unwrap();
    let psi = crs.read_f32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_theta"),NavDataValue::Float(theta));
    options_map.insert(String::from("demo_phi"),NavDataValue::Float(phi));
    options_map.insert(String::from("demo_psi"),NavDataValue::Float(psi));

    let altitude = crs.read_i32::<LittleEndian>().unwrap() / 10;
    options_map.insert(String::from("demo_altitude"),NavDataValue::Int(altitude));

    let vx = crs.read_f32::<LittleEndian>().unwrap();
    let vy = crs.read_f32::<LittleEndian>().unwrap();
    let vz = crs.read_f32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_vx"),NavDataValue::Float(vx));
    options_map.insert(String::from("demo_vy"),NavDataValue::Float(vy));
    options_map.insert(String::from("demo_vz"),NavDataValue::Float(vz));

    let num_frames = crs.read_u32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_num_frames"),NavDataValue::Uint(num_frames));

    for i in 0..9 {
        let val = crs.read_f32::<LittleEndian>().unwrap();
        options_map.insert(format!("demo_det_cam_rot_{}", i),NavDataValue::Float(val));
    }
    for i in 0..3 {
        let val = crs.read_f32::<LittleEndian>().unwrap();
        options_map.insert(format!("demo_det_cam_trans_{}", i),NavDataValue::Float(val));
    }
    let det_tag_index = crs.read_u32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_detection_tag_index"),NavDataValue::Uint(det_tag_index));

    let det_tag_type = crs.read_u32::<LittleEndian>().unwrap();
    options_map.insert(String::from("demo_detection_tag_type"),NavDataValue::Uint(det_tag_type));

    for i in 0..9 {
        let val = crs.read_f32::<LittleEndian>().unwrap();
        options_map.insert(format!("demo_cam_rot_{}", i),NavDataValue::Float(val));
    }
    for i in 0..3 {
        let val = crs.read_f32::<LittleEndian>().unwrap();
        options_map.insert(format!("demo_cam_trans_{}", i),NavDataValue::Float(val));
    }
}

fn get_navdata_thread(op_stream: Option<UdpSocket>,
                      print_error: bool,
                      command_receiver: Receiver<String>,
                      result_sender: Sender<Option<NavDataValue>>) {
    let stream = op_stream.unwrap();
    let mut options: HashMap<String, NavDataValue> = HashMap::new();

    let tmp = [1 as u8, 0 as u8, 0 as u8, 0 as u8];
    let mut seq_num = 0;
    stream.send(&tmp).unwrap();
    loop {
        match command_receiver.try_recv() {
            Ok(option_name) => {
                if option_name == "exit" {
                    break;
                }
                result_sender.send(options.get(&option_name)
                                   .map(|a| a.copy())).unwrap();
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
        let mut buff = [0; 65535];
        match stream.recv(&mut buff) {
            Ok(_) => {
                let mut crs = Cursor::new(buff.iter());
                let header = crs.read_u32::<LittleEndian>().unwrap();
                let drone_state = crs.read_u32::<LittleEndian>().unwrap();
                let packet_seq = crs.read_u32::<LittleEndian>().unwrap();
                let vision_flag = crs.read_u32::<LittleEndian>().unwrap();

                if packet_seq > seq_num {
                    seq_num = packet_seq;
                    options.insert(String::from("header_header"), NavDataValue::Uint(header));
                    options.insert(String::from("header_seq_num"), NavDataValue::Uint(seq_num));
                    options.insert(String::from("header_drone_state"), NavDataValue::Uint(drone_state));
                    options.insert(String::from("header_flag"), NavDataValue::Uint(vision_flag));
                    loop {
                        let id = crs.read_u16::<LittleEndian>().unwrap();
                        if id == 0 {
                            decode_id0(&mut crs, &mut options, print_error);
                        } else if id != 65535 { // Checksum packet
                            let mut size = crs.read_u32::<LittleEndian>().unwrap();
                            while size > 32 {
                                let _tmp = crs.read_u32::<LittleEndian>().unwrap();
                                size -= 32;
                            }
                            while size > 8 {
                                let _tmp = crs.read_u8().unwrap();
                                size -= 8;
                            }
                            break;
                        }

                    }
                }
            }
            _ => {

            }
        }
    }
}

impl NavData {
    /// Returns a NavData object with default settings
    pub fn new() -> NavData {
        return NavData {
            navdata: String::new(),
            state: vec![32; 0],
            navdata_count: 0,
            navdata_timestamp: 0,
            navdata_decoding_time: 0.0,
            no_navdata: false,
            command_sender: None,
            result_receiver: None,
            join_handle: None,
        };
    }


    pub fn get_navdata(&mut self, name: String) -> Option<NavDataValue> {
        let cmd_sender = self.command_sender.take().unwrap();
        cmd_sender.send(name).unwrap();
        self.command_sender.replace(cmd_sender);

        let res_rec = self.result_receiver.take().unwrap();
        let recv_result = res_rec.recv();
        self.result_receiver.replace(res_rec);
        match recv_result {
            Ok(res) => { res }
            _ => { None }
        }
    }

    pub fn get_navdata_str(&mut self, name: &str) -> Option<NavDataValue> {
        self.get_navdata(String::from(name))
    }

    pub fn start_navdata_listening_thread(&mut self,
                                          tcp_stream: UdpSocket,
                                          print_error: bool) {
        let (c_s, c_r) = mpsc::channel();
        let (r_s, r_r) = mpsc::channel();
        self.command_sender = Some(c_s);
        self.result_receiver = Some(r_r);
        self.join_handle = Some(thread::spawn(move || {
            get_navdata_thread(Some(tcp_stream),
            print_error,
            c_r,
            r_s);
        }));
    }

    pub fn stop_navdata_listening_thread(&mut self) {
        self.command_sender.take().unwrap().send(String::from("exit")).unwrap();
        self.result_receiver.take().unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}
