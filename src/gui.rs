use gtk::{gdk, gio, Button, Orientation, prelude::*};
use std::fs::{self, read_to_string, File};
use std::io::{self, Write, BufRead};
use std::sync::{Arc, Mutex, mpsc};
use gtk::traits::LabelExt;
use std::cell::RefCell;
use dirs::home_dir;
use glib::clone;
use toml::Value;

mod stopwatch;
use stopwatch::Stopwatch;

mod timer;
use timer::Timer;

mod alarm;
use alarm::Alarm;

fn buid_app(application: &gtk::Application) {
    application.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        let style = include_bytes!("style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    application.run();
}

pub fn main(rx: mpsc::Receiver<bool>) {
    let application = gtk::Application::new(Some("com.crolk"), gio::ApplicationFlags::empty());
    let window = gtk::ApplicationWindow::new(&application);

    build_ui(&window);
    buid_app(&application);


    glib::timeout_add_local(std::time::Duration::from_millis(100), move || match rx.try_recv() {
        Ok(true) => {
            match window.is_visible() {
                true => window.hide(),
                false => window.show(),
            }
            glib::ControlFlow::Continue
        }
        _ => glib::ControlFlow::Continue
    });
}


fn build_ui(window: &gtk::ApplicationWindow) {
    window.set_title("crolk");
    window.set_default_size(427, 400);
    window.set_widget_name("window");
    window.connect_delete_event(|window, _| window.hide_on_delete());

    let main_box = gtk::Box::new(Orientation::Vertical, 20);
    
        let clock_label = gtk::Label::new(None);
        let clock_box: gtk::Box= gtk::Box::new(Orientation::Vertical, 0);
        {// clock
            let curr_time = current_time();
            clock_label.set_text(&curr_time);


            clock_box.set_valign(gtk::Align::Center);
            clock_box.add(&clock_label);
            clock_box.show_all();
        }


        let stopwatch = Arc::new(Mutex::new(RefCell::new(Stopwatch::new())));
        let stopwatch_box: gtk::Box = gtk::Box::new(Orientation::Vertical, 0); 
        let stopwatch_label = gtk::Label::new(None);
        let sw_button_pause = Button::with_label("Pause/Resume");
        let sw_button_reset = Button::with_label("Reset");
        let sw_button_lap= Button::with_label("Lap");
        let sw_label_lap = gtk::Label::new(None);
        { // STOPWATCH
            let button_start = Button::with_label("Start");

            // START
            let stopwatch_clone = Arc::clone(&stopwatch);
            button_start.connect_clicked(clone!(@strong sw_button_pause, @strong stopwatch_box, @strong sw_button_lap, @strong sw_label_lap => move |button_start| {
                let stopwatch = stopwatch_clone.lock().unwrap();
                let mut inner_stopwatch = stopwatch.borrow_mut();
                inner_stopwatch.start();

                button_start.hide();
                sw_button_pause.show();
                sw_button_lap.show();
                sw_label_lap.set_text("");
                stopwatch_box.remove(&sw_label_lap);
            }));

            // PAUSE/UNPAUSE
            let stopwatch_clone = Arc::clone(&stopwatch);
            sw_button_pause.connect_clicked(clone!(@strong sw_button_reset => move |_| {
                let stopwatch = stopwatch_clone.lock().unwrap();
                let mut inner_stopwatch = stopwatch.borrow_mut();
                inner_stopwatch.pause_unpause();

                if sw_button_reset.is_visible() { sw_button_reset.hide() } else { sw_button_reset.show() }
            }));

            // RESET
            let stopwatch_clone = Arc::clone(&stopwatch);
            sw_button_reset.connect_clicked(clone!(@strong button_start, @strong sw_button_pause, @strong sw_button_lap => move |sw_button_reset| {
                stopwatch_clone.lock().unwrap().replace(Stopwatch::new()); 

                button_start.show();
                sw_button_pause.hide();
                sw_button_reset.hide();
                sw_button_lap.hide()
            }));

            // LAP
            let stopwatch_clone = Arc::clone(&stopwatch);
            sw_button_lap.connect_clicked(clone!(@strong stopwatch_box, @strong window, @strong sw_label_lap => move |_| {
                let lap_info = stopwatch_clone.lock().unwrap().borrow_mut().lap();
                sw_label_lap.set_text(format!("{}\n{}: {}",sw_label_lap.text() ,lap_info.0 ,lap_info.1).as_str());
                stopwatch_box.remove(&sw_label_lap);
                stopwatch_box.add(&sw_label_lap);
                sw_label_lap.show();
            }));

            button_start.set_widget_name("button_close");
            sw_button_lap.set_widget_name("button_close");
            sw_button_pause.set_widget_name("button_close");
            sw_button_reset.set_widget_name("button_close");
            
            stopwatch_label.set_widget_name("label");

            stopwatch_box.add(&button_start);
            stopwatch_box.add(&sw_button_pause);
            stopwatch_box.add(&sw_button_reset);
            stopwatch_box.add(&sw_button_lap);
            stopwatch_box.add(&stopwatch_label);
            stopwatch_box.show_all();

            sw_button_lap.hide();
            sw_button_pause.hide();
            sw_button_reset.hide();
        }



        let timer = Timer::new();
        let arc_timer = Arc::new(Mutex::new(timer));
        let timer_box = gtk::Box::new(Orientation::Vertical, 0); 
        let timer_label = gtk::Label::new(None);
        let t_button_start = Button::with_label("Start");
        let t_button_stop = Button::with_label("Stop");
        let t_button_pause = Button::with_label("Pause");
        let t_button_resume = Button::with_label("Resume");
        { // TIMER
            let duration_box = gtk::Box::new(Orientation::Horizontal, 5);

            let hours_comb = gtk::ComboBoxText::new();
            let hour_label = gtk::Label::new(Some("H"));
            hour_label.set_widget_name("t_label");
            for i in 0..=60 { hours_comb.append_text(i.to_string().as_str()); }
            let min_comb = gtk::ComboBoxText::new();
            let min_label = gtk::Label::new(Some("M"));
            min_label.set_widget_name("t_label");
            for i in 0..=60 { min_comb.append_text(i.to_string().as_str()); }
            let sec_comb = gtk::ComboBoxText::new();
            let sec_label = gtk::Label::new(Some("S"));
            sec_label.set_widget_name("t_label");
            for i in 0..=60 { sec_comb.append_text(i.to_string().as_str()); }

            hours_comb.set_active(Some(0));
            min_comb.set_active(Some(0));
            sec_comb.set_active(Some(0));

            duration_box.add(&hours_comb);
            duration_box.add(&hour_label);
            duration_box.add(&min_comb);
            duration_box.add(&min_label);
            duration_box.add(&sec_comb);
            duration_box.add(&sec_label);
            duration_box.set_halign(gtk::Align::Center);


            t_button_start.connect_clicked(clone!(@strong arc_timer, @strong t_button_pause, @strong t_button_stop => move |t_button_start| {
                let duration = {
                    hours_comb.active_text().unwrap().parse::<u64>().unwrap() * 3600 +
                    min_comb.active_text().unwrap().parse::<u64>().unwrap() * 60 + 
                    sec_comb.active_text().unwrap().parse::<u64>().unwrap()
                };
                arc_timer.lock().unwrap().start(std::time::Duration::from_secs(duration));

                t_button_start.hide();
                t_button_pause.show();
                t_button_stop.show();
            }));

            t_button_pause.connect_clicked(clone!(@strong arc_timer, @strong t_button_resume => move |t_button_pause| {
                arc_timer.lock().unwrap().pause();

                t_button_pause.hide();
                t_button_resume.show();
            }));

            t_button_resume.connect_clicked(clone!(@strong arc_timer, @strong t_button_pause => move |t_button_resume| {
                arc_timer.lock().unwrap().resume();

                t_button_resume.hide();
                t_button_pause.show();
            }));


            t_button_stop.connect_clicked(clone!(@strong arc_timer, @strong t_button_pause, @strong t_button_resume, @strong t_button_start => move |t_button_stop| {
                arc_timer.lock().unwrap().stop();

                t_button_pause.hide();
                t_button_resume.hide();
                t_button_stop.hide();
                t_button_start.show();
            }));



            timer_box.add(&duration_box);
            timer_box.add(&t_button_start);
            timer_box.add(&t_button_pause);
            timer_box.add(&t_button_resume);
            timer_box.add(&t_button_stop);
            timer_box.add(&timer_label);
            timer_box.show_all();
            t_button_pause.hide();
            t_button_resume.hide();
            t_button_stop.hide();
        }

        



        let alarm_box: gtk::Box = gtk::Box::new(Orientation::Vertical, 5); 
        { // ALARM
            let path = home_dir().unwrap().join(".config/crolk/presets.toml");

            if !path.exists() {
                fs::create_dir_all(home_dir().unwrap().join(".config/crolk")).unwrap();
                File::create(&path).unwrap();
            }

            let save_file: Value = toml::from_str(read_to_string(path).expect("Failed to read presets.toml").as_str()).expect("Failed to parse TOML");
            if let Some(table) = save_file.as_table() {
                for (key, entry) in table.iter() {
                    if let (Some(hour), Some(min)) = (
                        entry.get("hour").and_then(|v| v.as_integer()),
                        entry.get("min").and_then(|v| v.as_integer()),
                    ) {
                        let is_pm = hour > 11;
                        let alarm_preset_box = create_alarm_box(hour as u32, min as u32, true, key.clone(), is_pm);
                        alarm_box.add(&alarm_preset_box);
                    } 
                }
            } 


            let button_add_box = gtk::Box::new(Orientation::Horizontal, 0);
            {
                let button_add = gtk::Button::with_label("+");
                button_add.connect_clicked(clone!(@strong alarm_box, @strong button_add_box => move |_| {
                    let alarm_preset_box = create_alarm_box(0, 0, false, String::from("0"), false);
                    alarm_box.add(&alarm_preset_box);
                    alarm_box.remove(&button_add_box);
                    alarm_box.add(&button_add_box);
                }));
                
                button_add_box.add(&button_add);
                button_add_box.show_all();
            }
            
            alarm_box.add(&button_add_box);
            alarm_box.show();
        }



    let top_box: gtk::Box = gtk::Box::new(Orientation::Horizontal, 5);
    { // TOP BAR
        use gtk::glib;
        use glib::clone;


        let button_close = Button::with_label("X");
        button_close.set_widget_name("button_close");
        button_close.connect_clicked(clone!(@strong window => move |_| {
            window.close()
        }));

        let button_clock = Button::with_label("Clock");
        button_clock.set_widget_name("button_close");
        button_clock.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box => move |_| {
            clock_box.show();
            stopwatch_box.hide();
            timer_box.hide();
            alarm_box.hide();
        }));

        let button_stopwatch = Button::with_label("Stopwatch");
        button_stopwatch.set_widget_name("button_close");
        button_stopwatch.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box => move |_| {
            clock_box.hide();
            stopwatch_box.show();
            timer_box.hide();
            alarm_box.hide();
        }));

        let button_timer = Button::with_label("Timer");
        button_timer.set_widget_name("button_close");
        button_timer.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box => move |_| {
            clock_box.hide();
            stopwatch_box.hide();
            timer_box.show();
            alarm_box.hide();
        }));

        let button_alarm = Button::with_label("Alarm");
        button_alarm.set_widget_name("button_close");
        button_alarm.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box => move |_| {
            clock_box.hide();
            stopwatch_box.hide();
            timer_box.hide();
            alarm_box.show();
        }));
        


        top_box.add(&button_alarm);
        top_box.add(&button_timer);
        top_box.add(&button_stopwatch);
        top_box.add(&button_clock);
        top_box.add(&button_close);
        top_box.show_all();
    }



    main_box.add(&top_box);
    main_box.add(&clock_box);
    main_box.add(&stopwatch_box);
    main_box.add(&timer_box);
    main_box.add(&alarm_box);
    window.add(&main_box);
    main_box.show();
    window.show();


    stopwatch_box.hide();
    timer_box.hide();
    alarm_box.hide();



    glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
        if main_box.is_visible() {
        // CLOCK
            clock_label.set_text(&current_time().as_str());

        // STOPWATCH
            stopwatch_label.set_text(stopwatch.lock().unwrap().borrow_mut().get_elapsed().as_str());

        // TIMER
            let timer = arc_timer.lock().unwrap();
            timer_label.set_text(timer.get_remaining_time().as_str());
            if timer.get_remaining_time() == "00:00:00" { t_button_start.show(); t_button_stop.hide(); t_button_pause.hide() }
        } 
        
        glib::ControlFlow::Continue
    });
}


