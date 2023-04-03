#[allow(dead_code)]
#[allow(unused_imports)]

use std::collections::HashMap;
use std::error::Error;

#[allow(unused_imports)]
use std::io::{BufReader, BufRead};
use serde::Deserialize;

#[allow(unused_imports)]
use crate::ServerConfig;
// use crate::ServerConfigType;

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_login() -> Result<(), String> {
    use crate::search::SplunkClient;
    use crate::{ServerConfig, ServerConfigType};

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig);
    eprintln!("{:?}", client);
    client.login().await.unwrap();
    Ok(())
}

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_execution() -> Result<(), String> {
    use crate::search::SearchJob;
    use crate::search::SplunkClient;
    use crate::{ServerConfig, ServerConfigType};
    use futures::stream::TryStreamExt; // for map_err
    use tokio::io::AsyncBufReadExt;
    use tokio_util::io::StreamReader;

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig);
    println!("{:#?}", client.serverconfig);

    client.login().await?;

    let search_string =
        r#"| makeresults 1 | eval foo="12345,12345" | makemv foo delim="," | mvexpand foo"#;
    println!("search string: {}", search_string);
    let search = SearchJob::create(search_string);

    let search = search
        .create(&mut client)
        .await
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    let stream = search.creation_response.bytes_stream();

    fn convert_err(_err: reqwest::Error) -> std::io::Error {
        todo!()
    }

    let mut lines = get_lines! {stream};

    while let Some(line) = lines.next_line().await.unwrap() {
        // println!("{line:?}");
        let resultline: crate::search::SearchResult = serde_json::from_str(&line).unwrap();
        println!("{:#?}", resultline);
    }
    println!("Done printing lines...");

    Ok(())
}

#[cfg(feature = "xml_raw")]
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_get_current_context() -> Result<(), String> {
    use crate::search::SplunkClient;
    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    let mut client = SplunkClient::default().with_config(serverconfig);
    client.login().await?;

    eprintln!("{:#?}", client.get_current_context().await?);

    Ok(())
}

#[cfg(feature = "xml_raw")]
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_get_capabilities() -> Result<(), String> {
    use crate::search::SplunkClient;
    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    let mut client = SplunkClient::default().with_config(serverconfig);
    client.login().await?;

    eprintln!("{:#?}", client.get_capabilities().await?);

    Ok(())
}

#[tokio::test]
async fn test_xmlrs_parser() -> Result<(), Box<dyn Error>> {

    use std::fs::File;

    let file = File::open("./src/tests/testdata/current_context.xml").unwrap();
    // let reader = BufReader::new(file);

    use xml::reader::{EventReader, XmlEvent};

    fn indent(size: usize) -> String {
        const INDENT: &'static str = "    ";
        (0..size)
            .map(|_| INDENT)
            .fold(String::with_capacity(size * INDENT.len()), |r, s| r + s)
    }
    let parser = EventReader::new(file);
    let mut depth = 0;
    for e in parser {
        match e {
            Ok(val) => {
                match val {
                    // XmlEvent::StartDocument {
                    //     version,
                    //     encoding,
                    //     standalone,
                    // } => todo!(),
                    // XmlEvent::EndDocument => todo!(),
                    // XmlEvent::ProcessingInstruction { name, data } => todo!(),
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => {
                        println!("{}+{} {:?}", indent(depth), name, attributes);
                        depth += 1;
                    }
                    XmlEvent::EndElement { name } => {
                        depth -= 1;
                        println!("{}-{}", indent(depth), name);
                    }
                    // XmlEvent::CData(_) => todo!(),
                    // XmlEvent::Comment(_) => todo!(),
                    XmlEvent::Characters(data) => println!("{}+{}", indent(depth), data),
                    XmlEvent::Whitespace(_) => {},
                    _ => println!("unhandled: {:?}", val),
                }
            }
            // Ok(XmlEvent::StartElement { name,attributes, .. }) => {
            //     println!("{}+{} {:?}", indent(depth), name, attributes);
            //     depth += 1;
            // }
            // Ok(XmlEvent::EndElement { name }) => {
            //     depth -= 1;
            //     println!("{}-{}", indent(depth), name);
            // }
            Err(e) => {
                println!("Error: {}", e);
                break;
            } // _ => {

              //     println

              // }
        }
    }

    Ok(())
}


#[tokio::test]
async fn test_feedrs_parser() -> Result<(), Box<dyn Error>> {
    use std::fs::File;

    let filename = "./src/tests/testdata/current_context.atom";
    // use std::fs::File;
    use std::io::BufReader;
    use atom_syndication::Feed;

    let file = File::open(filename).unwrap();
    let feed = Feed::read_from(BufReader::new(file)).unwrap();

    // eprintln!("Feed: {feed:#?}");


    // println!("Authors: {:?}", feed.authors);

    let entry1 = feed.entries[0].clone();
    let stripped_response = entry1.content.unwrap().value.unwrap().replace(r#"\n"#, "\n").replace("<s:", "<").replace("</s:", "</");
    println!("{}", stripped_response);

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Key{
        #[serde(rename="@name")]
        name: String,
        list: Vec<String>,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Dict {
        key: HashMap<String, Key>
    }

    let xmlcontent: Dict = serde_xml_rs::from_str(&stripped_response).unwrap();

    println!("{:#?}", xmlcontent);

    Ok(())
}