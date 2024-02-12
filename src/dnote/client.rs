use zbus::proxy;
use zbus::Connection;

// DBus interface proxy for: `org.freedesktop.Notifications`
#[proxy(interface = "org.freedesktop.Notifications", assume_defaults = true)]
trait Notifications {
    /// CloseNotification method
    fn close_notification(&self, id: u32) -> zbus::Result<()>;

    /// GetCapabilities method
    fn get_capabilities(&self) -> zbus::Result<Vec<String>>;

    /// GetServerInformation method
    fn get_server_information(&self) -> zbus::Result<(String, String, String, String)>;

    /// Notify method
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;

    /// ActionInvoked signal
    #[zbus(signal)]
    fn action_invoked(&self, id: u32, action_key: &str) -> zbus::Result<()>;

    /// NotificationClosed signal
    #[zbus(signal)]
    fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;
}

pub async fn notify(
    app_name: &str,
    replaces_id: u32,
    app_icon: &str,
    summary: &str,
    body: &str,
    actions: &[&str],
    hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    expire_timeout: i32,
) {
    let connection = Connection::session().await.expect("");
    let note = NotificationsProxy::new(&connection).await.expect("");
    let reply = note
        .notify(
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        )
        .await
        .expect("");
    dbg!(reply);
}

pub async fn close_notification(id: u32) {
    let connection = Connection::session().await.expect("");
    let note = NotificationsProxy::new(&connection).await.expect("");
    note.close_notification(id).await.expect("");
}
