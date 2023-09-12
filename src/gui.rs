use gtk::{gdk, gio, Button, Orientation, prelude::*};
use gtk::traits::LabelExt;
use std::sync::{Arc, Mutex, mpsc};
use std::cell::RefCell;
use glib::clone;

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
    window.set_default_size(400, 400);
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
        }

        
        let alarm = Alarm::new(1);
        let arc_alarm = Arc::new(Mutex::new(alarm));
        let a_switch = gtk::Switch::new();
        let alarm_box: gtk::Box = gtk::Box::new(Orientation::Vertical, 5); 
        { // ALARM
            let alarm_preset_box = create_alarm_box(1, &arc_alarm, &a_switch);
            
 
            let button_add_box = gtk::Box::new(Orientation::Horizontal, 0);
            {
                let button_add = gtk::Button::with_label("+");
                button_add.connect_clicked(clone!(@strong alarm_box, @strong button_add_box, @strong arc_alarm, @strong a_switch => move |_| {
                    let alarm_preset_box = create_alarm_box(1, &arc_alarm, &a_switch);
                    alarm_box.add(&alarm_preset_box);
                    alarm_box.remove(&button_add_box);
                    alarm_box.add(&button_add_box);
                    alarm_box.show_all();
                }));
                
                button_add_box.add(&button_add);
            }
            
            
            alarm_box.add(&alarm_preset_box);
            alarm_box.add(&button_add_box);
        }




    let top_box: gtk::Box = gtk::Box::new(Orientation::Horizontal, 5);
    { // TOP BAR
        use gtk::glib;
        use glib::clone;


        let button_close = Button::with_label("X");
        button_close.set_widget_name("button_close");
        button_close.connect_clicked(clone!(@strong window => move |_| {
            window.close();
            // std::process::exit(0);
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
        


        // top_box.set_halign(gtk::Align::Start);
        top_box.add(&button_alarm);
        top_box.add(&button_timer);
        top_box.add(&button_stopwatch);
        top_box.add(&button_clock);
        top_box.add(&button_close);
    }



    main_box.add(&top_box);
    main_box.add(&clock_box);
    main_box.add(&stopwatch_box);
    main_box.add(&timer_box);
    main_box.add(&alarm_box);
    window.add(&main_box);
    window.show_all();


    stopwatch_box.hide();
    timer_box.hide();
    alarm_box.hide();


    t_button_pause.hide();
    t_button_resume.hide();
    t_button_stop.hide();
    

    sw_button_lap.hide();
    sw_button_pause.hide();
    sw_button_reset.hide();


    // let a_switch_clone = a_switch.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
    // CLOCK
        clock_label.set_text(&current_time().as_str());

    // STOPWATCH
        stopwatch_label.set_text(stopwatch.lock().unwrap().borrow_mut().get_elapsed().as_str());

    // TIMER
        let timer = arc_timer.lock().unwrap();
        timer_label.set_text(timer.get_remaining_time().as_str());
        if timer.get_remaining_time() == "00:00:00" { t_button_start.show(); t_button_stop.hide(); t_button_pause.hide() }

    // ALARM
        if !arc_alarm.lock().unwrap().get_state() { a_switch.set_state(false) }

        glib::ControlFlow::Continue
    });

}


fn create_alarm_box(_id: u8, arc_alarm: &Arc<Mutex<Alarm>>, a_switch: &gtk::Switch) -> gtk::Box {
    let alarm_preset_box = gtk::Box::new(Orientation::Horizontal, 5);
    {

        let day_comb = gtk::ComboBoxText::new();
        let day_label = gtk::Label::new(Some("D"));
        day_label.set_widget_name("t_label");
        day_comb.append_text("Monday");
        day_comb.append_text("Tuesday");
        day_comb.append_text("Wednesday");
        day_comb.append_text("Thursday");
        day_comb.append_text("Friday");
        day_comb.append_text("Saturday");
        day_comb.append_text("Sunday");

        let hours_comb = gtk::ComboBoxText::new();
        let hour_label = gtk::Label::new(Some("H"));
        hour_label.set_widget_name("t_label");
        for i in 0..=24 { hours_comb.append_text(i.to_string().as_str()); }

        let min_comb = gtk::ComboBoxText::new();
        let min_label = gtk::Label::new(Some("M"));
        min_label.set_widget_name("t_label");
        for i in 0..=59 { min_comb.append_text(i.to_string().as_str()); }

        day_comb.set_active(Some(0));
        hours_comb.set_active(Some(0));
        min_comb.set_active(Some(0));



        a_switch.connect_changed_active(clone!(@strong arc_alarm, @strong hours_comb, @strong min_comb, @strong day_comb => move |a_switch| {
            let is_active = a_switch.is_active();
            let day = match day_comb.active_text().unwrap().as_str() {
                "Monday" => 1,
                "Tuesday" => 2,
                "Wednesday" => 3,
                "Thursday" => 4,
                "Friday" => 5,
                "Saturday" => 6,
                "Sunday" => 7,
                _ => return
            };  
            if is_active {
                let time = {
                    format!("{}{}{}", hours_comb.active_text().unwrap().parse::<u32>().unwrap(), min_comb.active_text().unwrap().parse::<u32>().unwrap(), day)
                    .parse::<u32>().expect("crashed at trying to convert time as string to time as u32")};
                arc_alarm.lock().unwrap().start(time);
            } else {
                arc_alarm.lock().unwrap().stop();
            }
        }));


        alarm_preset_box.set_halign(gtk::Align::Center);

        alarm_preset_box.add(&day_comb);
        alarm_preset_box.add(&day_label);
        alarm_preset_box.add(&hours_comb);
        alarm_preset_box.add(&hour_label);
        alarm_preset_box.add(&min_comb);
        alarm_preset_box.add(&min_label);
        alarm_preset_box.add(a_switch);
    }
    alarm_preset_box
}

fn current_time() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))
}