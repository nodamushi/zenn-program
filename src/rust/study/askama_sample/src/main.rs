use askama::Template;

#[derive(Template)]
#[template(path = "foobar.hpp")]
/// テンプレートオブジェクト
struct Foobar<'a> {
    namespace: &'a str,
    structures: Vec<Structure<'a>>,
}

/// 構造体
struct Structure<'a> {
    name: &'a str,
    abst: &'a str,
    field: Vec<Field<'a>>,
}

/// フィールド
struct Field<'a> {
    name: &'a str,
    abst: &'a str,
    ctype: &'a str,
    dims: &'a [usize],
}

impl<'a> Foobar<'a> {
    fn new(namespace: &'a str) -> Self {
        Self {
            namespace,
            structures: Vec::new(),
        }
    }
}

impl<'a> Structure<'a> {
    fn new(name: &'a str, abst: &'a str) -> Self {
        Self {
            name,
            abst,
            field: Vec::new(),
        }
    }
}

impl<'a> Field<'a> {
    fn arr(name: &'a str, abst: &'a str, ctype: &'a str, dims: &'a [usize]) -> Self {
        Self {
            name,
            abst,
            ctype,
            dims,
        }
    }
    fn var(name: &'a str, abst: &'a str, ctype: &'a str) -> Self {        Self {
            name,
            abst,
            ctype,
            dims: &[]
        }
    }

    /// 配列かどうか。 メソッドも普通に呼べる
    fn is_array(&self) -> bool {
        match self.dims.len() {
            0 => false,
            1 => self.dims[0] > 1,
            _ => true,
        }
    }
}

fn main() {
    let foobar = {
        let mut f = Foobar::new("fooooobaaaaaa");
        f.structures.push({
            let mut s = Structure::new("Foo", "foooooo");
            s.field.push(Field::var("hoge", "HOGE", "const char*"));
            s.field.push(Field::var("piyo", "PIYO", "uint8_t"));
            s.field
                .push(Field::arr("taro", "TARO", "uint32_t", &[3, 2, 10]));
            s
        });
        f.structures.push({
            let mut s = Structure::new("Bar", "baaaaaa");
            s.field.push(Field::var("x", "HOGE", "uint32_t"));
            s.field.push(Field::var("y", "PIYO", "uint8_t"));
            s
        });
        f
    };
    println!("{}", foobar.render().unwrap());
}
