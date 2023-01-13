use gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;

use crate::lbutil;
use crate::ui;
use crate::ui::icons;

impl super::App {
    pub fn open_settings_dialog(&self) {
        let d = gtk::Dialog::builder()
            .transient_for(&self.window)
            .modal(true)
            .default_width(600)
            .default_height(450)
            .resizable(false)
            .title("Settings")
            .build();

        let tabs = gtk::Notebook::builder()
            .tab_pos(gtk::PositionType::Left)
            .show_border(false)
            .build();
        tab(&tabs, "Account", icons::ACCOUNT, &self.acct_settings(&d));
        tab(&tabs, "Usage", icons::USAGE, &self.usage_settings(&d));
        tab(&tabs, "Application", icons::APP, &self.app_settings());

        d.set_child(Some(&tabs));
        d.show();
    }

    fn acct_settings(&self, settings_win: &gtk::Dialog) -> gtk::Box {
        let cntr = settings_box();
        match lbutil::get_account(&self.core) {
            Ok(maybe_acct) => {
                cntr.append(&heading("Info"));
                cntr.append(&acct_info(maybe_acct.as_ref()));
                cntr.append(&separator());
                cntr.append(&heading("Export"));
                cntr.append(&self.acct_export(settings_win));
            }
            Err(err) => {
                let err_lbl = gtk::Label::builder()
                    .label(&err)
                    .halign(gtk::Align::Center)
                    .valign(gtk::Align::Center)
                    .build();
                cntr.append(&err_lbl);
            }
        }
        cntr
    }

    fn acct_export(&self, settings_win: &gtk::Dialog) -> gtk::Box {
        let cntr = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_start(4)
            .margin_bottom(20)
            .build();

        let acct_secret = match self.core.export_account() {
            Ok(v) => v,
            Err(err) => {
                cntr.append(&gtk::Label::new(Some(&format!("{:?}", err)))); //todo
                return cntr;
            }
        };

        let warning = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .label(EXPORT_DESC)
            .use_markup(true)
            .wrap(true)
            .margin_bottom(20)
            .build();

        let btn_copy = ui::clipboard_btn("Copy Key to Clipboard", &acct_secret);
        let btn_show_qr = gtk::Button::builder().label("Show Key as QR Code").build();

        let win = settings_win.clone();
        let core = self.core.clone();
        btn_show_qr.connect_clicked(move |btn_show_qr| {
            let spinner = gtk::Spinner::new();
            spinner.start();

            let loading = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            loading.set_halign(gtk::Align::Center);
            loading.append(&spinner);
            loading.append(&gtk::Label::new(Some("Generating QR...")));

            btn_show_qr.set_child(Some(&loading));

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            let core = core.clone();
            std::thread::spawn(move || {
                let result = core
                    .export_account_qr()
                    .map(std::io::Cursor::new)
                    .map_err(|err| format!("{:?}", err));
                tx.send(result).unwrap();
            });

            let btn_show_qr = btn_show_qr.clone();
            let win = win.clone();
            rx.attach(None, move |png_result| {
                let d = gtk::Dialog::builder()
                    .transient_for(&win)
                    .modal(true)
                    .build();

                match png_result.and_then(|cursor| {
                    Pixbuf::from_read(cursor).map_err(|err| format!("{:?}", err))
                }) {
                    Ok(pixbuf) => {
                        let qr_code = gtk::Image::builder()
                            .width_request(200)
                            .height_request(200)
                            .margin_top(20)
                            .margin_bottom(20)
                            .build();
                        qr_code.set_from_pixbuf(Some(&pixbuf));
                        d.set_child(Some(&qr_code));
                    }
                    Err(err) => {
                        let err_lbl = gtk::Label::builder()
                            .label(&err)
                            .name("err")
                            .halign(gtk::Align::Start)
                            .margin_top(16)
                            .margin_bottom(16)
                            .margin_start(16)
                            .margin_end(16)
                            .build();
                        d.set_child(Some(&err_lbl));
                    }
                }

                btn_show_qr.set_label("Show Key as QR Code");
                d.show();
                glib::Continue(false)
            });
        });

        let buttons = gtk::Grid::builder()
            .column_spacing(4)
            .column_homogeneous(true)
            .build();
        buttons.attach(&btn_copy, 0, 0, 1, 1);
        buttons.attach(&btn_show_qr, 1, 0, 1, 1);

        cntr.append(&warning);
        cntr.append(&buttons);
        cntr
    }

