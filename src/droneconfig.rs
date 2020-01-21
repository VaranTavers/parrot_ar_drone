use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};

pub struct DroneConfig {
    pub config_data: Vec<(String, String)>,
    pub config_sending: bool,
    pub session_id: String,
    pub user_id: String,
    pub application_id: String,
    pub send_config_save_mode: bool,
    command_sender: Option<Sender<String>>,
    result_receiver: Option<Receiver<Option<String>>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

pub fn get_default_settings() -> DroneConfig {
    return DroneConfig {
        config_data: Vec::new(),
        config_sending: false,
        session_id: String::from("03016321"),
        user_id: String::from("0a100407"),
        application_id: String::from("03016321"),
        send_config_save_mode: false,
        command_sender: None,
        result_receiver: None,
        join_handle: None,
    };
}

fn get_config_thread(op_stream: Option<TcpStream>,
                     command_receiver: Receiver<String>,
                     result_sender: Sender<Option<String>>) {
    let mut stream = op_stream.unwrap();
    stream.set_nonblocking(true).unwrap();
    let mut options: HashMap<String, String> = HashMap::new();

    loop {
        match command_receiver.try_recv() {
            Ok(option_name) => {
                if option_name == "exit" {
                    break;
                }
                result_sender.send(options.get(&option_name)
                                   .map(|s_ref| s_ref.clone())).unwrap();
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
        let mut buffer = String::new();
        // It's probably a bad idea to use read_to_string because it can hang
        // if stream has no end!!!
        let read_result = stream.read_to_string(&mut buffer);
        match read_result {
            Ok(_) => {
                let commands = buffer.split("\n")
                    .map(|s| String::from(s))
                    .collect::<Vec<String>>();
                for s in commands {
                    let parts = s.split("=")
                        .map(|s| String::from(s))
                        .collect::<Vec<String>>();
                    options.insert(parts[0].clone(), parts[1].clone());
                }
            }
            Err(_) => {}
        }
    }
}

impl DroneConfig {
    pub fn get_config(self, name: String) -> Option<String> {
        self.command_sender.unwrap().send(name).unwrap();
        match self.result_receiver.unwrap().recv() {
            Ok(res) => { res }
            _ => { None }
        }
    }

    pub fn get_config_str(self, name: &str) -> Option<String> {
        self.get_config(String::from(name))
    }

    pub fn start_config_listening_thread(&mut self, tcp_stream: TcpStream) {
        let (c_s, c_r) = mpsc::channel();
        let (r_s, r_r) = mpsc::channel();
        self.command_sender = Some(c_s);
        self.result_receiver = Some(r_r);
        self.join_handle = Some(thread::spawn(move || {
            get_config_thread(Some(tcp_stream),
                              c_r,
                              r_s);
        }));
    }

    pub fn stop_config_listening_thread(mut self) {
        self.command_sender.unwrap().send(String::from("exit")).unwrap();
        self.command_sender = None;
        self.result_receiver = None;
        self.join_handle.take().unwrap().join().unwrap();
        self.join_handle = None;
    }
}
