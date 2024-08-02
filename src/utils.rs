use std::str::FromStr;
use itertools::Itertools;

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn snake_to_pascal_case(text: &str) -> String {
    text.split("_").map(capitalize).join("")
}

pub fn from_filename<T: FromStr>(filename: &str) -> Option<T> {
    T::from_str(&snake_to_pascal_case(filename)).ok()
}