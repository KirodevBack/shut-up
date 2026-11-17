use super::cache;
use crate::app::App;
use crate::platform::{app_name, Labels};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject, Sel};
use objc2::{define_class, msg_send, sel, AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSAlertStyle, NSApplication, NSApplicationActivationPolicy,
    NSControlStateValueOff, NSControlStateValueOn, NSImage, NSMenu, NSMenuDelegate, NSMenuItem,
    NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSData, NSObject, NSObjectProtocol, NSSize, NSString};
use std::cell::OnceCell;

pub struct Ivars {
    app: &'static App,
    labels: Labels,
    status_item: OnceCell<Retained<NSStatusItem>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "ShutUpMenuController"]
    #[ivars = Ivars]
    pub struct MenuController;

    impl MenuController {
        #[unsafe(method(toggleTarget:))]
        fn toggle_target(&self, sender: &NSMenuItem) {
            let names = self.ivars().app.toggle(sender.tag() as usize);
            self.warn_not_running(&names);
        }

        #[unsafe(method(toggleAll:))]
        fn toggle_all(&self, _sender: &NSMenuItem) {
            let names = self.ivars().app.toggle_all();
            self.warn_not_running(&names);
        }

        #[unsafe(method(clearCache:))]
        fn clear_cache(&self, _sender: &NSMenuItem) {
            if self.confirm_clear_cache() {
                cache::clear(&self.ivars().labels);
            }
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: &NSMenuItem) {
            self.ivars().app.shutdown();
            std::process::exit(0);
        }
    }

    unsafe impl NSObjectProtocol for MenuController {}

    unsafe impl NSMenuDelegate for MenuController {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            menu.removeAllItems();
            self.fill_menu(menu);
        }
    }
);

impl MenuController {
    fn new(app: &'static App, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars {
            app,
            labels: Labels::default(),
            status_item: OnceCell::new(),
        });
        unsafe { msg_send![super(this), init] }
    }

    fn mtm(&self) -> MainThreadMarker {
        MainThreadMarker::from(self)
    }

    fn item(&self, title: &str, action: Option<Sel>) -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(self.mtm()),
                &NSString::from_str(title),
                action,
                &NSString::new(),
            )
        };
        if action.is_some() {
            let target: &AnyObject = self.as_ref();
            unsafe { item.setTarget(Some(target)) };
        }
        item
    }

    fn checked_item(&self, title: &str, action: Sel, checked: bool) -> Retained<NSMenuItem> {
        let item = self.item(title, Some(action));
        item.setState(if checked { NSControlStateValueOn } else { NSControlStateValueOff });
        item
    }

    fn fill_menu(&self, root: &NSMenu) {
        let mtm = self.mtm();
        let ivars = self.ivars();
        let app = ivars.app;

        let targets = NSMenu::new(mtm);
        for (index, target) in app.targets().iter().enumerate() {
            let item = self.checked_item(&target.name, sel!(toggleTarget:), app.is_enabled(index));
            item.setTag(index as isize);
            targets.addItem(&item);
        }
        targets.addItem(&NSMenuItem::separatorItem(mtm));
        let all = self.checked_item(ivars.labels.toggle_all, sel!(toggleAll:), app.all_enabled());
        targets.addItem(&all);

        let submenu = self.item(ivars.labels.submenu, None);
        submenu.setSubmenu(Some(&targets));

        root.addItem(&submenu);
        root.addItem(&self.item(ivars.labels.clear_cache, Some(sel!(clearCache:))));
        root.addItem(&NSMenuItem::separatorItem(mtm));
        root.addItem(&self.item(ivars.labels.quit, Some(sel!(quit:))));
    }

    fn warn_not_running(&self, names: &[&str]) {
        let Some(text) = self.ivars().labels.not_running_message(names) else {
            return;
        };
        let mtm = self.mtm();
        NSApplication::sharedApplication(mtm).activate();
        let alert = NSAlert::new(mtm);
        alert.setAlertStyle(NSAlertStyle::Warning);
        alert.setMessageText(&NSString::from_str(app_name()));
        alert.setInformativeText(&NSString::from_str(&text));
        alert.runModal();
    }

    fn confirm_clear_cache(&self) -> bool {
        let mtm = self.mtm();
        NSApplication::sharedApplication(mtm).activate();
        let alert = NSAlert::new(mtm);
        alert.setAlertStyle(NSAlertStyle::Warning);
        alert.setMessageText(&NSString::from_str(self.ivars().labels.clear_cache));
        alert.setInformativeText(&NSString::from_str(self.ivars().labels.clear_cache_warning));
        alert.addButtonWithTitle(&NSString::from_str(self.ivars().labels.continue_button));
        alert.addButtonWithTitle(&NSString::from_str(self.ivars().labels.cancel_button));
        alert.runModal() == NSAlertFirstButtonReturn
    }

    fn install_status_item(&self) {
        let mtm = self.mtm();
        let status_item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);

        if let Some(button) = status_item.button(mtm) {
            let logo = NSData::with_bytes(include_bytes!("../../../assets/shut-it-up.png"));
            if let Some(image) = NSImage::initWithData(NSImage::alloc(), &logo) {
                image.setSize(NSSize::new(18.0, 18.0));
                button.setImage(Some(&image));
            }
            button.setToolTip(Some(&NSString::from_str(app_name())));
        }

        let menu = NSMenu::new(mtm);
        menu.setDelegate(Some(ProtocolObject::from_ref(self)));
        self.fill_menu(&menu);
        status_item.setMenu(Some(&menu));
        let _ = self.ivars().status_item.set(status_item);
    }
}

pub fn run(app: &'static App) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let application = NSApplication::sharedApplication(mtm);
    application.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let controller = MenuController::new(app, mtm);
    controller.install_status_item();
    application.run();
}
