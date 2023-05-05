use std::fmt::{Debug, Display};

trait Foo: Default + Display {
  // selfを受け取るメソッド
  fn bar(&self) -> u32;
  // selfを受け取って更新するメソッド
  fn foobar(&mut self, x: u32) -> ();
  // 関連関数.
  fn foo_gen() -> Self {
    Self::default()
  }
}

#[derive(Debug)]
struct Hoge {
  x: u32,
}
impl Default for Hoge {
  fn default() -> Self {
    Self { x: 0 }
  }
}
impl Foo for Hoge {
  fn bar(&self) -> u32 {
    self.x
  }
  fn foobar(&mut self, x: u32) -> () {
    self.x = x;
  }
}
impl Display for Hoge {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Hoge.x = {}", self.x)
  }
}

fn piyo() -> impl Foo + Debug {
  Hoge::foo_gen()
}
fn piyopiyo() -> impl Foo + Debug {
  Hoge::foo_gen()
}

fn piyo2<T: Foo>() -> T {
  T::default()
}

fn puri(foo: &mut impl Foo) {
  foo.foobar(10);
  println!("puri:foo = {foo}.")
}

fn puri2<T: Foo>(foo: &mut T) {
  foo.foobar(10);
  println!("puri2:foo = {foo}.")
}

fn puri3(foo: &mut (impl Foo + Debug)) {
  foo.foobar(10);
  println!("puri3:foo = {:}.", foo)
}

fn puri4<T>(f1: &mut T, f2: &mut T)
where
  T: Foo + Debug,
{
  f1.foobar(10);
  f2.foobar(20);
  println!("puri4:f1 = {:}, f2 = {}.", f1, f2)
}

trait X {
  fn x(&self) -> u32;
}

impl X for u32 {
  fn x(&self) -> u32 {
    *self
  }
}

trait Bar1<I> {
  fn get(&self) -> I;
  fn set(&mut self, value: I) -> ();
}

impl Bar1<u32> for Hoge {
  fn get(&self) -> u32 {
    self.x
  }
  fn set(&mut self, value: u32) -> () {
    self.x = value;
  }
}

impl Bar1<String> for Hoge {
  fn get(&self) -> String {
    "Hoge".to_string()
  }
  fn set(&mut self, value: String) -> () {}
}

trait Bar2 {
  type I;
  fn get(&self) -> Self::I;
  fn set(&mut self, value: Self::I) -> ();
}

impl Bar2 for Hoge {
  type I = u32;
  fn get(&self) -> Self::I {
    self.x
  }

  fn set(&mut self, value: Self::I) -> () {
    self.x = value;
  }
}

fn bar1<I, T: Bar1<I>>(x: &T) -> I {
  x.get()
}
fn bar2<T: Bar2>(x: &T) -> T::I {
  x.get()
}

fn main() {
  let mut p = piyo();
  puri(&mut p);
  let mut p2: Hoge = piyo2();
  puri2(&mut p2);
  let mut p3 = Hoge { x: 0 };
  puri3(&mut p3);
  // puri4(&mut p, &mut p2); Error
  // let mut pp = piyopiyo();
  // puri4(&mut p, &mut pp); Error
  puri4(&mut p2, &mut p3);

  let _: u32 = bar1(&p2);
  bar2(&p3);
}
