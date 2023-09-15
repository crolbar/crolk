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
        self.notify_time_until_alarm();
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

    fn notify_time_until_alarm(&self) {
        let mut target_time_secs = 0;
        let current_time_secs = (Local::now().hour() * 3600 + Local::now().minute() * 60) as i32;
        for (i, c) in self.target_time.clone().unwrap().split_whitespace().enumerate() {
            match i {
                0 =>  target_time_secs += c.parse::<i32>().unwrap() * 3600,
                1 => target_time_secs += c.parse::<i32>().unwrap() * 60,
                _ => ()
            }
        };

        let time_untill_alarm = match target_time_secs > current_time_secs {
            true => target_time_secs - current_time_secs,
            false => target_time_secs - current_time_secs + 86400
        };

        let time_untill_alarm = format!("Time untill alarm: {} hours and {} minutes", time_untill_alarm / 3600, (time_untill_alarm % 3600) / 60);
        Notification::new().summary(&time_untill_alarm).icon("/usr/share/icons/Dracula/24/actions/colors-chromared.svg").show().unwrap();
    }
}