    fn usage_settings(&self, settings_win: &gtk::Dialog) -> gtk::Stack {
        let metrics_result = self.core.get_usage();
        let uncompressed_result = self.core.get_uncompressed_usage();

        let usage = ui::UsageSettings::new();
        usage.set_metrics(metrics_result, uncompressed_result);

        let settings_win = settings_win.clone();
        let core = self.core.clone();
        usage.connect_begin_upgrade(move |usage| {
            let maybe_subscription = match core.get_subscription_info() {
                Ok(maybe_subscription) => maybe_subscription,
                Err(err) => {
                    ui::show_err_dialog(&settings_win, &format!("{:?}", err));
                    return;
                }
            };

            let upgrading = ui::PurchaseFlow::new(maybe_subscription);
            upgrading.connect_cancelled({
                let pages = usage.pages.clone();

                move |upgrading| {
                    pages.set_visible_child_name("home");
                    pages.remove(&upgrading.cntr);
                }
            });
            upgrading.connect_confirmed({
                let core = core.clone();
                let usage = usage.clone();

                move |upgrading, method| {
                    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                    std::thread::spawn({
                        let core = core.clone();

                        move || {
                            let new_tier = lb::StripeAccountTier::Premium(method);
                            let result = core.upgrade_account_stripe(new_tier);
                            tx.send(result).unwrap();
                        }
                    });

                    let payment_ui = gtk::Box::new(gtk::Orientation::Vertical, 12);
                    payment_ui.append(&gtk::Spinner::builder().spinning(true).build());
                    payment_ui.append(&gtk::Label::new(Some("Processing credit card payment...")));

                    upgrading.show_pay_screen(&payment_ui);

                    let btn_finish = gtk::Button::with_label("Finish");
                    btn_finish.set_halign(gtk::Align::Center);
                    btn_finish.connect_clicked({
                        let core = core.clone();
                        let usage = usage.clone();
                        let upgrade_process_cntr = upgrading.cntr.clone();

                        move |_| {
                            let metrics_result = core.get_usage();
                            let uncompressed_result = core.get_uncompressed_usage();
                            usage.set_metrics(metrics_result, uncompressed_result);
                            usage.pages.set_visible_child_name("home");
                            usage.pages.remove(&upgrade_process_cntr);
                        }
                    });

                    let upgrading = upgrading.clone();
                    rx.attach(None, move |switch_tier_result| {
                        ui::clear(&payment_ui);
                        match switch_tier_result {
                            Ok(_) => {
                                upgrading.mark_final_header_section_complete();

                                let check = gtk::Image::from_icon_name("emblem-ok-symbolic");
                                check.set_pixel_size(40);

                                payment_ui.append(&check);
                                payment_ui.append(&gtk::Label::new(Some("Payment complete!")));
                                payment_ui.append(&btn_finish);
                            }
                            Err(err) => {
                                upgrading.set_final_header_icon(icons::ERROR_RED);
                                btn_finish.set_label("Close");
                                let err_msg = payment_err_to_string(err);
                                payment_ui.append(&gtk::Label::new(Some(&err_msg)));
                                payment_ui.append(&btn_finish);
                            }
                        }
                        glib::Continue(false)
                    });
                }
            });
            usage.pages.add_named(&upgrading.cntr, Some("upgrade"));
            usage.pages.set_visible_child_name("upgrade");
        });

        usage.pages
    }

    fn app_settings(&self) -> gtk::Box {
        let cntr = settings_box();
        cntr.append(&heading("General"));
        cntr.append(&self.general_settings());
        cntr.append(&separator());
        cntr.append(&heading("File Tree"));
        cntr.append(&self.filetree_settings());
        cntr
    }

    fn general_settings(&self) -> gtk::Box {
        let s = self.settings.clone();
        let maximize = gtk::CheckButton::with_label("Maximize window on startup");
        maximize.set_active(s.read().unwrap().window_maximize);
        maximize.connect_toggled(move |maximize| {
            s.write().unwrap().window_maximize = maximize.is_active();
        });

        let s = self.settings.clone();
        let open_new_files = gtk::CheckButton::with_label("Open newly created files");
        open_new_files.set_active(s.read().unwrap().open_new_files);
        open_new_files.connect_toggled(move |open_new_files| {
            s.write().unwrap().open_new_files = open_new_files.is_active();
        });

        let s = self.settings.clone();
        let auto_save = gtk::CheckButton::with_label("Auto-save");
        auto_save.set_active(s.read().unwrap().auto_save);
        auto_save.connect_toggled(move |auto_save| {
            s.write().unwrap().auto_save = auto_save.is_active();
        });

        let s = self.settings.clone();
        let auto_sync = gtk::CheckButton::with_label("Auto-sync");
        auto_sync.set_active(s.read().unwrap().auto_sync);
        auto_sync.connect_toggled(move |auto_sync| {
            s.write().unwrap().auto_sync = auto_sync.is_active();
        });

        let general = section();
        general.append(&maximize);
        general.append(&open_new_files);
        general.append(&auto_save);
        general.append(&auto_sync);
        general
    }

