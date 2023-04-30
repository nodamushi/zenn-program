use std::{error::Error, result};
use thiserror::Error;

#[derive(Debug)]
pub enum MyError {
  Foo(String),
  Bar(u32),
}

impl std::error::Error for MyError {}
impl std::fmt::Display for MyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Foo(x) => write!(f, "Foo Error: {}", x),
      Self::Bar(x) => write!(f, "Bar Error: {}", x),
    }
  }
}

pub type Result<T> = std::result::Result<T, MyError>;

pub fn foo(x: &str) -> Result<()> {
  Err(MyError::Foo(x.to_string()))
}

pub fn bar(v: u32) -> Result<()> {
  Err(MyError::Bar(v))
}

pub fn hoge() -> Result<()> {
  match foo("piyo") {
    Ok(()) => {}
    Err(e) => return Err(e),
  };
  // ↑と同等
  foo("hoge")?;
  Ok(())
}

pub fn piyo() -> result::Result<u32, Box<dyn Error>> {
  bar(32)?;
  let _ = std::fs::File::open("piyo")?;
  Ok(1)
}

// ---------------------------
// Box<dyn Error> → MyError2
// ---------------------------
#[derive(Debug)]
pub enum MyError2 {
  MyError(MyError),
  IoError(std::io::Error),
}

impl std::error::Error for MyError2 {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::MyError(x) => Some(x),
      Self::IoError(x) => Some(x),
    }
  }
}
impl std::fmt::Display for MyError2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MyError(x) => write!(f, "MyError Error: {}", x),
      Self::IoError(x) => write!(f, "IO Error: {}", x),
    }
  }
}

impl From<MyError> for MyError2 {
  fn from(x: MyError) -> Self {
    Self::MyError(x)
  }
}

impl From<std::io::Error> for MyError2 {
  fn from(x: std::io::Error) -> Self {
    Self::IoError(x)
  }
}

pub fn piyo2() -> result::Result<u32, MyError2> {
  bar(32)?;
  let _ = std::fs::File::open("piyo")?;
  Ok(1)
}

// -----------------------
// Use thiserror
// -----------------------
#[derive(Error, Debug)]
pub enum MyError3 {
  #[error("3: My Error: {0}")]
  MyError(#[from] MyError),
  #[error("3: Io Error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("3: Hoge Error {a}, {b}")]
  Hoge { a: u32, b: String },
}

pub fn piyo3() -> result::Result<u32, MyError3> {
  bar(32)?;
  let _ = std::fs::File::open("piyo")?;
  Err(MyError3::Hoge {
    a: 1,
    b: "hoge".to_string(),
  })
}
