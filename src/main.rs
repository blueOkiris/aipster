/*
 * Author(s): Dylan Turner <dylantdmt@gmail.com>
 * Description: Entry point for application
 */

mod pkg;

use gtk::{
    Application, ApplicationWindow,
    Box, ScrolledWindow, Label, Button,
    Align, PolicyType, Orientation,
    prelude::{
        ApplicationExtManual, ApplicationExt,
        WidgetExt, ContainerExt, BoxExt
    }, Adjustment
};
use crate::pkg::{
    pull_package_list, Package
};

const MARGIN: i32 = 5;

fn main() {
    let app = Application::builder()
        .application_id("blueOkiris.github.Aipster")
        .build();

    app.connect_activate(|app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("Aipster")
            .default_width(800)
            .default_height(600)
            .expand(true).hexpand(true).vexpand(true)
            .build();

        let pkg_disp = create_pkg_display();
        win.add(&pkg_disp);

        win.show_all();
    });

    app.run();
}

/// Create a list of packages in a vertically scrolled box
fn create_pkg_display() -> Box {
    // Main, expanding container
    let cont = Box::builder()
        .margin(0).hexpand(true).vexpand(true)
        .build();

    // A scrollbar bc the item count will be large
    let scroll_cont = ScrolledWindow::builder()
        .margin(MARGIN).hexpand(true).vexpand(true)
        .hscrollbar_policy(PolicyType::Never).vscrollbar_policy(PolicyType::Always)
        .build();

    // An inner container for all the items that will be scollable
    let pkg_disp = Box::builder()
        .orientation(Orientation::Vertical)
        .margin(0).hexpand(true).vexpand(true)
        .build();

    // Add each package thing to the main vbox
    for pkg in pull_package_list() {
        let pkg_view = create_pkg_view(&pkg);
        pkg_disp.pack_start(&pkg_view, true, true, MARGIN as u32);
    }

    scroll_cont.add(&pkg_disp);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}

// Turn package data into a graphical view
fn create_pkg_view(pkg: &Package) -> Box {
    // Main container of each item
    let cont = Box::builder()
        .margin(0).hexpand(true).vexpand(true)
        .build();

    // Hor scroll for long desc
    let scroll_cont = ScrolledWindow::builder()
        .margin(MARGIN).hexpand(true).vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic).vscrollbar_policy(PolicyType::Never)
        .build();
    
    /*
     * Actual container of info
     *
     * ----------------------------------------------
     * |          | Name: name                      |
     * |   Image  | Description: blah blah blah ... |
     * |          | ...                             |
     * |__________|_________________________________|
     */
    let entry = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin(0).hexpand(true).vexpand(true)
        .build();

    let pic = Label::new(Some("PIC HERE"));
    
    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin(0).hexpand(true).vexpand(true)
        .build();

    let name_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    name_box.pack_start(&Label::new(Some("Name: ")), true, true, 0);
    name_box.pack_start(&Label::new(Some(&pkg.name.clone())), true, true, 0);
    let desc_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    desc_box.pack_start(&Label::new(Some("Description: ")), true, true, 0);
    desc_box.pack_start(&Label::new(Some(&pkg.description.clone())), true, true, 0);

    info_box.pack_start(&name_box, true, true, 0);
    info_box.pack_start(&desc_box, true, true, 0);

    entry.pack_start(&pic, true, true, 0);
    entry.pack_start(&info_box, true, true, 0);

    scroll_cont.add(&entry);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}
