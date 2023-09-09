use tray_item::{TrayItem, IconSource};
mod gui;

fn main() {
    let tray = std::thread::spawn(move || {
    gtk::init().unwrap();
        let mut tray = TrayItem::new("Crolk", IconSource::Resource("accessories-")).unwrap();
 
        tray.add_menu_item("Hide/Show", || {
            gui::main();
        }).unwrap();

        tray.add_menu_item("Quit", || {
            std::process::exit(0);
        }).unwrap();

        gui::main();
    gtk::main();
    });
    tray.join().unwrap();
}