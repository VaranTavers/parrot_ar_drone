use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::{time, thread};

pub struct Communication {
    pub drone_ip: String,
    pub nav_data_port: u32,
    pub video_port: u32,
    pub cmd_port: u32,
    pub ctl_port: u32,
    command_stream: Option<TcpStream>,
    command_list: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    connection_thread: Option<thread::JoinHandle<()>>,
}


pub fn get_default_settings() -> Communication {
    return Communication {
        drone_ip: String::from("192.168.1.1"),
        nav_data_port: 5554,
        video_port: 5555,
        cmd_port: 5556,
        ctl_port: 5559,
        command_stream: None,
        command_list: Arc::new(Mutex::new(Vec::new())),
        connection_thread: None,
    };
}

fn communication_thread(command_stream: Option<TcpStream>, 
                        command_list: Arc<Mutex<Vec<(String, Vec<String>)>>>) {
    let mut stream = command_stream.unwrap();
    let mut cmd_count = 0;
    loop {
        let mut cmd_list = command_list.lock().unwrap();
        let mut s = String::new();
        if cmd_list.len() > 0 {
            let (cmd_str, params) = cmd_list.pop().unwrap();
            s = format_command(cmd_count, cmd_str, params);
        } else {
            s = format_command(cmd_count, String::from("COMWDG"), Vec::new());
        }
        stream.write(s.as_bytes()); 
        cmd_count += 1;
        drop(cmd_list); // We release the lock
        thread::sleep(time::Duration::from_millis(100));
    }
}

impl Communication {
    pub fn try_connection(&self) -> bool {
        let stream = TcpStream::connect(format!("{}:{}", self.drone_ip, 21)); 

        match stream {
            Ok(_) => {
                return true;
            }

            Err(_) =>  {
                return false; 
            }
        }
    }

    pub fn command(&mut self, command: &str, params: Vec<String>) {
        let mut c_vec = self.command_list.lock().unwrap();
        c_vec.push((String::from(command), params));
        // When c_vec goes out of scope command_list gets unlocked
    }

    pub fn start_connection(&mut self) -> Result<(), String> {
        let connect = TcpStream::connect(format!("{}:{}", self.drone_ip, self.cmd_port));
        match connect {
            Ok(stream) => {
                self.connection_thread = Some(thread::spawn(|| {
                    communication_thread(Some(stream), Arc::new(Mutex::new(Vec::new())));
                }));
                return Ok(());
            }
            Err(_) => {
                return Err(String::from("Connection failed!"));
            }
        }
    }

}
