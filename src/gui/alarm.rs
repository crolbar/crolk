use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::{Timelike, Local};
use chrono::Datelike;
use notify_rust::Notification;
use std::thread;



pub struct Alarm {
    id: u8,
    running: Arc<AtomicBool>,
    target_time: u32,
}


impl Alarm {
    pub fn new(id: u8) -> Self {
        Alarm { 
            id: id,
            running: Arc::new(AtomicBool::new(true)),
            target_time: 0,
        }
    }

    pub fn start(&mut self, time: u32 ) {
        self.running = Arc::from(AtomicBool::new(true));
        self.target_time = time;
        self.start_alarm()
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.target_time = 0;
    }


    fn start_alarm(&mut self) {
        let running_clone = self.running.clone();
        let target_time_clone = self.target_time.clone();

       thread::spawn(move || {
        let mut finished = false;
            while running_clone.load(Ordering::Relaxed) {
                if target_time_clone == format!("{}{}{}", Local::now().hour(), Local::now().minute(), Local::now().weekday().number_from_monday()).parse::<u32>().expect("crash at trying to convert string time to u32 time") {
                    running_clone.store(false, Ordering::Relaxed); 
                    finished = true;
                }


                thread::sleep(Duration::from_millis(900));
            }
            if finished {
                Notification::new().summary("Alarm ran out!").icon("/usr/share/icons/Dracula/24/actions/colors-chromared.svg").show().unwrap();
                std::process::Command::new("paplay").arg("/usr/share/sounds/freedesktop/stereo/bell.oga").output().expect("crashed trying to execute paplay");
                println!("Alarm ran out");
            }
        });
    }

    pub fn get_state(&self) -> bool {
       self.running.load(Ordering::Relaxed)
    }
}