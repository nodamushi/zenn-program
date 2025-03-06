//! # Google Drive API Rustクライアント
//!
//! このクレートはGoogle Drive APIを使用するためのシンプルなRustラッパーを提供します。
//! 認証、ファイルの一覧取得、メタデータの取得、ファイルのダウンロードなどの
//! 基本的な機能をサポートしています。
//!
//! ## 機能
//!
//! - OAuth2認証プロセスをサポート
//! - ファイルやフォルダの一覧取得
//! - ファイルのメタデータを取得
//! - ファイルのダウンロード（バイナリまたはファイルとして保存）
//!
//! ## 使用例
//!
//! ```skip
//! use google_drive::{GDrive, GDriveId};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 認証を行う
//!     let drive = GDrive::oauth(
//!         "client_secret.json",
//!         "./token.json",
//!         None,
//!     ).await?;
//!
//!     // フォルダIDを指定してファイル一覧を取得
//!     let folder_id: GDriveId = "your_folder_id".into();
//!     let files = drive.list(&folder_id).await?;
//!
//!     // ファイルをダウンロード
//!     for meta in files {
//!         if !meta.is_directory() && meta.can_download {
//!             drive.download_and_save(&meta.id, format!("./downloads/{}", meta.name)).await?;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

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

/// Google DriveのIDを表す構造体
///
/// この構造体はGoogle Driveのファイルやフォルダを識別するためのIDをラップします。
/// StringやstrからIDを作成することができ、表示や比較のための実装が提供されています。
///
/// # 例
///
/// ```
/// use google_drive::GDriveId;
///
/// let id1: GDriveId = "1wS6tVoVmkdMZ96DgcKew-pSYBpJa9pXA".into();
/// let id2 = GDriveId::from("1wS6tVoVmkdMZ96DgcKew-pSYBpJa9pXA");
///
/// assert_eq!(id1, id2);
/// println!("ID: {}", id1); // ID: 1wS6tVoVmkdMZ96DgcKew-pSYBpJa9pXA
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

