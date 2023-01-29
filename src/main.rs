/*
 * Author(s): Dylan Turner <dylantdmt@gmail.com>
 * Description: Entry point for application
 */

use gtk::{
    Application, ApplicationWindow,
    prelude::{
        ApplicationExtManual, ApplicationExt, WidgetExt
    }
};

mod pkg;

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
            .build();

        win.show_all();
    });

    app.run();
}
