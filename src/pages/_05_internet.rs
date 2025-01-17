crate::generate_page!(Internet {
    btn_next: libhelium::Button,
}:
    init(root, sender, model, widgets) {
        let sender1 = sender.clone();
        sender.oneshot_command(async move { check_online(sender1).await });
        model.btn_next = widgets.prev_next_btns.next.clone();
    }
    update(self, message, sender) {
        IsOnline => self.btn_next.set_sensitive(true),
    } => {}

    gtk::Box {
        set_orientation: gtk::Orientation::Vertical,
        set_spacing: 16,
        set_margin_horizontal: 80,
        set_vexpand: true,
        set_hexpand: true,
        set_valign: gtk::Align::Center,
        set_halign: gtk::Align::Fill,

        gtk::Image {
            set_icon_name: Some("dialog-question-symbolic"),
            inline_css: "-gtk-icon-size: 64px",
        },

        gtk::Label {
            set_label: &gettext("Let's Get You Online"),
            add_css_class: "view-subtitle",
            inline_css: "font-weight: bold",
        },

        gtk::Label {
            set_label: &gettext("Connect to the Internet to get the latest and greatest."),
        },

        libhelium::Button {
            //set_is_pill: true,
            set_halign: gtk::Align::Center,
            set_hexpand: false,
            #[watch]
            set_label: &gettext("I don't have Internet"),
            inline_css: "padding-left: 48px; padding-right: 48px",
            connect_clicked[sender] => move |_| {
                SETTINGS.write().nointernet = true;
                sender.input(Self::Input::Nav(NavAction::Next));
            },
        }

    },

    #[name(prev_next_btns)]
    #[template] crate::ui::PrevNextBtns {
        #[template_child] prev {
            connect_clicked => Self::Input::Nav(NavAction::Back),
        },
        #[template_child] next {
            set_sensitive: false,
            connect_clicked[sender] => move |_| {
                SETTINGS.write().nointernet = false;
                sender.input(Self::Input::Nav(NavAction::Next));
            },
        },
    }
);

async fn check_online(sender: ComponentSender<InternetPage>) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(crate::APPID)
        .build()
        .unwrap();
    let arch = std::env::consts::ARCH;
    let edition = &CFG.edition;
    loop {
        if let Ok(202) = client
            .post("https://plausible.fyralabs.com/api/event")
            .header(
                "Content-Type",
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .body(format!(
                r#"
                {{
                    "name": "pageview",
                    "url": "app://internet/{arch}/{edition}",
                    "domain": "taidan",
                    "props": {{
                        "arch": "{arch}",
                        "edition": "{edition}"
                    }}
                }}
                "#
            ))
            .send()
            .await
            .map(|r| r.status().as_u16())
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    sender.input(InternetPageMsg::IsOnline);
}
