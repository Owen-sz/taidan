#![warn(rust_2018_idioms)]

pub mod backend;
pub mod cfg;
pub mod macros;
pub mod pages;
pub mod prelude;
pub mod ui;

use crate::prelude::*;
use gtk::glib::translate::FromGlibPtrNone;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, RelmApp, SimpleComponent,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const APPID: &str = "com.FyraLabs.Taidan";

// configuration of the distro OOBE.
pub static CFG: std::sync::LazyLock<cfg::Config> = std::sync::LazyLock::new(|| {
    tracing::debug!("Initializing cfg::Config");
    let mut cfg = cfg::Config::default();
    cfg.populate();
    tracing::debug!("Populated cfg::Config (turn on `trace` to see body)");
    cfg
});

pub static SETTINGS: relm4::SharedState<backend::settings::Settings> = relm4::SharedState::new();

generate_pages!(Page AppModel AppMsg:
    00: Welcome,
    01: WhoAreYou,
    02: Password,
    03: Internet,
    04: Analytics,
    05: CrashReport,
    06: Location,
    07: NightLight,
    08: Theme,
    09: Browser,
    10: Categories,
    11: Installing,
);

#[derive(Debug)]
pub enum NavAction {
    GoTo(Page),
    Next,
    Back,
    Quit,
}

#[derive(Debug)]
pub enum AppMsg {
    Nav(NavAction),
}

#[allow(clippy::str_to_string)]
#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();

    type Input = AppMsg;
    type Output = ();

    view! {
        libhelium::ApplicationWindow {
            set_title: Some(&gettext("Welcome to %s").replace("%s", &CFG.distro)),
            set_default_width: 600,
            set_default_height: 500,
            set_vexpand: true,
            set_align: gtk::Align::Fill,

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_vexpand: true,
                set_align: gtk::Align::Fill,
                set_orientation: gtk::Orientation::Vertical,

                #[transition = "SlideLeftRight"]
                #[name = "stack"]
                match model.page {
                    Page::Welcome => *model.welcome_page.widget(),
                    Page::WhoAreYou => *model.who_are_you_page.widget(),
                    Page::Password => *model.password_page.widget(),
                    Page::Internet => *model.internet_page.widget(),
                    Page::Analytics => *model.analytics_page.widget(),
                    Page::CrashReport => *model.crash_report_page.widget(),
                    Page::Location => *model.location_page.widget(),
                    Page::NightLight => *model.night_light_page.widget(),
                    Page::Theme => *model.theme_page.widget(),
                    Page::Browser => *model.browser_page.widget(),
                    Page::Categories => *model.categories_page.widget(),
                    Page::Installing => *model.installing_page.widget(),
                }
            }
        }
    }

    // Initialize the UI.
    fn init(
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // TODO: make libhelium force this
        let display = gtk::gdk::Display::default().unwrap();
        let settings = gtk::Settings::for_display(&display);
        settings.set_gtk_icon_theme_name(Some("Hydrogen"));
        gtk::gio::resources_register_include!("icons.gresource").unwrap();
        let theme = gtk::IconTheme::for_display(&display);
        theme.add_resource_path("/com/FyraLabs/Taidan/icons/symbolic/actions");

        let model = Self::_default(sender);

        let widgets = view_output!();

        widgets.stack.set_vexpand(true);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        tracing::trace!(?message, "AppModel: Received message");
        match message {
            AppMsg::Nav(NavAction::Next)
                if self.page == Page::Password && SETTINGS.read().skipconfig =>
            {
                self.page = Page::Installing;
                relm4::spawn(backend::start_install(
                    SETTINGS.read().clone(),
                    self.installing_page.sender().clone(),
                ));
            }
            AppMsg::Nav(NavAction::Next) if usize::from(self.page) == 3 => {
                tracing::trace!("Skipping to page 7 after Page::Internet");
                self.page = 7.try_into().expect("No page 7!");
            }
            AppMsg::Nav(NavAction::Back) if usize::from(self.page) == 7 => {
                tracing::trace!("Skipping to page 3");
                self.page = 3.try_into().expect("No page 3!");
            }
            AppMsg::Nav(NavAction::GoTo(page)) => self.page = page,
            AppMsg::Nav(NavAction::Quit) => std::process::exit(0),
            AppMsg::Nav(NavAction::Next) => {
                self.page = usize::from(self.page)
                    .wrapping_add(1)
                    .try_into()
                    .expect("No next page!");
                if self.page == Page::Installing {
                    relm4::spawn(backend::start_install(
                        SETTINGS.read().clone(),
                        self.installing_page.sender().clone(),
                    ));
                }
            }
            AppMsg::Nav(NavAction::Back) => {
                self.page = usize::from(self.page)
                    .wrapping_sub(1)
                    .try_into()
                    .expect("No prev page!");
            }
        }
    }
}

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
fn main() -> std::io::Result<()> {
    let _guard = setup_logs_and_install_panic_hook();

    gettextrs::textdomain(APPID)?;
    gettextrs::bind_textdomain_codeset(APPID, "UTF-8")?;

    let app = libhelium::Application::builder()
        .application_id(APPID)
        .flags(libhelium::gtk::gio::ApplicationFlags::default())
        // SAFETY: placeholder
        .default_accent_color(unsafe {
            &libhelium::RGBColor::from_glib_none(std::ptr::from_mut(
                &mut libhelium::ffi::HeRGBColor {
                    r: 0.0,
                    g: 7.0,
                    b: 143.0,
                },
            ))
        })
        .build();

    tracing::debug!("Starting Taidan");
    RelmApp::from_app(app).run::<AppModel>(());
    Ok(())
}

/// Returns a logging guard.
///
/// # Panics
/// - cannot install `color_eyre`
/// - cannot create readymade tempdir
#[allow(clippy::cognitive_complexity)]
fn setup_logs_and_install_panic_hook() -> impl std::any::Any {
    color_eyre::install().expect("install color_eyre");
    let temp_dir = tempfile::Builder::new()
        .prefix("taidan-logs")
        .tempdir()
        .expect("create logs tempdir")
        .into_path();
    // create dir
    std::fs::create_dir_all(&temp_dir).expect("create logs tempdir");
    let file_appender = tracing_appender::rolling::never(&temp_dir, "taidan.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_env("TAIDAN_LOG"))
        .with(
            tracing_journald::layer()
                .unwrap()
                .with_syslog_identifier("taidan".to_owned()),
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .compact(),
        )
        .init();

    if cfg!(debug_assertions) {
        tracing::info!("Running in debug mode");
    }
    tracing::info!("Taidan {version}", version = env!("CARGO_PKG_VERSION"));
    tracing::info!("Logging to journald");
    tracing::info!(
        "Logging to {tmp}/taidan.log",
        tmp = temp_dir.to_string_lossy()
    );
    guard
}
