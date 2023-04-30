// cargo run --bin hoge で実行可能

fn main() {
  println!("Hoge!");
}

#[cfg(test)]
mod tests {
  #[test]
  fn mogemoge() {
    assert_ne!(1 + 1, 3);
  }
}
