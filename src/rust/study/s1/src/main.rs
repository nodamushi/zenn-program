// cargo run --bin s1 で実行可能
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
  pub fn my_panic() -> u32 {
    panic!("Panic!");
  }

  #[test]
  fn hogehoge() {
    assert_eq!(2 + 2, 4);
  }
  #[test]
  #[should_panic]
  fn panic_test() {
    let x = my_panic();
    println!("x = {}", x);
  }
}
