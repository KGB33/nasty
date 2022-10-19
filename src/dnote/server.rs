use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::{
    dbus_interface,
    zvariant::{DeserializeDict, SerializeDict, Type},
    Connection,
};

#[derive(Serialize, Deserialize)]
struct Notification {
    app_name: String,
    app_icon: String,
    summary: String,
    body: String,
    actions: Vec<String>,
    hints: Hints,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug)]
// `Type` treats `dict` is an alias for `a{sv}`.
#[zvariant(signature = "dict")]
struct Hints {
    #[zvariant(rename = "action-icons")]
    action_icons: Option<bool>,
    categary: Option<String>,
    #[zvariant(rename = "desktop-entry")]
    desktop_entry: Option<String>,
    //image_data: iiibiiay // not implementing this
    #[zvariant(rename = "image-path")]
    image_path: Option<String>,
    resident: Option<bool>,
    transient: Option<bool>,
    urgency: Option<u8>,
}

#[derive(Serialize, Deserialize)]
struct Notes {
    notifications: HashMap<u32, Notification>,
    priority: Vec<u32>,
    last_id: u32,
}

impl Notes {
    fn new() -> Notes {
        Notes {
            notifications: HashMap::new(),
            priority: Vec::new(),
            last_id: 1,
        }
    }
    fn next_id(&mut self) -> u32 {
        self.last_id += 1;
        return self.last_id;
    }
    fn on_change(&mut self) {
        self.update_urgency();
        let out = serde_json::to_string(self).expect("");
        println!("{}", out);
    }
    fn update_urgency(&mut self) {
        self.priority = self.notifications.keys().copied().collect();
        self.priority.sort_by_key(|k| {
            (
                10 - self.notifications.get(k).unwrap().hints.urgency.unwrap(),
                k.clone(),
            )
        });
    }
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl Notes {
    // CloseNotification method
    async fn close_notification(&mut self, id: u32) {
        self.notifications.remove_entry(&id);
        self.on_change();
    }

    // GetCapabilities method
    async fn get_capabilities(&self) -> zbus::fdo::Result<Vec<String>> {
        // uncomment lines as they're implemented
        Ok(Vec::from([
            // String::from("actions-icons"),
            // String::from("actions"),
            String::from("body"),
            // String::from("body-hyperlinks"),
            // String::from("body-markup"),
            // String::from("icon-multi"),
            // String::from("icon-static"),
            // String::from("persistence"),
            // String::from("sound"),
        ]))
    }

    // GetServerInformation method
    #[dbus_interface(out_args("name", "vendor", "version", "spec_version"))]
    async fn get_server_information(&self) -> zbus::fdo::Result<(&str, &str, &str, &str)> {
        return Ok(("dnote", "kgb33", "v0.0.0", "1.2"));
    }

    // Notify method
    async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: Hints,
        _expire_timeout: i32, // TODO
    ) -> u32 {
        let mut replaces_id = replaces_id;
        if replaces_id == 0 {
            replaces_id = self.next_id();
        }
        self.notifications.insert(
            replaces_id,
            Notification {
                app_name,
                app_icon,
                summary,
                body,
                actions,
                hints,
            },
        );
        self.on_change();
        return replaces_id;
    }

    // ActionInvoked signal
    //#[dbus_proxy(signal)]
    //fn action_invoked(&self, id: u32, action_key: &str) -> zbus::Result<()>;

    // NotificationClosed signal
    async fn notification_closed(&mut self, id: u32, _reason: u32) {
        self.notifications.remove(&id);
        self.on_change();
    }
}

pub async fn serve() {
    let connection = Connection::session().await.expect("");
    // setup the server
    connection
        .object_server()
        .at("/org/freedesktop/Notifications", Notes::new())
        .await
        .expect("");
    // before requesting the name
    connection
        .request_name("org.freedesktop.Notifications")
        .await
        .expect("");

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
