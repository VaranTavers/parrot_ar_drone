use std::net::TcpStream;
use std::net::UdpSocket;
use std::{time, thread};
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};

/// Component that is responsible for the communication between the drone and
/// this API.
pub struct Communication {
    /// The drones IP (default is 192.168.1.1)
    pub drone_ip: String,
    /// UDP port (default 5554) from which we receive Navigation Data
    pub nav_data_port: u32,
    /// UDP port (default 5555) from which we receive the video image.
    /// It should be readable by OpenCV without any fancy processing
    pub video_port: u32,
    /// UDP port (default 5556) to which we send commands
    pub cmd_port: u32,
    /// TCP port (default 5559) from which we get config information
    pub ctl_port: u32,
    connection_thread: Option<thread::JoinHandle<()>>,
    command_channel: Option<Sender<(String, Vec<String>)>>
}

/// Get a Communication struct with default settings, if you didn't do
/// any fancy shenanigans with the drones settings, this should be enough
/// for you.
pub fn get_default_settings() -> Communication {
    return Communication {
        drone_ip: String::from("192.168.1.1"),
        nav_data_port: 5554,
        video_port: 5555,
        cmd_port: 5556,
        ctl_port: 5559,
        connection_thread: None,
        command_channel: None
    };
}

fn format_command(
    command_num: usize,
    command: String,
    params: Vec<String>) -> String {
    let command = params.iter().fold(format!("AT*{}={}", command, command_num), 
                                     |acc, value| format!("{},{}", acc, value));
    format!("{}\r", command)
}

fn communication_thread(socket: UdpSocket, 
                        receiver: Receiver<(String, Vec<String>)>,
                        address: String,
                        echo_commands: bool) {
    let mut cmd_count = 3;
    let mut wait_count = 0;
    loop {
        match receiver.try_recv() {
            Ok((cmd_str, params)) => {
                if cmd_str == "exit" {
                    break;
                }
                let s = format_command(cmd_count, cmd_str, params);
                if echo_commands {
                    println!("{}", s);
                }
                socket.send_to(s.as_bytes(), &address).unwrap(); 
                cmd_count += 1;
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {
                wait_count += 1;
                if wait_count == 4 {
                    let s = format_command(cmd_count, String::from("COMWDG"), Vec::new());
                    socket.send_to(s.as_bytes(), &address).unwrap(); 
                    cmd_count += 1;
                    wait_count = 0;
                }
            }
        }
        thread::sleep(time::Duration::from_millis(50));
    }
}

impl Communication {
    /// Tries connecting to the drone (may hang on routers which have the
    /// drones ip (default 192.168.1.1)
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

    /// Pushes a command to the send queue, every 50 ms a command is sent, if
    /// no command is in queue then every 200 ms a keepalive command is sent.
    pub fn command(&mut self, command: &str, params: Vec<String>) {
        match &self.command_channel {
            Some(channel) => {
                channel.send((String::from(command), params)).unwrap();
            }
            None => {}
        }
    }

    /// Same as command, however it takes Vec<&str> and converts is himself
    pub fn command_str(&mut self, command: &str, params: Vec<&str>) {
        let string_params = params.iter()
            .map(|s: &&str| String::from(*s)).collect::<Vec<String>>();
        self.command(command, string_params);
    }

    /// Initialises the connection with the drone, sends 2 commands which
    /// seem to initialize the drone (taken from ps_drone). Creates a separate
    /// thread to deal with sending these commands and the keepalive sign.
    /// Parameters: echo_commands: Should it print every command sent? (except keepalives)
    pub fn start_connection(&mut self, echo_commands: &bool) -> Result<(), String> {
        let socket = UdpSocket::bind("0.0.0.0:5556").expect("couldn't bind to address");
        socket.set_nonblocking(true).unwrap();
        let address = format!("{}:{}", self.drone_ip, self.cmd_port);

        // Creating a channel to kill the thread
        let (sender, receiver) = mpsc::channel();
        self.command_channel = Some(sender);

        // Sending first two commands
        let s = String::from("\r");
        socket.send_to(s.as_bytes(), &address).unwrap();
        thread::sleep(time::Duration::from_millis(10));
        let s = String::from("AT*PMODE=1,2\rAT*MISC=2,2,20,2000,3000\r");
        let e_c = *echo_commands;
        if e_c {
            println!("{}", s);
        }
        socket.send_to(s.as_bytes(), &address).unwrap();

        self.connection_thread = Some(thread::spawn(move || {
            communication_thread(socket,
                                 receiver,
                                 address,
                                 e_c);
        }));
        return Ok(());
    }

    /// Shuts down the communication thread and the connection to the drone
    pub fn shutdown_connection(mut self) {
        let sender = self.command_channel.unwrap();
        sender.send((String::from("exit"), Vec::new())).unwrap();
        self.command_channel = None;
        self.connection_thread.take().unwrap().join().unwrap();
        self.connection_thread = None;
    }

    pub fn get_ctl_tcp_connection(&self) -> Result<TcpStream, String> {
        let socket = TcpStream::connect(format!("{}:{}", self.drone_ip, self.ctl_port)) ;
        match socket {
            Ok(stream) => {
                return Ok(stream);
            }
            Err(error) => {
                return Err(format!("{}", error));
            }
        }
    }

}
