use std::{
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{anyhow, Result};
use reqwest::Client;
use ripunzip::UnzipOptions;
use tempfile::tempdir;
use tokio::{io::AsyncWriteExt, process::Command, time::Instant};

#[tokio::main]
async fn main() {
    init().await;
    let a = test::<ZipExtra>().await;
    let b = test::<Ripunzip>().await;
    let c = test::<ParallelZip>().await;
    let d = test::<AsyncZip>().await;
    let e = test::<AsyncZipParallel>().await;

    println!("a == c ? {}", a == c);
    println!("a == e ? {}", a == e);

}

// The wrap time of Windows explorer is 2:23
const TEST_ZIP_FILE_URL: &str = "https://github.com/winpython/winpython/releases/download/13.1.202502222final/Winpython64-3.12.9.0dot.zip";
const TEST_ZIP_PATH: &str = ".tmp/winpython.zip";

async fn init() {
    let p = Path::new(TEST_ZIP_PATH);
    if !p.is_file() {
        let client = Client::new();
        let response = client
            .get(TEST_ZIP_FILE_URL)
            .send()
            .await
            .expect("[ERR] Request failed");
        let bytes = response.bytes().await.expect("[ERR] Failed to get bytes");
        if let Some(parent) = p.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .expect("[ERR] Failed to create directory");
        }
        let mut file = tokio::fs::File::create(p)
            .await
            .expect("[ERR] Failed to create file");
        file.write_all(&bytes)
            .await
            .expect("[ERR] failed to write file");
    }
}

trait Unzip {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()>;
}

async fn test<U: Unzip>() -> Vec<String> {
    let name = std::any::type_name::<U>();
    let Ok(odir) = tempdir() else {
        eprintln!("Fail to create temp directory");
        exit(1)
    };
    println!("[LOG] Test {}", name);
    let instant = Instant::now();
    if let Err(e) = U::unzip(TEST_ZIP_PATH, &odir).await {
        println!("[ERR] Fail to test {}: {}", name, e);
        return vec![];
    }
    let time = instant.elapsed();
    println!("[LOG]   Result: {:?}", time);
    let path: &Path = odir.as_ref();
    if let Ok(x) = find_and_sort(path.into()) {
        x
    } else {
        vec![]
    }
}

/// 指定されたディレクトリを再帰的に検索し、見つかったファイルパスをソートして返す
///
/// # 引数
/// * `dir` - 検索を開始するディレクトリのパス
///
/// # 戻り値
/// * `Result<Vec<String>, Box<dyn Error>>` - 成功した場合はソートされたファイルパスのリスト、失敗した場合はエラー
pub fn find_and_sort(root_path: PathBuf) -> Result<Vec<String>> {
    let mut result = Vec::new();
    collect_files(&root_path, &root_path, &mut result)?;
    result.sort();
    Ok(result)
}

/// 再帰的にファイルを収集する補助関数
fn collect_files(
    root: &Path,
    current: &Path,
    result: &mut Vec<String>
) -> Result<()> {
    if current.is_dir() {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            collect_files(root, &path, result)?;
        }
    } else if current.is_file() {
        if let Ok(relative) = current.strip_prefix(root) {
            if let Some(path_str) = relative.to_str() {
                if !path_str.is_empty() {
                    result.push(path_str.to_string());
                }
            }
        }
    }
    Ok(())
}



fn is_safe_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if path.to_str().is_none() || path.to_string_lossy().contains('\0') {
        return false;
    }

    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if components.is_empty() {
                    return false;
                }
                components.pop();
            }
            std::path::Component::Normal(_) => components.push(component),
            std::path::Component::CurDir => {}
            _ => return false,
        }
    }
    true
}

///
/// zip_extra
///
struct ZipExtra {}
impl Unzip for ZipExtra {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()> {
        use std::fs::File;
        use std::io::BufReader;

        let reader = BufReader::new(File::open(src)?);
        zip_extract::extract(reader, dir.as_ref(), false)?;

        Ok(())
    }
}

///
/// ripunzip
///
struct Ripunzip {}
impl Unzip for Ripunzip {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()> {
        use std::fs::File;

        let file = File::open(src)?;
        let zip = ripunzip::UnzipEngine::for_file(file)?;
        zip.unzip(UnzipOptions {
            output_directory: Some(dir.as_ref().into()),
            password: None,
            single_threaded: false,
            filename_filter: None,
            progress_reporter: Box::new(ripunzip::NullProgressReporter {}),
        })?;

        Ok(())
    }
}

struct ParallelZip {}
impl Unzip for ParallelZip {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()> {
        use std::fs::File;
        use std::io::BufReader;

