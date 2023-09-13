use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use notify_rust::Notification;
use std::thread;

pub struct Timer {
    running: Arc<AtomicBool>,
    target_time: Option<Instant>,
    paused_duration: Option<Duration>,
}


impl Timer {
    pub fn new() -> Self {
        Timer { 
            running: Arc::new(AtomicBool::new(false)),
            target_time: None,
            paused_duration: None,
        }
    }

    pub fn start(&mut self, duration: Duration) {
        self.running = Arc::from(AtomicBool::new(true));
        self.target_time = Some(Instant::now() + duration);
        self.start_timer()
    }


    pub fn pause(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.paused_duration = Some(self.target_time.unwrap().duration_since(Instant::now()));
    }

    pub fn resume(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        self.target_time = Some(Instant::now() + self.paused_duration.unwrap());
        self.paused_duration = None;
        self.start_timer();
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.target_time = None;
        self.paused_duration = None;
    }


    fn start_timer(&mut self) {
        let running_clone = self.running.clone();
        let target_time_clone = self.target_time.clone();

        thread::spawn(move || {
        let mut finished = false;
            while running_clone.load(Ordering::Relaxed) {
                if target_time_clone <= Some(Instant::now()) {
                    running_clone.store(false, Ordering::Relaxed); 
                    finished = true;
                }


                thread::sleep(Duration::from_millis(200));
            }
            if finished {
                Notification::new().summary("Timer ran out!").icon("/usr/share/icons/Dracula/24/actions/colors-chromared.svg").show().expect("crashed at trying to send notification");
                std::process::Command::new("paplay").arg("/usr/share/sounds/freedesktop/stereo/bell.oga").output().expect("crashed trying to execute paplay");
                println!("timer ran out");
            }
        });
    }

    pub fn get_remaining_time(&self) -> String {
        if self.running.load(Ordering::Relaxed) {
            let remaining_time = self.target_time.unwrap().duration_since(Instant::now()).as_secs();
            format!("{:02}:{:02}:{:02}", remaining_time / 3600, (remaining_time % 3600) / 60, remaining_time % 60)
        } else {
            let remaining_time = self.paused_duration.unwrap_or_else(|| Duration::from_secs(0)).as_secs();
            format!("{:02}:{:02}:{:02}", remaining_time / 3600, (remaining_time % 3600) / 60, remaining_time % 60)
        }
    }
}