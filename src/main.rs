/*
 * Author(s): Dylan Turner <dylantdmt@gmail.com>
 * Description:
 * - Entry point for application
 * - For now, most gtk stuff goes here as it's not too complicated
 */

mod pkg;

use std::{
    cmp::Ordering,
    process::Command,
    str::from_utf8
};
use gtk::{
    Application, ApplicationWindow, Dialog,
    Box, ScrolledWindow, Label, Button, Entry, CheckButton,
    Align, PolicyType, Orientation, ResponseType,
    prelude::{
        ApplicationExtManual, ApplicationExt, DialogExt,
        WidgetExt, ContainerExt, BoxExt, ButtonExt, ToggleButtonExt, WidgetExtManual, EntryExt
    }
};
use webkit2gtk::{
    WebView,
    traits::WebViewExt
};
use rust_fuzzy_search::fuzzy_compare;
use reqwest::blocking::get;
use crate::pkg::{
    pull_package_list, get_pkg_manifest, Package
};

const MARGIN: i32 = 5;
const BAR_HEIGHT: i32 = 20;
const MIN_SEARCH_BAR_LEN: i32 = 100;
const ICON_SIZE: i32 = 80;
const BUTTON_SIZE: i32 = 64;
const DIALOG_WIDTH: i32 = 300;
const DIALOG_HEIGHT: i32 = 180;

static mut WIN: Option<ApplicationWindow> = None;
static mut PANEL: Option<Box> = None;

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
        let panel = create_win_content(false, None);
        win.add(&panel);

        unsafe {
            WIN = Some(win.clone());
            PANEL = Some(panel.clone());
        }

        win.show_all();
    });

    app.run();
}

/// Use global state to redo the window creation
fn refresh_window(installed_only: bool, search: Option<String>) {
    let win = unsafe {
        WIN.clone().unwrap()
    };
    let panel = unsafe {
        PANEL.clone().unwrap()
    };
    win.remove(&panel);
    unsafe {
        panel.destroy();
    }

    let new_panel = create_win_content(installed_only, search);
    win.add(&new_panel);
    
    unsafe {
        WIN = Some(win.clone());
        PANEL = Some(new_panel.clone());
    }

    win.show_all();
}

/// Put top bar and package list into a container
fn create_win_content(installed_only: bool, search: Option<String>) -> Box {
    let top_bar = create_top_bar(installed_only, search.clone());
    let pkg_disp = create_pkg_display(installed_only, search.clone());

    let panel = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true).vexpand(true)
        .margin(MARGIN)
        .build();
    panel.pack_start(&top_bar, false, false, 0);
    panel.pack_end(&pkg_disp, true, true, 0);

    panel
}

