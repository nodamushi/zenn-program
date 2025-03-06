use std::{future::poll_fn, path::Path, pin::Pin, sync::Arc};

use google_drive3::{
    DriveHub,
    api::File,
    common::GetToken,
    hyper::{
        self,
        body::{Body, Bytes},
    },
    hyper_rustls::{HttpsConnector, HttpsConnectorBuilder},
    hyper_util::{self, client::legacy::connect::HttpConnector, rt::TokioExecutor},
    yup_oauth2::{
        InstalledFlowAuthenticator, InstalledFlowReturnMethod, authenticator::Authenticator,
        read_application_secret,
    },
};

pub use google_drive3::yup_oauth2::authenticator_delegate::InstalledFlowDelegate;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Google Drive ID
pub struct GDriveId(String);

impl From<String> for GDriveId {
    fn from(v: String) -> Self {
        Self(v)
    }
}
impl From<&str> for GDriveId {
    fn from(v: &str) -> Self {
        Self(v.into())
    }
}
impl std::fmt::Display for GDriveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl AsRef<str> for GDriveId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// --------------------------------------------------------

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("GoogleDrive API Error: {0}")]
    GoogleDriveAPIError(#[from] google_drive3::Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Download Error: {0}")]
    DownloadError(#[from] hyper::Error),
    #[error("Invalid Response: {0}")]
    InvalidResponse(u16),
    #[error("Meta data '{0}' is null.")]
    MetaIsNull(&'static str),
    #[error("Download Error: {0} is a directory")]
    DirectoryDownloadError(GDriveId),
    #[error("Internal Error")]
    InternalError,
}

// --------------------------------------------------------

/// メタデータ
#[derive(Debug, Clone)]
pub struct GMeta {
    pub id: GDriveId,
    pub name: String,
    pub mime_type: String,
    pub modified_time: chrono::DateTime<chrono::offset::Utc>,
    pub can_download: bool,
}

impl GMeta {
    pub fn is_google_app(&self) -> bool {
        self.mime_type.starts_with("application/vnd.google-apps.")
    }

    pub fn is_directory(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }

    pub fn is_google_app_file(&self) -> bool {
        self.is_google_app() && !self.is_directory()
    }

    fn new(file: File) -> Result<Self, Error> {
        let Some(id) = file.id else {
            return Err(Error::MetaIsNull("id"));
        };
        let Some(mime_type) = file.mime_type else {
            return Err(Error::MetaIsNull("mime_type"));
        };
        let Some(name) = file.name else {
            return Err(Error::MetaIsNull("name"));
        };
        let Some(modified_time) = file.modified_time else {
            return Err(Error::MetaIsNull("modifiedTime"));
        };
        let Some(capabilities) = file.capabilities else {
            return Err(Error::MetaIsNull("capabilities"));
        };
        let Some(can_download) = capabilities.can_download else {
            return Err(Error::MetaIsNull("capabilities.canDownload"));
        };
        Ok(GMeta {
            id: id.into(),
            name,
            mime_type,
            modified_time,
            can_download,
        })
    }
}

// --------------------------------------------------------
const READONLY: &str = "https://www.googleapis.com/auth/drive.readonly";
const SCOPES: &[&str] = &[READONLY];

pub trait DownloadHandler {
    fn set_size(&self, size: usize) -> impl Future<Output = Result<(), Error>> + Send;
    fn write(&self, b: Bytes) -> impl Future<Output = Result<(), Error>> + Send;
}


pub struct GDrive(DriveHub<HttpsConnector<HttpConnector>>);

impl GDrive {
    pub async fn oauth<P1, P2>(
        client_secret: P1,
        save_token: P2,
        flow_delegate: Option<Box<dyn InstalledFlowDelegate>>,
    ) -> Result<Self, Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let secret = read_application_secret(client_secret).await?;
        let builder =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(save_token.as_ref());
        let builder = match flow_delegate {
            Some(x) => builder.flow_delegate(x),
            None => builder,
        };
        let auth = builder.build().await?;
        let auth = Box::new(OAuthTokenProvider(auth));
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(
            HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );
        Ok(Self(DriveHub::new(client, auth)))
    }

    pub async fn list(&self, id: &GDriveId) -> Result<Vec<GMeta>, Error> {
        let query = format!("'{}' in parents", id);
        let mut v = Vec::new();
        let mut next = self.list_internal(&query, None, &mut v).await?;
        while next.is_some() {
            next = self.list_internal(&query, None, &mut v).await?;
        }
        Ok(v)
    }

    async fn list_internal(
        &self,
        query: &str,
        next_page_token: Option<String>,
        meta: &mut Vec<GMeta>,
    ) -> Result<Option<String>, Error> {
        let x = self
            .0
            .files()
            .list()
            .q(query)
            .param(
                "fields",
                "nextPageToken, files(id,name,mimeType,modifiedTime,capabilities(canDownload))",
            )
            .include_items_from_all_drives(true)
            .supports_all_drives(true)
            .add_scopes(SCOPES);
        let x = match next_page_token {
            Some(ref y) => x.page_token(y),
            None => x,
        };
        let (rsp, flist) = x.doit().await?;

        if !rsp.status().is_success() {
            return Err(Error::InvalidResponse(rsp.status().as_u16()));
        }
        if let Some(files) = flist.files {
            for file in files {
                meta.push(GMeta::new(file)?);
            }
        }
        Ok(flist.next_page_token)
    }

    pub async fn get_meta(&self, id: &GDriveId) -> Result<GMeta, Error> {
        let (rsp, file) = self
            .0
            .files()
            .get(id.as_ref())
            .param("fields", "id,name,mimeType,modifiedTime,capabilities(canDownload)")
            .supports_all_drives(true)
            .add_scopes(SCOPES)
            .doit()
            .await?;
        if !rsp.status().is_success() {
            return Err(Error::InvalidResponse(rsp.status().as_u16()));
        }
        Ok(GMeta::new(file)?)
    }

    pub async fn download<H>(&self, id: &GDriveId, handler: H) -> Result<(), Error>
    where
        H: DownloadHandler + Sync + Send + 'static,
    {
        let (mut rsp, _file) = self
            .0
            .files()
            .get(id.as_ref())
            .param("alt", "media")
            .supports_all_drives(true)
            .add_scopes(SCOPES)
            .doit()
            .await?;

        if !rsp.status().is_success() {
            return Err(Error::InvalidResponse(rsp.status().as_u16()));
        }
        let hint = rsp.size_hint();
        let size = hint.upper().unwrap_or(hint.lower()) as usize;
        handler.set_size(size).await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        let task = tokio::task::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = handler.write(data).await {
                    return Err(e);
                }
            }
            Ok(())
        });

        while !rsp.is_end_stream() {
            match poll_fn(|cx| {
                let body = Pin::new(rsp.body_mut());
                body.poll_frame(cx)
            })
            .await
            {
                Some(frame) => {
                    let frame = frame?;
                    let data = frame.into_data().unwrap();
                    if let Err(_) = tx.send(data).await {
                        return Err(Error::InternalError);
                    }
                }
                None => break,
            }
        }
        drop(tx);
        task.await.map_err(|_| Error::InternalError)?
    }

    pub async fn download_and_save<P: AsRef<Path>>(
        &self,
        id: &GDriveId,
        file: P,
    ) -> Result<(), Error> {
        let file = file.as_ref();
        if let Some(dir) = file.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        let file = tokio::fs::File::create(file).await?;
        struct X(Arc<tokio::sync::Mutex<tokio::fs::File>>);
        impl DownloadHandler for X {
            fn set_size(&self, _size: usize) -> impl Future<Output = Result<(), Error>> + Send {
                async { Ok(()) }
            }
            fn write(&self, b: Bytes) -> impl Future<Output = Result<(), Error>> + Send {
                let file = self.0.clone();
                async move {
                    use tokio::io::AsyncWriteExt;
                    let mut f = file.lock().await;
                    f.write_all(&b).await?;
                    Ok(())
                }
            }
        }
        let handler = X(Arc::new(tokio::sync::Mutex::new(file)));
        self.download(id, handler).await
    }

    pub async fn download_as_binary(&self, id: &GDriveId) -> Result<Vec<u8>, Error> {
        let data = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        struct X(Arc<tokio::sync::Mutex<Vec<u8>>>);
        impl DownloadHandler for X {
            fn set_size(&self, size: usize) -> impl Future<Output = Result<(), Error>> + Send {
                let data = self.0.clone();
                async move {
                    let mut data = data.lock().await;
                    data.reserve(size);
                    Ok(())
                }
            }

            fn write(&self, b: Bytes) -> impl Future<Output = Result<(), Error>> + Send {
                let data = self.0.clone();
                async move {
                    let mut data = data.lock().await;
                    data.extend_from_slice(&b);
                    Ok(())
                }
            }
        }
        let handler = X(data.clone());
        self.download(id, handler).await?;
        Ok(Arc::try_unwrap(data).unwrap().into_inner())
    }
}

//-----------------------------------------------------------

#[derive(Clone)]
struct OAuthTokenProvider(Authenticator<HttpsConnector<HttpConnector>>);

impl GetToken for Box<OAuthTokenProvider> {
    fn get_token<'a>(
        &'a self,
        scopes: &'a [&str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        Option<String>,
                        Box<dyn std::error::Error + Send + Sync>,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(self.get(scopes))
    }
}

impl OAuthTokenProvider {
    async fn get<'a>(
        &'a self,
        scopes: &'a [&str],
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let x = self.0.token(scopes).await?;
        Ok(x.token().map(|x| x.to_string()))
    }
}