fn create_alarm_box(hour: u32, min: u32, is_preset: bool, key_num: String, is_pm: bool) -> gtk::Box {
    let alarm = Alarm::new();
    let arc_alarm = Arc::new(Mutex::new(alarm));
    let alarm_preset_box = gtk::Box::new(Orientation::Horizontal, 5);
    {
        let hours_comb = gtk::ComboBoxText::new();
        let hour_label = gtk::Label::new(Some("H"));
        hour_label.set_widget_name("t_label");
        for i in 1..=12 { hours_comb.append_text(i.to_string().as_str()); }

        let min_comb = gtk::ComboBoxText::new();
        let min_label = gtk::Label::new(Some("M"));
        min_label.set_widget_name("t_label");
        for i in 0..=59 { min_comb.append_text(i.to_string().as_str()); }

        let hour_comb_text_index = if is_pm { (hour % 12) - 1 } else { match hour { 0 => 11, _ => hour - 1 } };

        hours_comb.set_active(Some(hour_comb_text_index));
        min_comb.set_active(Some(min));

        let pm_am_button = gtk::Button::with_label("AM");
        let stop_button = gtk::Button::with_label("Stop");
        let start_button = gtk::Button::with_label("Start");
        let del_button = gtk::Button::with_label("-");
        let save_button = gtk::Button::with_label("Save");


        pm_am_button.set_widget_name("pm_am_button");
        pm_am_button.set_tooltip_text(Some("switch between am and pm"));
        pm_am_button.connect_clicked(move |pm_am_button| {
            pm_am_button.set_label(if pm_am_button.label().unwrap() == "AM" { "PM" } else { "AM" });
        });
        if is_pm { pm_am_button.set_label("PM") };


        start_button.connect_clicked(clone!(@strong stop_button, @strong arc_alarm, @strong hours_comb, @strong min_comb, @strong pm_am_button => move |start_button| {
            let hour = match (hours_comb.active_text().unwrap().as_str(), pm_am_button.label().unwrap().as_str()) {
                ("1", "PM") => 13, ("1", "AM") => 1,
                ("2", "PM") => 14, ("2", "AM") => 2,
                ("3", "PM") => 15, ("3", "AM") => 3,
                ("4", "PM") => 16, ("4", "AM") => 4,
                ("5", "PM") => 17, ("5", "AM") => 5,
                ("6", "PM") => 18, ("6", "AM") => 6,
                ("7", "PM") => 19, ("7", "AM") => 7,
                ("8", "PM") => 20, ("8", "AM") => 8,
                ("9", "PM") => 21, ("9", "AM") => 9,
                ("10", "PM") => 22, ("10", "AM") => 10,
                ("11", "PM") => 23, ("11", "AM") => 11,
                ("12", "PM") => 12, ("12", "AM") => 0,
                _ => { println!("Invalid hour time"); return }
            };

            let time = format!("{} {}", hour, min_comb.active_text().unwrap());
            arc_alarm.lock().unwrap().start(time);
            
            glib::timeout_add_local(std::time::Duration::from_millis(800),clone!(@strong start_button, @strong stop_button, @strong arc_alarm 
            => move || match arc_alarm.lock().unwrap().get_state() {
                true => glib::ControlFlow::Continue,
                false => {
                    stop_button.hide();
                    start_button.show();
                    glib::ControlFlow::Break
                }
            }));

            stop_button.show();
            start_button.hide()
        }));


        stop_button.connect_clicked(clone!(@strong start_button, @strong arc_alarm => move |stop_button| {
            arc_alarm.lock().unwrap().stop();
            stop_button.hide();
            start_button.show()
        }));


        let key_num = Arc::new(std::sync::atomic::AtomicU32::new(key_num.parse::<u32>().unwrap()));
        save_button.set_widget_name("save_button");
        save_button.connect_clicked(clone!(@strong hours_comb, @strong min_comb, @strong key_num, @strong pm_am_button => move |save_button| {
            let hour = match (hours_comb.active_text().unwrap().as_str(), pm_am_button.label().unwrap().as_str()) {
                ("1", "PM") => 13, ("1", "AM") => 1,
                ("2", "PM") => 14, ("2", "AM") => 2,
                ("3", "PM") => 15, ("3", "AM") => 3,
                ("4", "PM") => 16, ("4", "AM") => 4,
                ("5", "PM") => 17, ("5", "AM") => 5,
                ("6", "PM") => 18, ("6", "AM") => 6,
                ("7", "PM") => 19, ("7", "AM") => 7,
                ("8", "PM") => 20, ("8", "AM") => 8,
                ("9", "PM") => 21, ("9", "AM") => 9,
                ("10", "PM") => 22, ("10", "AM") => 10,
                ("11", "PM") => 23, ("11", "AM") => 11,
                ("12", "PM") => 12, ("12", "AM") => 0,
                _ => { println!("Invalid hour time"); return }
            };

            let min = min_comb.active_text().unwrap();
            let file_path = home_dir().unwrap().join(".config/crolk/presets.toml");
            let key = io::BufReader::new(File::open(&file_path).unwrap()).lines().last().unwrap_or_else(|| Ok("0 ".to_string())).unwrap().split_once(" ").unwrap().0.parse::<u32>().unwrap();

            key_num.store(key + 1, std::sync::atomic::Ordering::Relaxed);
            write!(std::fs::OpenOptions::new().write(true).append(true).open(file_path).unwrap(), "\n{} = {{hour = {}, min = {}}}",&key + 1, hour, min).expect("Failed to write to presets file!");  

            save_button.hide();
        }));
         
        del_button.set_widget_name("del_button");
        del_button.connect_clicked(clone!(@strong alarm_preset_box, @strong arc_alarm, @strong key_num => move |_| {
            unsafe { alarm_preset_box.destroy() }
            arc_alarm.lock().unwrap().stop();


            let file_path = home_dir().unwrap().join(".config/crolk/presets.toml");
            let contents = fs::read_to_string(&file_path).unwrap();
            let filtered_contents: String = contents.lines().filter(|line| {
                    if let Some((key, _)) = line.trim().split_once('=') {
                        return key.trim() != key_num.load(std::sync::atomic::Ordering::Relaxed).to_string()
                    }
                    true 
                }).collect::<Vec<&str>>().join("\n");
            fs::write(file_path, filtered_contents).unwrap();
        }));
            

        alarm_preset_box.add(&del_button);
        alarm_preset_box.add(&pm_am_button);
        alarm_preset_box.add(&hours_comb);
        alarm_preset_box.add(&hour_label);
        alarm_preset_box.add(&min_comb);
        alarm_preset_box.add(&min_label);
        alarm_preset_box.add(&start_button);
        alarm_preset_box.add(&stop_button);
        if !is_preset.to_owned() { alarm_preset_box.add(&save_button) }
        alarm_preset_box.show_all();
        stop_button.hide();
    }
    alarm_preset_box
}

fn current_time() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %I:%M:%S %p"))
}