        let len = {
            let reader = BufReader::new(File::open(&src)?);

            zip::ZipArchive::new(reader)?.len()
        };
        let task = async |from: usize, end: usize, src: PathBuf, base: PathBuf| -> Result<()> {
            let reader = BufReader::new(File::open(src)?);
            let mut zip = zip::ZipArchive::new(reader)?;
            for i in from..end {
                let mut file = zip.by_index(i)?;
                let path = file.mangled_name();
                if path.to_string_lossy().is_empty() {
                    continue;
                }
                if !is_safe_path(&path) {
                    continue;
                }
                let path = base.join(path);

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(path)?;
                } else if let Some(parent) = path.parent() {
                    if !parent.is_dir() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut out = std::fs::File::create(&path)?;
                    std::io::copy(&mut file, &mut out)?;
                }
            }
            Ok(())
        };

        let cores = num_cpus::get() / 2;
        println!("[LOG]  cores = {}", cores);
        let joins: Vec<_> = (0..cores)
            .into_iter()
            .map(|i| {
                let from = len * i / cores;
                let end = len * (i + 1) / cores;
                tokio::task::spawn(task(from, end, src.as_ref().into(), dir.as_ref().into()))
            })
            .collect();
        let mut errmsg = String::new();
        for j in joins {
            match j.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    errmsg.push_str(&format!("{}\n", e));
                }
                Err(e) => {
                    errmsg.push_str(&format!("{}\n", e));
                }
            }
        }
        if !errmsg.is_empty() {
            Err(anyhow!("{}", errmsg))
        } else {
            Ok(())
        }
    }
}

///
/// async_zip
///
struct AsyncZip {}
impl Unzip for AsyncZip {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()> {
        use async_zip::tokio::read::seek::ZipFileReader;
        use tokio::fs::{create_dir_all, File};
        use tokio::io::BufReader;
        use tokio_util::compat::FuturesAsyncReadCompatExt;

        let mut zip = ZipFileReader::with_tokio(BufReader::new(File::open(src).await?)).await?;
        let base = dir.as_ref();
        let len = zip.file().entries().len();
        for i in 0..len {
            let e = zip.file().entries().get(i).unwrap();
            let path = Path::new(e.filename().as_str()?);
            if !is_safe_path(&path) {
                continue;
            }

            let path = base.join(path);

            if e.dir()? {
                create_dir_all(path).await?;
            } else {
                let mut reader = zip.reader_without_entry(i).await?.compat();
                let mut file = File::create(path).await?;
                tokio::io::copy(&mut reader, &mut file).await?;
            }
        }
        Ok(())
    }
}

///
/// async_zip (parallel)
///
struct AsyncZipParallel {}
impl Unzip for AsyncZipParallel {
    async fn unzip<S: AsRef<Path>, D: AsRef<Path>>(src: S, dir: D) -> Result<()> {
        use async_zip::tokio::read::seek::ZipFileReader;
        use tokio::fs::{create_dir_all, File};
        use tokio::io::BufReader;
        use tokio_util::compat::FuturesAsyncReadCompatExt;

        let len = {
            let zip = ZipFileReader::with_tokio(BufReader::new(File::open(&src).await?)).await?;
            zip.file().entries().len()
        };
        let task = async |from: usize, end: usize, src: PathBuf, base: PathBuf| -> Result<()> {
            let mut zip = ZipFileReader::with_tokio(BufReader::new(File::open(src).await?)).await?;
            for i in from..end {
                let e = zip.file().entries().get(i).unwrap();
                let path = Path::new(e.filename().as_str()?);
                if !is_safe_path(&path) {
                    continue;
                }

                let path = base.join(path);

                if e.dir()? {
                    create_dir_all(path).await?;
                } else if let Some(parent) = path.parent() {
                    if !parent.is_dir() {
                        create_dir_all(parent).await?;
                    }
                    let mut reader = zip.reader_without_entry(i).await?.compat();
                    let mut file = File::create(path).await?;
                    tokio::io::copy(&mut reader, &mut file).await?;
                }
            }
            Ok(())
        };

        let cores = num_cpus::get();
        println!("[LOG]  cores = {}", cores);
        let joins: Vec<_> = (0..cores)
            .into_iter()
            .map(|i| {
                let from = len * i / cores;
                let end = len * (i + 1) / cores;
                tokio::task::spawn(task(from, end, src.as_ref().into(), dir.as_ref().into()))
            })
            .collect();
        let mut errmsg = String::new();
        for j in joins {
            match j.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    errmsg.push_str(&format!("{}\n", e));
                }
                Err(e) => {
                    errmsg.push_str(&format!("{}\n", e));
                }
            }
        }
        if !errmsg.is_empty() {
            Err(anyhow!("{}", errmsg))
        } else {
            Ok(())
        }
    }
}
