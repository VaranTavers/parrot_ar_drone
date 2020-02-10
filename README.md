# parrot_ar_drone

# Introduction
This is a Rust API to control Parrot AR Drone 2.0. It is based upon ps_drone (the Python API),
and may be taken down if the author of that API requires it (hopefully not).

# Cargo
```
parrot_ar_drone = "0.1.0"
```

# Example Code
```rust
    use parrot_ar_drone::*;
    use navdata::NavDataValue;
    use std::{time, thread};

    fn main() {
        let mut drone = get_drone();
        
        let test_result = drone.startup();
        match test_result {
            Ok(()) => {
                println!("Drone connection successful.");
                thread::sleep(time::Duration::from_secs(3)); // It is advised to wait a bit for the connections to establish.
                drone.trim();
                let mut i = 0;
                drone.takeoff();
                loop {
                    thread::sleep(time::Duration::from_secs(1));
                    match drone.get_navdata("demo_battery") {
                        Some(NavDataValue::Uint(a)) => { println!("Battery: {}%", a); }
                        _ => { println!("Battery status unknown!"); }
                    }
                    match drone.get_navdata("header_seq_num") {
                        Some(NavDataValue::Uint(a)) => { println!("Seq num: {}", a); }
                        _ => { println!("Seq num unknown!"); }
                    }

                    match drone.get_navdata("demo_altitude") {
                        Some(NavDataValue::Int(a)) => { println!("Alt: {}", a); }
                        _ => { println!("Altitude unknown!"); }
                    }
                    i += 1;
                    if i == 10 {
                        break;
                    }
                }
                drone.land();
                drone.shutdown();
            }
            _ => {}
        }
    }
```
