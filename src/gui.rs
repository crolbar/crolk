use gtk::{gdk, gio, Button, Orientation, prelude::*};
use gtk::traits::LabelExt;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use glib::clone;

mod stopwatch;
use stopwatch::Stopwatch;


pub fn main() {
    let application = gtk::Application::new(Some("com.crolk"), gio::ApplicationFlags::empty());
    
    application.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        let style = include_bytes!("style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        app.connect_activate(build_ui);
    });

    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("crolk");
    window.set_default_size(400, 400);
    window.set_widget_name("window");



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
        let pause_button = Button::with_label("Pause");
        let reset_button = Button::with_label("Reset");
        let lap_button= Button::with_label("Lap");
        let lap_label = gtk::Label::new(None);
        { // STOPWATCH
            let start_button = Button::with_label("Start");

            // START
            let stopwatch_clone = Arc::clone(&stopwatch);
            start_button.connect_clicked(clone!(@strong start_button, @strong pause_button, @strong stopwatch_box, @strong lap_button, @strong lap_label => move |_| {
                let stopwatch = stopwatch_clone.lock().unwrap();
                let mut inner_stopwatch = stopwatch.borrow_mut();
                inner_stopwatch.start();

                start_button.hide();
                pause_button.show();
                lap_button.show();
                lap_label.set_text("");
                stopwatch_box.remove(&lap_label);
            }));

            // PAUSE/UNPAUSE
            let stopwatch_clone = Arc::clone(&stopwatch);
            pause_button.connect_clicked(clone!(@strong reset_button => move |_| {
                let stopwatch = stopwatch_clone.lock().unwrap();
                let mut inner_stopwatch = stopwatch.borrow_mut();
                inner_stopwatch.pause_unpause();

                if reset_button.is_visible() { reset_button.hide() } else { reset_button.show() }
            }));

            // RESET
            let stopwatch_clone = Arc::clone(&stopwatch);
            reset_button.connect_clicked(clone!(@strong start_button, @strong pause_button, @strong lap_button => move |reset_button| {
                stopwatch_clone.lock().unwrap().replace(Stopwatch::new()); 

                start_button.show();
                pause_button.hide();
                reset_button.hide();
                lap_button.hide()
            }));

            // LAP
            let stopwatch_clone = Arc::clone(&stopwatch);
            lap_button.connect_clicked(clone!(@strong stopwatch_box, @strong window, @strong lap_label => move |_| {
                let lap_info = stopwatch_clone.lock().unwrap().borrow_mut().lap();
                lap_label.set_text(format!("{}\n{}: {}",lap_label.text() ,lap_info.0 ,lap_info.1).as_str());
                stopwatch_box.remove(&lap_label);
                stopwatch_box.add(&lap_label);
                lap_label.show();
            }));
            
            stopwatch_box.add(&start_button);
            stopwatch_box.add(&pause_button);
            stopwatch_box.add(&reset_button);
            stopwatch_box.add(&lap_button);
            stopwatch_box.add(&stopwatch_label);
        }



        let timer_box: gtk::Box = gtk::Box::new(Orientation::Vertical, 0); 
        { // TIMER

            let label = gtk::Label::new(Some("TIMER"));



            timer_box.add(&label);
        }

        

       let alarm_box: gtk::Box = gtk::Box::new(Orientation::Vertical, 0); 
        { // ALARM

            let label = gtk::Label::new(Some("ALARM"));


            alarm_box.add(&label);
        }



        





    let top_box: gtk::Box = gtk::Box::new(Orientation::Horizontal, 5);
    { // TOP BAR
        use gtk::glib;
        use glib::clone;


        let button_close = Button::with_label("X");
        button_close.set_widget_name("button_close");
        button_close.connect_clicked(clone!(@strong window => move |_| {
            // window.close();
            std::process::exit(0);
        }));

        let button_clock = Button::with_label("Clock");
        button_clock.set_widget_name("button_close");
        button_clock.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box, @strong main_box, @strong window => move |_| {
            main_box.add(&clock_box);
            main_box.remove(&stopwatch_box);
            main_box.remove(&timer_box);
            main_box.remove(&alarm_box);
            window.show_all();
        }));         

        let button_stopwatch = Button::with_label("Stopwatch");
        button_stopwatch.set_widget_name("button_close");
        button_stopwatch.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box, @strong main_box, @strong window, @weak pause_button, @weak lap_button, @weak reset_button => move |_| {
            main_box.remove(&clock_box);
            main_box.add(&stopwatch_box);
            main_box.remove(&timer_box);
            main_box.remove(&alarm_box);
            window.show_all();
            lap_button.hide();
            pause_button.hide();
            reset_button.hide();
        }));

        let button_timer = Button::with_label("Timer");
        button_timer.set_widget_name("button_close");
        button_timer.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box, @strong main_box, @strong window => move |_| {
            main_box.remove(&clock_box);
            main_box.remove(&stopwatch_box);
            main_box.add(&timer_box);
            main_box.remove(&alarm_box);
            window.show_all();
        }));

        let button_alarm = Button::with_label("Alarm");
        button_alarm.set_widget_name("button_close");
        button_alarm.connect_clicked(clone!(@strong stopwatch_box, @strong clock_box, @strong timer_box, @strong alarm_box, @strong main_box, @strong window => move |_| {
            main_box.remove(&clock_box);
            main_box.remove(&stopwatch_box);
            main_box.remove(&timer_box);
            main_box.add(&alarm_box);
            window.show_all();
        }));
        


        top_box.set_halign(gtk::Align::End);
        top_box.add(&button_alarm);
        top_box.add(&button_timer);
        top_box.add(&button_stopwatch);
        top_box.add(&button_clock);
        top_box.add(&button_close);
    }







    main_box.add(&top_box);
    main_box.add(&clock_box);
    window.add(&main_box);
    window.show_all();


    glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
    // CLOCK
        clock_label.set_text(&current_time().as_str());

    // STOPWATCH
        stopwatch_label.set_text(stopwatch.lock().unwrap().borrow_mut().get_elapsed().as_str());



        glib::ControlFlow::Continue
    });

}


fn current_time() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))
}