    fn filetree_settings(&self) -> gtk::Box {
        let section = section();
        section.append(
            &gtk::Label::builder()
                .label("Show columns:")
                .margin_bottom(4)
                .build(),
        );
        for col in ui::FileTreeCol::removable() {
            let ch = gtk::CheckButton::with_label(&col.name());
            ch.set_active(
                !self
                    .settings
                    .read()
                    .unwrap()
                    .hidden_tree_cols
                    .contains(&col.name()),
            );
            let app = self.clone();
            ch.connect_toggled(move |_| app.tree_toggle_col(col));
            section.append(&ch);
        }
        section
    }
}

fn tab<W: IsA<gtk::Widget>>(tabs: &gtk::Notebook, name: &str, icon_name: &str, stuff: &W) {
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(22);

    let icon_and_name = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_bottom(12)
        .margin_top(12)
        .spacing(9)
        .build();

    icon_and_name.append(&icon);
    icon_and_name.append(&gtk::Label::new(Some(name)));

    let content = gtk::ScrolledWindow::new();
    content.set_child(Some(stuff));

    tabs.append_page(&content, Some(&icon_and_name));
}

fn acct_info(maybe_acct: Option<&lb::Account>) -> gtk::Grid {
    let info = gtk::Grid::builder()
        .column_spacing(16)
        .row_spacing(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    match maybe_acct {
        Some(a) => {
            info.attach(&grid_key("Username: "), 0, 0, 1, 1);
            info.attach(&grid_val(&a.username), 1, 0, 1, 1);
            info.attach(&grid_key("Server: "), 0, 1, 1, 1);
            info.attach(&grid_val(&a.api_url), 1, 1, 1, 1);
        }
        None => info.attach(&grid_key("NO ACCOUNT"), 0, 0, 1, 1),
    }
    info
}

fn settings_box() -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 12);
    b.set_margin_start(12);
    b.set_margin_end(12);
    b
}

fn heading(txt: &str) -> gtk::Label {
    gtk::Label::builder()
        .css_classes(vec!["settings-heading".to_string()])
        .label(txt)
        .halign(gtk::Align::Start)
        .margin_top(12)
        .margin_bottom(6)
        .build()
}

fn section() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Start)
        .margin_start(8)
        .build()
}

fn separator() -> gtk::Separator {
    let s = gtk::Separator::new(gtk::Orientation::Horizontal);
    s.set_margin_top(20);
    s.set_margin_bottom(4);
    s
}

fn grid_key(txt: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(txt)
        .halign(gtk::Align::Start)
        .build()
}

fn grid_val(txt: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(&format!("<b>{}</b>", txt))
        .use_markup(true)
        .halign(gtk::Align::Start)
        .build()
}

fn payment_err_to_string(err: lb::Error<lb::UpgradeAccountStripeError>) -> String {
    use lb::UpgradeAccountStripeError::*;
    match err {
        lb::Error::UiError(err) => match err {
            CouldNotReachServer => "Unable to connect to server.",
            OldCardDoesNotExist => "Could not find your current card.",
            AlreadyPremium => "You are already subscribed for this tier.",
            InvalidCardNumber => "Invalid card number.",
            InvalidCardCvc => "Invalid CVC.",
            InvalidCardExpYear => "Invalid expiration year.",
            InvalidCardExpMonth => "Invalid expiration month.",
            CardDecline => "Your card was declined.",
            CardHasInsufficientFunds => "Your card has insufficient funds.",
            TryAgain => "Please try again.",
            CardNotSupported => "The card you provided is not supported.",
            ExpiredCard => "The card you provided has expired.",
            ClientUpdateRequired => "You are using an out-of-date app. Please upgrade!",
            CurrentUsageIsMoreThanNewTier => {
                "Your current usage is greater than the data cap of your desired subscription tier."
            }
            ExistingRequestPending => {
                "Too many requests. Please wait a little while before trying again."
            }
        }
        .to_string(),
        lb::Error::Unexpected(err) => err,
    }
}

const EXPORT_DESC: &str = "\
Lockbook encrypts your data with a secret key that remains on your devices. \
<b>Whoever has access to this key has complete knowledge and control of your data.</b>

Do not give this key to anyone. Do not display the QR code if there are cameras around.";