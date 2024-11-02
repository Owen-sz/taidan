crate::generate_page!(NightLight {
    pub setting: bool
}:
    update(self, message, sender) {
        Switch(state: bool) => self.setting = state,
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
            set_label: &gettext("Night Light"),
            add_css_class: "view-subtitle",
            inline_css: "font-weight: bold",
        },

        gtk::Label {
            set_use_markup: true,
            set_label: &gettext("Tint the display with a warm tone at night to reduce eyestrain."),
        },

        gtk::Label {
            set_use_markup: true,
            set_label: &gettext("Night Light is not proven to help with difficulty falling sleep."),
        },

        // gtk::Label {
        //     set_use_markup: true,
        //     set_label: &gettext("(Night Light is also known as Night Color on KDE Plasma.)"),
        // },

        #[template] crate::ui::SwitchBox {
            set_title: &gettext("Night Light"),
            #[template_child] switch {
                connect_state_set[sender] => move |_, state| {
                    sender.input(Self::Input::Switch(state));
                    glib::Propagation::Proceed
                },
            }
        },
    },

    #[template] crate::ui::PrevNextBtns {
        #[template_child] prev {
            connect_clicked => Self::Input::Nav(NavAction::Back),
        },
        #[template_child] next {
            connect_clicked => Self::Input::Nav(NavAction::Next),
        },
    }
);
