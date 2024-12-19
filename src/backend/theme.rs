use crate::prelude::*;

//? https://gitlab.gnome.org/GNOME/gsettings-desktop-schemas/-/merge_requests/63/diffs#65231a5ac1ed586909f5137f1e9bdfe879aaa67b_314_318
#[derive(Clone, Copy, Debug, Default)]
pub enum AccentColor {
    #[default]
    Blue,
    Teal,
    Green,
    Yellow,
    Orange,
    Red,
    Pink,
    Purple,
    Slate,
}

impl From<AccentColor> for &str {
    fn from(value: AccentColor) -> Self {
        match value {
            AccentColor::Blue => "blue",
            AccentColor::Teal => "teal",
            AccentColor::Green => "green",
            AccentColor::Yellow => "yellow",
            AccentColor::Orange => "orange",
            AccentColor::Red => "red",
            AccentColor::Pink => "pink",
            AccentColor::Purple => "purple",
            AccentColor::Slate => "slate",
        }
    }
}

impl AccentColor {
    pub async fn gsettings(self, user: &str, is_dark: bool) -> color_eyre::Result<()> {
        let p = tokio::process::Command::new("su")
            .args([
                user,
                "-c",
                &format!(
                    "gsettings set org.gnome.desktop.interface accent-color {}",
                    <&str>::from(self)
                ),
            ])
            .status()
            .await
            .wrap_err("fail to run `gsettings`")?;
        if !p.success() {
            return Err(eyre!("cannot set accent-color").note(format!("Exit code: {:?}", p.code())));
        }

        let p = tokio::process::Command::new("su")
            .args([
                user,
                "-c",
                &format!(
                    "gsettings set org.gnome.desktop.interface color-scheme {}",
                    if is_dark { "prefer-dark" } else { "default" }
                ),
            ])
            .status()
            .await
            .wrap_err("fail to run `gsettings`")?;
        if !p.success() {
            return Err(eyre!("cannot set color-scheme").note(format!("Exit code: {:?}", p.code())));
        }

        let p = tokio::process::Command::new("su")
            .args([
                user,
                "-c",
                &format!(
                    "gsettings set org.gnome.desktop.interface gtk-theme {}",
                    if is_dark { "Adwaita-dark" } else { "Adwaita" }
                ),
            ])
            .status()
            .await
            .wrap_err("fail to run `gsettings`")?;
        if !p.success() {
            return Err(eyre!("cannot set gtk-theme").note(format!("Exit code: {:?}", p.code())));
        }

        Ok(())
    }

    #[must_use]
    pub fn w3_color_keywords(self) -> &'static str {
        match self {
            Self::Slate => "slategrey",
            x => x.into(),
        }
    }
    pub async fn plasma(self, user: &str, is_dark: bool) -> color_eyre::Result<()> {
        let theme = if is_dark { "BreezeDark" } else { "BreezeLight" };
        let accent = self.w3_color_keywords();
        let p = tokio::process::Command::new("su")
            .args([
                user,
                "-c",
                &format!("plasma-apply-colorscheme {theme} -a {accent}"),
            ])
            .status()
            .await
            .wrap_err("fail to run `plasma-apply-colorscheme`")?;
        if !p.success() {
            return Err(eyre!("`plasma-apply-colorscheme` failed")
                .note(format!("Exit code: {:?}", p.code())));
        }

        Ok(())
    }
}

pub async fn plasma_set_theme_only(user: &str, is_dark: bool) -> color_eyre::Result<()> {
    let theme = if is_dark { "BreezeDark" } else { "BreezeLight" };

    let p = tokio::process::Command::new("su")
        .args([user, "-c", &format!("plasma-apply-colorscheme {theme}")])
        .status()
        .await
        .wrap_err("fail to run `plasma-apply-colorscheme`")?;

    if !p.success() {
        return Err(
            eyre!("`plasma-apply-colorscheme` failed").note(format!("Exit code: {:?}", p.code()))
        );
    }

    Ok(())
}

pub async fn set_theme(
    user: Option<&str>,
    is_dark: bool,
    accent: Option<AccentColor>,
) -> color_eyre::Result<()> {
    let mut tmp = Default::default();
    let user = user.unwrap_or_else(|| {
        tmp = uzers::get_current_username().expect("can't get current username");
        tmp.to_str().unwrap()
    });
    if let Ok(true) = tokio::fs::try_exists("/usr/bin/plasma-apply-colorscheme").await {
        if let Some(accent) = accent {
            accent
                .plasma(user, is_dark)
                .await
                .wrap_err("cannot set accent and theme for plama")?;
        } else {
            plasma_set_theme_only(user, is_dark)
                .await
                .wrap_err("cannot set theme for plasma")?;
        }
    } else if let Ok(true) = tokio::fs::try_exists("/usr/bin/gsettings").await {
        accent
            .unwrap_or_default()
            .gsettings(user, is_dark)
            .await
            .wrap_err("cannot set accent/theme using gsettings")?;
    } else {
        panic!("Neither plasma-apply-colorscheme and gsettings are found in /usr/bin");
    }
    Ok(())
}

pub async fn set_night_light(user: Option<&str>, enabled: bool) -> color_eyre::Result<()> {
    let mut tmp = Default::default();
    let user = user.unwrap_or_else(|| {
        tmp = uzers::get_current_username().expect("can't get current username");
        tmp.to_str().unwrap()
    });
    if let Ok(true) = tokio::fs::try_exists("kwriteconfig6").await {
        let p = tokio::process::Command::new("su").args([user, "-c", &format!("kwriteconfig6 --file ~/.config/kwinrc --group NightColor --key Active --type bool {enabled}")]).status().await.wrap_err("fail to run `kwriteconfig6`")?;
        if !p.success() {
            return Err(eyre!("`kwriteconfig6` failed").note(format!("Exit code: {:?}", p.code())));
        }
    } else {
        let p = tokio::process::Command::new("su").args([user, "-c", &format!("gsettings set org.gnome.settings-daemon.plugins.color night-light-enabled {enabled}")]).status().await.wrap_err("fail to run `gsettings`")?;
        if !p.success() {
            return Err(eyre!("`gsettings` failed").note(format!("Exit code: {:?}", p.code())));
        }
    }
    Ok(())
}
