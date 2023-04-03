use env_logger::{Builder, Target};
use log::*;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use splunk::strip_atom_noise;

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
enum XMLParserState {
    Idle,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Author {
    name: String,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code,non_camel_case_types)]
enum KeyValue {
    list{ item: Vec<String> },
    #[serde(rename="$text")]
    text (String),
    dict (Vec<Key>),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code,non_camel_case_types)]
struct ListItem {
    #[serde(rename="@name")]
    name: String,
    #[serde(rename="$value")]
    value: Option<KeyValue>,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Key {
    #[serde(rename="$value")]
    key: Vec<ListItem>,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
/// `<content>` ... working?
struct Content {
    dict: Vec<Key>,
}


fn main() {

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout).format_timestamp(None).init();


    let filename = "./src/tests/testdata/current_context_stripped.atom";
    let mut file = File::open(filename).unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml = strip_atom_noise(xml);
    debug!("{}", xml);



    let result: Content = from_str(&xml).unwrap();
    debug!("result:\n{:#?}", result);

}
