use tray_item::{TrayItem, IconSource};
use std::sync::mpsc;

mod gui;

fn main() {    
    let (tx, rx) = mpsc::sync_channel::<bool>(2);

    let tray = std::thread::spawn(move || {
    gtk::init().unwrap();
        let mut tray = TrayItem::new("Crolk", IconSource::Resource("colors-chromared")).unwrap();
 
        tray.add_menu_item("Hide/Show", move || {
            tx.send(true).unwrap();
        }).unwrap();

        tray.add_menu_item("Quit", || {
            std::process::exit(0);
        }).unwrap();

    gui::main(rx);

    gtk::main();
    });
    tray.join().unwrap();
}