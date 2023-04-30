use std::error::Error;

mod myerror;

fn main() -> std::result::Result<(), Box<dyn Error>> {
  match myerror::hoge() {
    Ok(_) => {}
    Err(e) => {
      println!("piyo3 Error!! {e}");
      if let Some(source) = e.source() {
        println!("  Error source {source}");
      }
    }
  }

  match myerror::piyo() {
    Ok(_) => {}
    Err(e) => {
      println!("piyo3 Error!! {e}");
      if let Some(source) = e.source() {
        println!("  Error source {source}");
      }
    }
  }

  match myerror::piyo2() {
    Ok(_) => {}
    Err(e) => {
      println!("piyo3 Error!! {e}");
      if let Some(source) = e.source() {
        println!("  Error source {source}");
      }
    }
  }
  match myerror::piyo3() {
    Ok(_) => {}
    Err(e) => {
      println!("piyo3 Error!! {e}");
      if let Some(source) = e.source() {
        println!("  Error source {source}");
      }
    }
  }

  Ok(())
}