/// Google Driveの操作時に発生する可能性のあるエラー
///
/// このエラー型はGoogle Drive APIの使用中に発生する様々なエラーを表します。
/// API関連のエラー、IO操作のエラー、ダウンロードエラー、およびその他の問題を含みます。
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Google Drive API自体からのエラー
    #[error("GoogleDrive API Error: {0}")]
    GoogleDriveAPIError(#[from] google_drive3::Error),

    /// ファイルシステム操作に関連するエラー
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    /// ファイルダウンロード中のネットワークエラー
    #[error("Download Error: {0}")]
    DownloadError(#[from] hyper::Error),

    /// 無効なHTTPレスポンスステータスを受け取った
    #[error("Invalid Response: {0}")]
    InvalidResponse(u16),

    /// 必要なメタデータフィールドがnullだった
    #[error("Meta data '{0}' is null.")]
    MetaIsNull(&'static str),

    /// ディレクトリをファイルとしてダウンロードしようとした
    #[error("Download Error: {0} is a directory")]
    DirectoryDownloadError(GDriveId),

    /// 内部エラー
    #[error("Internal Error")]
    InternalError,
}

// --------------------------------------------------------

/// Google Driveファイルのメタデータ
///
/// この構造体はGoogle Driveのファイルやフォルダに関するメタデータを表します。
/// IDや名前、MIMEタイプ、変更日時などの基本的な情報が含まれます。
#[derive(Debug, Clone)]
pub struct GMeta {
    /// ファイルやフォルダのID
    pub id: GDriveId,

    /// ファイルやフォルダの名前
    pub name: String,

    /// MIMEタイプ
    pub mime_type: String,

    /// 最終変更日時（UTC）
    pub modified_time: chrono::DateTime<chrono::offset::Utc>,

    /// ダウンロード可能かどうか
    pub can_download: bool,
}

impl GMeta {
    /// このファイルがGoogle Apps形式（Googleドキュメント、スプレッドシートなど）かどうかを判定
    ///
    /// Google Apps形式のファイルは特別な処理が必要で、通常のダウンロードでは取得できません。
    /// エクスポート機能を使用する必要があります。
    pub fn is_google_app(&self) -> bool {
        self.mime_type.starts_with("application/vnd.google-apps.")
    }

    /// このアイテムがディレクトリ（フォルダ）かどうかを判定
    pub fn is_directory(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }

    /// このファイルがGoogle Appsファイル（ディレクトリ以外）かどうかを判定
    pub fn is_google_app_file(&self) -> bool {
        self.is_google_app() && !self.is_directory()
    }

    /// Google Drive APIのFileオブジェクトからGMetaを作成
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

/// ファイルのダウンロード処理を定義するトレイト
///
/// このトレイトを実装することで、ダウンロードしたファイルのデータの処理方法を
/// カスタマイズすることができます。例えば、ファイルに保存したり、メモリに保持したり、
/// ストリーミング処理したりできます。
pub trait DownloadHandler {
    /// ダウンロードするファイルのサイズを設定
    ///
    /// このメソッドはダウンロードの開始時に呼び出され、ファイルの全体サイズを通知します。
    /// 必要に応じてバッファのプリアロケーションなどに使用できます。
    fn set_size(&self, size: usize) -> impl Future<Output = Result<(), Error>> + Send;

    /// ダウンロードしたデータのチャンクを書き込む
    ///
    /// このメソッドはダウンロードの進行中に複数回呼び出され、
    /// ダウンロードしたデータの各チャンクが渡されます。
    fn write(&self, b: Bytes) -> impl Future<Output = Result<(), Error>> + Send;
}


/// Google Driveへのアクセスを提供するメインクラス
///
/// この構造体は認証、ファイル一覧の取得、ダウンロードなど、
/// Google Drive APIとの対話に必要な主要な機能を提供します。
pub struct GDrive(DriveHub<HttpsConnector<HttpConnector>>);

impl GDrive {
    /// OAuth2認証を使用してGoogle Driveに接続する
    ///
    /// # 引数
    ///
    /// * `client_secret` - クライアントシークレットのJSONファイルのパス
    /// * `save_token` - 認証トークンを保存するパス
    /// * `flow_delegate` - 認証フローのカスタマイズに使用できるデリゲート（オプション）
    ///
    /// # 戻り値
    ///
    /// 認証済みのGDriveインスタンス、または発生したエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, InstalledFlowDelegate};
    ///
    /// struct MyDelegate;
    ///
    /// impl InstalledFlowDelegate for MyDelegate {
    ///     // 実装は省略
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth(
    ///         "client_secret.json",
    ///         "./token.json",
    ///         Some(Box::new(MyDelegate {})),
    ///     ).await?;
    ///
    ///     // ここでdriveを使用
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// 指定されたフォルダ内のファイルとフォルダの一覧を取得
    ///
    /// # 引数
    ///
    /// * `id` - 一覧を取得するフォルダのID
    ///
    /// # 戻り値
    ///
    /// フォルダ内のアイテムのメタデータのリスト、または発生したエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, GDriveId};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth("client_secret.json", "./token.json", None).await?;
    ///     let folder_id: GDriveId = "your_folder_id".into();
    ///
    ///     let items = drive.list(&folder_id).await?;
    ///     for item in items {
    ///         println!("{}: {}", item.name, if item.is_directory() { "フォルダ" } else { "ファイル" });
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn list(&self, id: &GDriveId) -> Result<Vec<GMeta>, Error> {
        let query = format!("'{}' in parents", id);
        let mut v = Vec::new();
        let mut next = self.list_internal(&query, None, &mut v).await?;
        while next.is_some() {
            next = self.list_internal(&query, None, &mut v).await?;
        }
        Ok(v)
    }

    /// 内部的なページング付きリスト取得関数
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

    /// ファイルまたはフォルダのメタデータを取得
    ///
    /// # 引数
    ///
    /// * `id` - メタデータを取得するファイル/フォルダのID
    ///
    /// # 戻り値
    ///
    /// ファイル/フォルダのメタデータ、または発生したエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, GDriveId};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth("client_secret.json", "./token.json", None).await?;
    ///     let file_id: GDriveId = "your_file_id".into();
    ///
    ///     let meta = drive.get_meta(&file_id).await?;
    ///     println!("名前: {}", meta.name);
    ///     println!("MIMEタイプ: {}", meta.mime_type);
    ///     println!("変更日時: {}", meta.modified_time);
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// カスタムハンドラを使用してファイルをダウンロード
    ///
    /// # 引数
    ///
    /// * `id` - ダウンロードするファイルのID
    /// * `handler` - ダウンロードしたデータを処理するハンドラ
    ///
    /// # 戻り値
    ///
    /// 成功した場合は`Ok(())`、失敗した場合はエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, GDriveId, DownloadHandler, Error};
    /// use hyper::body::Bytes;
    /// use std::sync::Arc;
    ///
    /// struct MyHandler(Arc<tokio::sync::Mutex<Vec<u8>>>);
    ///
    /// impl DownloadHandler for MyHandler {
    ///     fn set_size(&self, size: usize) -> impl std::future::Future<Output = Result<(), Error>> + Send {
    ///         let data = self.0.clone();
    ///         async move {
    ///             let mut data = data.lock().await;
    ///             data.reserve(size);
    ///             Ok(())
    ///         }
    ///     }
    ///
    ///     fn write(&self, b: Bytes) -> impl std::future::Future<Output = Result<(), Error>> + Send {
    ///         let data = self.0.clone();
    ///         async move {
    ///             let mut data = data.lock().await;
    ///             data.extend_from_slice(&b);
    ///             Ok(())
    ///         }
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth("client_secret.json", "./token.json", None).await?;
    ///     let file_id: GDriveId = "your_file_id".into();
    ///
    ///     let data = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    ///     let handler = MyHandler(data.clone());
    ///
    ///     drive.download(&file_id, handler).await?;
    ///
    ///     let content = Arc::try_unwrap(data).unwrap().into_inner();
    ///     println!("ダウンロードサイズ: {} バイト", content.len());
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// ファイルをダウンロードして指定したパスに保存
    ///
    /// # 引数
    ///
    /// * `id` - ダウンロードするファイルのID
    /// * `file` - 保存先のファイルパス
    ///
    /// # 戻り値
    ///
    /// 成功した場合は`Ok(())`、失敗した場合はエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, GDriveId};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth("client_secret.json", "./token.json", None).await?;
    ///     let file_id: GDriveId = "your_file_id".into();
    ///
    ///     drive.download_and_save(&file_id, "./downloads/myfile.pdf").await?;
    ///     println!("ファイルをダウンロードしました！");
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// ファイルをダウンロードしてバイト配列として返す
    ///
    /// # 引数
    ///
    /// * `id` - ダウンロードするファイルのID
    ///
    /// # 戻り値
    ///
    /// ダウンロードしたファイルのバイト配列、または発生したエラー
    ///
    /// # 例
    ///
    /// ```skip
    /// use google_drive::{GDrive, GDriveId};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let drive = GDrive::oauth("client_secret.json", "./token.json", None).await?;
    ///     let file_id: GDriveId = "your_file_id".into();
    ///
    ///     let data = drive.download_as_binary(&file_id).await?;
    ///     println!("ダウンロードサイズ: {} バイト", data.len());
    ///
    ///     Ok(())
    /// }
    /// ```
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

/// OAuth2トークンプロバイダ
///
/// なぜか Authenticator が GetToken を実装してないので自力で実装する
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
