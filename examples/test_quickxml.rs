use env_logger::{Builder, Target};
use log::*;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

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


    let filename = "./src/tests/testdata/current_context.atom";
    let mut file = File::open(filename).unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml = fix_xml(xml);
    debug!("{}", xml);
    // let mut reader = Reader::from_str(&xml);
    // reader.trim_text(true);
    // let mut writer:  = Writer::new(Cursor::new(Vec::new()));

    // let mut xmlstate = XMLParserState::Idle;


    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    /// `<entry>` - this works
    struct Entry {
        title: String,
        id: String,
        updated: String,
        author: Author,
        content: Content,
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Link {
        #[serde(rename="@href")]
        href: String,
        #[serde(rename="@rel")]
        rel: String,
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    /// The "Generator" bit - this works
    struct Generator {
        #[serde(rename="@build")]
        build: String,
        #[serde(rename="@version")]
        version: String,
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    /// The atom feed item - this works
    struct Feed {
        entry: Entry,
        author: Author,
        #[serde(rename="totalResults")]
        total_results: u32,
        #[serde(rename="itemsPerPage")]
        items_per_page: u32,
        #[serde(rename="startIndex")]
        start_index: u32,
        updated: String,
        title: String,
        id: String,
        generator: Generator,

    }

    fn fix_xml(input: String) -> String {
        let replacements = [
            "s",
            "opensearch",
        ];

        let mut res = input.clone();

        for r in replacements {
            res = res
                .replace(&format!("<{}:", r), "<")
                .replace(&format!("</{}:", r), "</")
        };
        res
    }

    let result: Feed = from_str(&xml).unwrap();
    debug!("result:\n{:#?}", result);

}
