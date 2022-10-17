use zbus::{dbus_interface, Connection};

struct Notes;

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl Notes {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }
    // CloseNotification method
    //async fn close_notification(&self, id: u32) -> zbus::Result<()>;

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
      &self,
      app_name: &str,
      replaces_id: u32,
      app_icon: &str,
      summary: &str,
      body: &str,
      actions: Vec<&str>,
      hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
      expire_timeout: i32,
    ) -> u32 {
        println!("notify called:
                 app_name: {app_name} 
                 replaces_id: {replaces_id}
                 app_icon: {app_icon}
                 summary: {summary}
                 body: {body}
                 actions: {actions:?}
                 hints: {hints:?}
                 expire_timeout: {expire_timeout}
                 ");
        return replaces_id; 
    }

    // ActionInvoked signal
    //#[dbus_proxy(signal)]
    //fn action_invoked(&self, id: u32, action_key: &str) -> zbus::Result<()>;

    // NotificationClosed signal
    //#[dbus_proxy(signal)]
    //fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;

}

pub async fn serve() {
    let connection = Connection::session().await.expect("");
    // setup the server
    connection
        .object_server()
        .at("/org/freedesktop/Notifications", Notes)
        .await.expect("");
    // before requesting the name
    connection
        .request_name("org.freedesktop.Notifications")
        .await.expect("");

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
