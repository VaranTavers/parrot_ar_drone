use std::net::TcpStream;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::{time, thread};
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};

pub struct Communication {
    pub drone_ip: String,
    pub nav_data_port: u32,
    pub video_port: u32,
    pub cmd_port: u32,
    pub ctl_port: u32,
    command_list: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    connection_thread: Option<thread::JoinHandle<()>>,
    thread_terminator: Option<Sender<i32>>
}


pub fn get_default_settings() -> Communication {
    return Communication {
        drone_ip: String::from("192.168.1.1"),
        nav_data_port: 5554,
        video_port: 5555,
        cmd_port: 5556,
        ctl_port: 5559,
        command_list: Arc::new(Mutex::new(Vec::new())),
        connection_thread: None,
        thread_terminator: None
    };
}

pub fn format_command(
    command_num: usize,
    command: String,
    params: Vec<String>) -> String {
    let command = params.iter().fold(format!("AT*{}={}", command, command_num), 
                                     |acc, value| format!("{},{}", acc, value));
    format!("{}\r", command)
}

fn communication_thread(socket: UdpSocket, 
                        command_list: Arc<Mutex<Vec<(String, Vec<String>)>>>,
                        receiver: Receiver<i32>,
                        address: String) {
    let mut cmd_count = 3;
    loop {
        match receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("Terminating.");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
        let mut cmd_list = command_list.lock().unwrap();
        let s;
        if cmd_list.len() > 0 {
            let (cmd_str, params) = cmd_list.pop().unwrap();
            s = format_command(cmd_count, cmd_str, params);
        } else {
            s = format_command(cmd_count, String::from("COMWDG"), Vec::new());
        }
        socket.send_to(s.as_bytes(), &address).unwrap(); 
        cmd_count += 1;
        drop(cmd_list); // We release the lock
        thread::sleep(time::Duration::from_millis(100));
    }
}

impl Communication {
    pub fn try_connection(&self) -> bool {
        let socket = TcpStream::connect(format!("{}:{}", self.drone_ip, 21)); 

        match socket {
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
        let socket = UdpSocket::bind("0.0.0.0:5556").expect("couldn't bind to address");
        socket.set_nonblocking(true).unwrap();
        let address = format!("{}:{}", self.drone_ip, self.cmd_port);

        // Creating a channel to kill the thread
        let (sender, receiver) = mpsc::channel();
        self.thread_terminator = Some(sender);

        // Sending first two commands
        let s = String::from("\r");
        socket.send_to(s.as_bytes(), &address).unwrap();
        thread::sleep(time::Duration::from_millis(10));
        let s = String::from("AT*PMODE=1,2\rAT*MISC=2,2,20,2000,3000\r");
        socket.send_to(s.as_bytes(), &address).unwrap();

        self.connection_thread = Some(thread::spawn(|| {
            communication_thread(socket,
                                 Arc::new(Mutex::new(Vec::new())),
                                 receiver,
                                 address);
        }));
        return Ok(());
    }

    pub fn shutdown_connection(mut self) {
        let sender = self.thread_terminator.unwrap();
        sender.send(1).unwrap();
        self.connection_thread.take().unwrap().join().unwrap();
        self.thread_terminator = None;
        self.connection_thread = None;
    }

}