/// Create a list of packages in a vertically scrolled box
fn create_pkg_display(installed_only: bool, search: Option<String>) -> Box {
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
    let mut online_pkgs = pull_package_list();
    if search.is_none() {
        online_pkgs.sort_by(|a, b| a.name.cmp(&b.name));
    } else {
        online_pkgs.sort_by(|a, b| {
            let a_score = fuzzy_compare(a.name.as_str(), search.clone().unwrap().as_str());
            let b_score = fuzzy_compare(b.name.as_str(), search.clone().unwrap().as_str());
            if a_score > b_score {
                Ordering::Greater
            } else if a_score < b_score {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        online_pkgs.reverse();
    }
    for pkg in online_pkgs {
        if installed_only && !inst_pkgs.iter().any(|e| e.name == pkg.name) {
            continue;
        }
        let pkg_view = create_pkg_view(
            &pkg, inst_pkgs.iter().find(|e| e.name == pkg.name),
            installed_only, search.clone()
        );
        pkg_disp.pack_start(&pkg_view, true, true, MARGIN as u32);
    }

    scroll_cont.add(&pkg_disp);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}

// Turn package data into a graphical view
fn create_pkg_view(
        pkg: &Package, inst_pkg: Option<&Package>,
        installed_only: bool, search: Option<String>) -> Box {
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
        .margin(MARGIN).hexpand(true).vexpand(true)
        .halign(Align::Start)
        .build();

    // Draw the icon and the button we'll use
    let graphic_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin(MARGIN).hexpand(true).vexpand(true)
        .build();

    let pic = WebView::builder().width_request(ICON_SIZE).height_request(ICON_SIZE).build();
    let icon_url = format!(
        "https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/icons/{}.svg",
        pkg.name.clone()
    );
    let resp = get(icon_url.clone()).expect("Failed to get icon!");
    let html = format!(
        "<html>\n<body style='{}'>\n<img src='{}' class='center' />\n<body>\n<html>",
        "background-color:#202040;",
        if resp.status() != 200 {
            "https://raw.githubusercontent.com/blueOkiris/aip-man-pkg-list/main/icons/default.svg"
        } else {
            icon_url.as_str()
        }
    );
    pic.load_html(html.as_str(), None);
    graphic_box.pack_start(&pic, true, true, 0);

    if inst_pkg.is_none() {
        let action_button = Button::builder()
            .label("Install")
            .hexpand(false).vexpand(false)
            .margin_top(MARGIN)
            .width_request(BUTTON_SIZE)
            .build();

        let pkg_name = pkg.name.clone();
        let install_search = search.clone();
        action_button.connect_clicked(move |_| {
            let result = Command::new("aipman")
                .args([ "install", pkg_name.clone().as_str() ])
                .output();
            match result {
                Err(err) => {
                    let dialog = create_status_dialog(
                        format!("Error: {}", err.to_string()).as_str(),
                        installed_only, install_search.clone()
                    );
                    dialog.show_all();
                }, Ok(output) => {
                    let dialog = create_status_dialog(format!(
                        "Command:\naipman install {}\n\nStdout:\n{}\nStderr:\n{}",
                        pkg_name,
                        from_utf8(&output.stdout.as_slice()).unwrap_or("Unknown"),
                        from_utf8(&output.stderr.as_slice()).unwrap_or("Unknown")
                    ).as_str(), installed_only, install_search.clone());
                    dialog.show_all();
                }
            }
        });

        graphic_box.pack_end(&action_button, true, true, 0);
    }
    if inst_pkg.is_some() && inst_pkg.unwrap().upgradable_to(&pkg) {
        let action_button = Button::builder()
            .label("Upgrade")
            .hexpand(false).vexpand(false)
            .margin_top(MARGIN)
            .width_request(BUTTON_SIZE)
            .build();

        let pkg_name = pkg.name.clone();
        let upgrade_search = search.clone();
        action_button.connect_clicked(move |_| {
            let result = Command::new("aipman")
                .args([ "install", pkg_name.clone().as_str() ])
                .output();
            match result {
                Err(err) => {
                    let dialog = create_status_dialog(
                        format!("Error: {}", err.to_string()).as_str(),
                        installed_only, upgrade_search.clone()
                    );
                    dialog.show_all();
                }, Ok(output) => {
                    let dialog = create_status_dialog(format!(
                        "Command:\naipman install {}\n\nStdout:\n{}\nStderr:\n{}",
                        pkg_name,
                        from_utf8(&output.stdout.as_slice()).unwrap_or("Unknown"),
                        from_utf8(&output.stderr.as_slice()).unwrap_or("Unknown")
                    ).as_str(), installed_only, upgrade_search.clone());
                    dialog.show_all();
                }
            }
        });

        graphic_box.pack_end(&action_button, true, true, 0);
    }
    if inst_pkg.is_some() {
        let action_button = Button::builder()
            .label("Remove")
            .hexpand(false).vexpand(false)
            .margin_top(MARGIN)
            .width_request(BUTTON_SIZE)
            .build();

        let pkg_name = pkg.name.clone();
        let remove_search = search.clone();
        action_button.connect_clicked(move |_| {
            let result = Command::new("aipman")
                .args([ "remove", pkg_name.clone().as_str() ])
                .output();
            match result {
                Err(err) => {
                    let dialog = create_status_dialog(
                        format!("Error: {}", err.to_string()).as_str(),
                        installed_only, remove_search.clone()
                    );
                    dialog.show_all();
                }, Ok(output) => {
                    let dialog = create_status_dialog(format!(
                        "Command:\naipman remove {}\n\nStdout:\n{}\nStderr:\n{}",
                        pkg_name,
                        from_utf8(&output.stdout.as_slice()).unwrap_or("Unknown"),
                        from_utf8(&output.stderr.as_slice()).unwrap_or("Unknown")
                    ).as_str(), installed_only, remove_search.clone());
                    dialog.show_all();
                }
            }
        });

        graphic_box.pack_end(&action_button, true, true, 0);
    }   

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

    entry.pack_start(&graphic_box, true, true, MARGIN as u32);
    entry.pack_end(&info_box, true, true, 0);

    scroll_cont.add(&entry);
    cont.pack_start(&scroll_cont, true, true, 0);
    cont
}

/// Create a pop up to show errors and aipman output. Should block rest of app.
fn create_status_dialog(msg: &str, installed_only: bool, search: Option<String>) -> Dialog {
    let dialog = Dialog::builder()
        .title("Status")
        .default_width(DIALOG_WIDTH).default_height(DIALOG_HEIGHT)
        .width_request(DIALOG_WIDTH).height_request(DIALOG_HEIGHT)
        .resizable(false).modal(true)
        .build();
    dialog.add_button("Close", ResponseType::Apply);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic).vscrollbar_policy(PolicyType::Always)
        .hexpand(true).vexpand(true)
        .margin(0)
        .build();

    let status = Label::builder()
        .label(msg).single_line_mode(false)
        .hexpand(true).vexpand(true)
        .margin_end(MARGIN)
        .build();

    dialog.connect_response(move |win, resp| {
        if resp == ResponseType::Apply {
            win.hide();
            refresh_window(installed_only, search.clone());
        }
    });

    scroll.add(&status);
    dialog.content_area().pack_start(&scroll, true, true, 0);
    dialog
}

/// Create a top bar with a search box, installed checkbox, and refresh
fn create_top_bar(installed_only: bool, search: Option<String>) -> Box {
    let cont = Box::builder()
        .hexpand(true).vexpand(false).height_request(BAR_HEIGHT)
        .orientation(Orientation::Horizontal)
        .build();

    let def_search = if search.is_none() {
        String::new()
    } else {
        search.clone().unwrap()
    };
    let search_bar = Entry::builder()
        .hexpand(true).vexpand(false).width_request(MIN_SEARCH_BAR_LEN)
        .margin_end(MARGIN)
        .text(if search.is_some() {
            def_search.as_str()
        } else {
            ""
        }).build();
    search_bar.connect_activate(move |bar| {
        if !bar.text().is_empty() {
            refresh_window(installed_only, Some(String::from(bar.text().as_str())));
        }
    });
    let search_button = Button::builder()
        .label("Search")
        .hexpand(false).vexpand(false)
        .margin_end(MARGIN)
        .build();
    let search_bar_sr_btn = search_bar.clone();
    search_button.connect_clicked(move |_| {
        if !search_bar_sr_btn.text().is_empty() {
            refresh_window(installed_only, Some(String::from(search_bar_sr_btn.text().as_str())));
        }
    });

    let installed = CheckButton::builder()
        .label("Installed")
        .active(installed_only)
        .hexpand(false).vexpand(false)
        .margin_end(MARGIN)
        .build();
    let inst_search = search.clone();
    installed.connect_toggled(move |inst| {
        refresh_window(inst.is_active(), inst_search.clone());
    });

    let refresh_button = Button::builder()
        .label("???")
        .hexpand(false).vexpand(false)
        .build();
    let search_ref_btn = search.clone();
    refresh_button.connect_clicked(move |_| {
        refresh_window(installed_only, search_ref_btn.clone());
    });

    cont.pack_end(&refresh_button, false, false, 0);
    cont.pack_end(&installed, false, false, 0);
    cont.pack_end(&search_button, false, false, 0);
    cont.pack_end(&search_bar, true, true, 0);
    cont
}
