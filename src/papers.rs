use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rdf::node::Node::{LiteralNode, UriNode};
use rdf::reader::n_triples_parser::NTriplesParser;
use rdf::reader::rdf_parser::RdfParser;

type Entity = String;

#[derive(Default, Debug)]
pub struct Paper {
    entity: Entity,
    rank: u32,
    title: String,
    date: String,
    citations: u32,
    journal: Entity,
}

impl TryFrom<&Vec<String>> for Paper {
    type Error = Box<dyn Error>;

    fn try_from(lines: &Vec<String>) -> Result<Self, Self::Error> {
        let lines: String = lines.join("\n");

        let mut entity: Option<String> = None;
        let mut rank: Option<u32> = None;
        let mut title: Option<String> = None;
        let mut date: Option<String> = None;
        let mut citations: Option<u32> = None;
        let mut journal: Option<String> = None;

        for triple in NTriplesParser::from_string(lines).decode()?.triples_iter() {
            let x = triple.subject();
            if let UriNode { uri } = x {
                if entity.is_none() {
                    entity = Some(String::from(uri.to_string()));
                }
            }

            if let UriNode { uri: pred_uri } = triple.predicate() {
                match pred_uri.to_string() {
                    pred_uri_str if pred_uri_str.ends_with("appearsInJournal") => {
                        if let UriNode { uri: obj_uri } = triple.object() {
                            journal = Some(String::from(obj_uri.to_string()));
                        }
                    }
                    pred_uri_str => {
                        if let LiteralNode { literal, .. } = triple.object() {
                            match pred_uri_str {
                                str if str.ends_with("title") => title = Some(literal.to_owned()),
                                str if str.ends_with("rank") => rank = Some(literal.parse()?),
                                str if str.ends_with("citationCount") => {
                                    citations = Some(literal.parse()?)
                                }
                                str if str.ends_with("publicationDate") => {
                                    date = Some(literal.to_owned())
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(Paper {
            entity: entity.ok_or("Entity missing")?,
            rank: rank.ok_or("Rank missing")?,
            title: title.ok_or("Title missing")?,
            date: date.ok_or("Date missing")?,
            citations: citations.ok_or("Citations missing")?,
            journal: journal.ok_or("Journal missing")?,
        })
    }
}

pub fn read_papers(path: &Path) -> Result<HashMap<Entity, Paper>, Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = String::new();

    const ENTITY: &str = "<http://ma-graph.org/entity/";

    let mut curr_entity_opt: Option<String> = None;

    let mut lines: Vec<String> = vec![];

    let mut papers = HashMap::new();

    let mut row = 0;

    while let Some(line) = reader
        .read_line(&mut buf)
        .map(|u| if u == 0 { None } else { Some(&buf) })
        .transpose()
    {
        row += 1;

        if row % 100_000 == 0 {
            println!("{}", row);
        }

        let line = line?;

        let entity = if line.starts_with(ENTITY) {
            line.split_once(' ').map(|split| split.0).unwrap()
        } else {
            ""
        };

        if let Some(curr_entity) = curr_entity_opt.as_ref() {
            if curr_entity != entity {
                curr_entity_opt = Some(entity.into());

                match Paper::try_from(&lines) {
                    Ok(paper) => {
                        //println!("{:?}", paper);
                        papers.insert(paper.entity.clone(), paper);
                    }
                    Err(_err) => {}
                }

                lines.clear();
            }
        } else {
            curr_entity_opt = Some(entity.into());
        }

        lines.push(line.clone());
        buf.clear();
    }

    Ok(papers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_papers() {
        let papers = read_papers(Path::new("Papers_100.nt")).unwrap();
        println!("{}", papers.len());
    }
}
