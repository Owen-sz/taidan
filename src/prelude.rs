pub use color_eyre::{
    eyre::{eyre, Context},
    Section,
};
pub use gettextrs::gettext;
pub use itertools::{Either, Itertools};
pub use libhelium::{
    glib::{self, prelude::*},
    prelude::*,
};
pub use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

pub use crate::{NavAction, CFG, SETTINGS};

pub(crate) static REQWEST_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);
