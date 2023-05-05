use serde::{Deserialize, Serialize};
use std::path::Path;

fn read_csv_sample1(path: &Path) -> Result<(), csv::Error> {
  let mut reader = csv::Reader::from_path(path)?;
  for result in reader.records() {
    let data = result?;
    println!("{data:?}");
  }
  Ok(())
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct Entry {
  year: u16,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  weight: f32,
  #[serde(with = "fat_format")]
  fat: Option<f32>,
}

mod fat_format {
  use serde::{Deserializer, Serializer, Deserialize};

  pub fn serialize<S>(value: &Option<f32>, s: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if let Some(f) = *value {
      s.serialize_f32(f)
    } else {
      s.serialize_str("??")
    }
  }

  pub fn deserialize<'d, D>(d: D) -> Result<Option<f32>, D::Error>
  where
    D: Deserializer<'d>,
  {
    Option::<f32>::deserialize(d).or_else(|_| Ok(None))
  }
}

fn read_csv_sample2(path: &Path) -> Result<Vec<Entry>, csv::Error> {
  let mut reader = csv::Reader::from_path(path)?;
  let mut list = Vec::new();
  for result in reader.deserialize() {
    let data: Entry = result?;
    list.push(data);
    println!("{data:?}");
  }
  Ok(list)
}

fn print_csv_sample(v: &Vec<Entry>) -> Result<(), csv::Error> {
  let mut writer = csv::Writer::from_writer(std::io::stdout());
  for e in v.into_iter() {
    writer.serialize(e)?;
  }
  Ok(())
}

fn main() -> Result<(), csv::Error> {
  let path = Path::new("weight_data.csv");
  read_csv_sample1(&path)?;
  let v = read_csv_sample2(&path)?;
  print_csv_sample(&v)?;
  Ok(())
}
