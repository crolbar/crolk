use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::{Timelike, Local};
use notify_rust::Notification;

pub struct Alarm {
    running: Arc<AtomicBool>,
    target_time: Option<String>,
}


impl Alarm {
    pub fn new() -> Self {
        Alarm { 
            running: Arc::new(AtomicBool::new(true)),
            target_time: None,
        }
    }

    pub fn start(&mut self, time: String ) {
        self.running = Arc::from(AtomicBool::new(true));
        self.target_time = Some(time);
        self.start_alarm()
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.target_time = None;
    }


    fn start_alarm(&mut self) {
        let running_clone = self.running.clone();
        let target_time_clone = self.target_time.clone().unwrap();

        std::thread::spawn(move || {
        let mut finished = false;
            while running_clone.load(Ordering::Relaxed) {
                if target_time_clone == format!("{} {}", Local::now().hour(), Local::now().minute()) {
                    running_clone.store(false, Ordering::Relaxed); 
                    finished = true;
                }


                std::thread::sleep(Duration::from_millis(800));
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