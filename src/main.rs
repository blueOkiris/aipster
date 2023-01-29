/*
 * Author(s): Dylan Turner <dylantdmt@gmail.com>
 * Description:
 * - Entry point for application
 * - For now, most gtk stuff goes here as it's not too complicated
 */

mod pkg;

use gtk::{
    Application, ApplicationWindow,
    Box, ScrolledWindow, Label, Button, Entry, CheckButton,
    Align, PolicyType, Orientation,
    prelude::{
        ApplicationExtManual, ApplicationExt,
        WidgetExt, ContainerExt, BoxExt
    }
};
use pkg::get_pkg_manifest;
use crate::pkg::{
    pull_package_list, Package
};

const MARGIN: i32 = 5;
const BAR_HEIGHT: i32 = 20;
const MIN_SEARCH_BAR_LEN: i32 = 100;

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

        let top_bar = create_top_bar();
        let pkg_disp = create_pkg_display();

        let panel = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true).vexpand(true)
            .margin(MARGIN)
            .build();
        panel.pack_start(&top_bar, false, false, 0);
        panel.pack_end(&pkg_disp, true, true, 0);
        win.add(&panel);

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
    let inst_pkgs = get_pkg_manifest();
    for pkg in pull_package_list() {
        let pkg_view = create_pkg_view(&pkg, inst_pkgs.iter().find(|e| e.name == pkg.name));
        pkg_disp.pack_start(&pkg_view, true, true, MARGIN as u32);
    }

    scroll_cont.add(&pkg_disp);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}

// Turn package data into a graphical view
fn create_pkg_view(pkg: &Package, inst_pkg: Option<&Package>) -> Box {
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
     * |          | Version Installed:              |
     * |__________| Version Available:              |
     * | In/Rm/Up | Url: https://grgsegag...        |
     * |__________|_________________________________|
     */
    let entry = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin(0).hexpand(true).vexpand(true)
        .halign(Align::Start)
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
    name_box.pack_start(&Label::new(Some(&pkg.name)), true, true, 0);
    let desc_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    desc_box.pack_start(&Label::new(Some("Description: ")), true, true, 0);
    desc_box.pack_start(&Label::new(Some(&pkg.description)), true, true, 0);
    let inst_version_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    inst_version_box.pack_start(&Label::new(Some("Installed Version: ")), true, true, 0);
    inst_version_box.pack_start(&Label::new(
        if inst_pkg.is_some() {
            Some(&inst_pkg.unwrap().version)
        } else {
            Some("None")
        }
    ), true, true, 0);
    let version_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    version_box.pack_start(&Label::new(Some("Available Version: ")), true, true, 0);
    version_box.pack_start(&Label::new(Some(&pkg.version)), true, true, 0);
    let url_box = Box::builder()
        .orientation(Orientation::Horizontal).halign(Align::Start)
        .margin(0).hexpand(true).vexpand(true)
        .build();
    url_box.pack_start(&Label::new(Some("Url: ")), true, true, 0);
    url_box.pack_start(&Label::new(Some(&pkg.url)), true, true, 0);

    info_box.pack_start(&name_box, true, true, 0);
    info_box.pack_start(&desc_box, true, true, 0);
    info_box.pack_start(&inst_version_box, true, true, 0);
    info_box.pack_start(&version_box, true, true, 0);
    info_box.pack_start(&url_box, true, true, 0);

    entry.pack_start(&pic, true, true, MARGIN as u32);
    entry.pack_end(&info_box, true, true, 0);

    scroll_cont.add(&entry);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}

/// Create a top bar with a search box, installed checkbox, and refresh
fn create_top_bar() -> Box {
    let cont = Box::builder()
        .hexpand(true).vexpand(false).height_request(BAR_HEIGHT)
        .orientation(Orientation::Horizontal)
        .build();

    let search_bar = Entry::builder()
        .hexpand(true).vexpand(false).width_request(MIN_SEARCH_BAR_LEN)
        .build();
    let search_button = Button::builder()
        .label("Search")
        .hexpand(false).vexpand(false)
        .margin_end(MARGIN)
        .build();

    let installed = CheckButton::builder()
        .label("Installed")
        .hexpand(false).vexpand(false)
        .margin_end(MARGIN)
        .build();

    let refresh_button = Button::builder()
        .label("⟲")
        .hexpand(false).vexpand(false)
        .build();

    cont.pack_end(&refresh_button, false, false, 0);
    cont.pack_end(&installed, false, false, 0);
    cont.pack_end(&search_button, false, false, 0);
    cont.pack_end(&search_bar, true, true, 0);
    cont
}
