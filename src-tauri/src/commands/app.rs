use serde::Serialize;

use crate::commands::{CommandResponse, CommandResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub app_name: &'static str,
    pub version: &'static str,
    pub runtime_network_enabled: bool,
    pub network_policy: &'static str,
}

#[tauri::command]
pub async fn get_app_status() -> CommandResult<AppStatus> {
    Ok(CommandResponse::new(AppStatus {
        app_name: "Filnizer",
        version: env!("CARGO_PKG_VERSION"),
        runtime_network_enabled: true,
        network_policy: "Network is disabled by default except user-confirmed LibreOffice installation through winget.",
    }))
}

#[cfg(test)]
mod tests {
    use super::get_app_status;

    #[tokio::test]
    async fn app_status_declares_confirmed_install_network_policy() {
        let response = get_app_status().await.unwrap();

        assert!(response.data.runtime_network_enabled);
        assert_eq!(response.data.app_name, "Filnizer");
        assert!(response.data.network_policy.contains("user-confirmed"));
    }
}
