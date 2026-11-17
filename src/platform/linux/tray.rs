use super::cache;
use crate::app::App;
use crate::platform::{app_id, app_name, Labels};
use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{CheckmarkItem, StandardItem, SubMenu};
use ksni::{Error, Icon, MenuItem, ToolTip, Tray};

pub struct ShutUpTray {
    app: &'static App,
    labels: &'static Labels,
}

impl ShutUpTray {
    fn target_items(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = self
            .app
            .targets()
            .iter()
            .enumerate()
            .map(|(index, target)| {
                CheckmarkItem {
                    label: self.labels.target(&target.name, self.app.is_running(index)),
                    checked: self.app.is_enabled(index),
                    activate: Box::new(move |tray: &mut Self| tray.app.toggle(index)),
                    ..Default::default()
                }
                .into()
            })
            .collect();

        items.push(MenuItem::Separator);
        items.push(
            CheckmarkItem {
                label: self.labels.toggle_all.into(),
                checked: self.app.all_enabled(),
                activate: Box::new(|tray: &mut Self| tray.app.toggle_all()),
                ..Default::default()
            }
            .into(),
        );
        items
    }
}

impl Tray for ShutUpTray {
    fn id(&self) -> String {
        app_id().into()
    }

    fn title(&self) -> String {
        app_name().into()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: app_name().into(),
            ..Default::default()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![
            pixmap(22, include_bytes!(concat!(env!("OUT_DIR"), "/tray-22.argb"))),
            pixmap(32, include_bytes!(concat!(env!("OUT_DIR"), "/tray-32.argb"))),
            pixmap(48, include_bytes!(concat!(env!("OUT_DIR"), "/tray-48.argb"))),
        ]
    }

    fn menu_about_to_show(&mut self) {
        self.app.refresh_running();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            SubMenu {
                label: self.labels.submenu.into(),
                submenu: self.target_items(),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: self.labels.clear_cache.into(),
                activate: Box::new(|tray: &mut Self| cache::ask_and_clear(tray.labels)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: self.labels.quit.into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.app.shutdown();
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn pixmap(size: i32, argb: &[u8]) -> Icon {
    Icon {
        width: size,
        height: size,
        data: argb.to_vec(),
    }
}

pub fn spawn(app: &'static App) -> Result<Handle<ShutUpTray>, Error> {
    ShutUpTray {
        app,
        labels: Box::leak(Box::new(Labels::default())),
    }
    .spawn()
}
