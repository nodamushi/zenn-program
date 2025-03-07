use std::path::PathBuf;

use anyhow::Result;
use google_drive::{GDrive, GDriveId, InstalledFlowDelegate};

/// ログインの為にブラウザを開く
struct OpenInstalledFlowDelegate;
impl InstalledFlowDelegate for OpenInstalledFlowDelegate {
    fn redirect_uri(&self) -> Option<&str> {
        None
    }

    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> std::pin::Pin<Box<dyn Future<Output = std::result::Result<String, String>> + Send + 'a>>
    {
        println!("URL: {}", url);
        Box::pin(async move {
            open::that(url).map_err(|e| format!("{:?}", e))?;
            Ok("".to_string())
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let tmp:PathBuf = "./tmp".into();
    tokio::fs::create_dir_all(&tmp).await?;
    let drive = GDrive::oauth(
        "client_secret.json",
        "./tmp/token.json",
        Some(Box::new(OpenInstalledFlowDelegate {})),
    )
    .await?;
    let folder_id: GDriveId = "1wS6tVoVmkdMZ96DgcKew-pSYBpJa9pXA".into();

    for meta in drive.list(&folder_id).await? {
        println!("meta: {:?}", meta);
        if !meta.is_google_app() && meta.can_download{ // google app は export をしないといけない
            let path = tmp.join(&meta.name);
            drive.download_and_save(&meta.id, path).await?;
        }
    }

    Ok(())
}
