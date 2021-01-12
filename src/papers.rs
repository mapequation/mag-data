use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rdf::node::Node;
use rdf::node::Node::UriNode;
use rdf::reader::n_triples_parser::NTriplesParser;
use rdf::reader::rdf_parser::RdfParser;

trait Bleh {
    fn to_string(&self) -> String;
}

impl Bleh for Node {
    fn to_string(&self) -> String {
        match self {
            Node::UriNode { uri } => uri.to_string().to_owned(),
            Node::LiteralNode { literal, .. } => literal.clone(),
            Node::BlankNode { .. } => "".into(),
        }
    }
}

#[derive(Default)]
struct Paper {
    id: u32,
    rank: u32,
    title: String,
    date: String,
    citations: u32,
    journal: String,
}

pub fn read_papers(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();

    const PAPER_URI: &str = "http://ma-graph.org/class/Paper";
    const RANK_URI: &str = "http://ma-graph.org/property/rank";
    const TITLE_URI: &str = "http://purl.org/dc/terms/title";
    const DATE_URI: &str = "http://prismstandard.org/namespaces/1.2/basic/publicationDate";
    const JOURNAL_URI: &str = "http://ma-graph.org/property/appearsInJournal";
    const CITATION_URI: &str = "http://ma-graph.org/property/citationCount";
    const TYPE_URI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    const EST_CITATION_URI: &str = "http://ma-graph.org/property/estimatedCitationCount";

    while let Some(line) = reader.read_line(&mut buf)
        .map(|u| if u == 0 { None } else { Some(&buf) })
        .transpose() {
        let line = line?;

        let graph = NTriplesParser::from_string(line).decode()?;

        // subject predicate object
        let triple = graph.triples_iter().nth(0).unwrap();

        let subject = triple.subject();
        let pred = triple.predicate();
        let object = triple.object();


        println!("{}", line.trim());
        println!("{:?}", subject);
        println!("{:?}", pred);
        println!("{:?}", object);
        if let UriNode { .. } = object {
            if object.to_string().ends_with("Paper") {
                let string = subject.to_string();
                let id = string.split("/").last().unwrap().parse::<u32>()?;
                println!("is paper {}", id);
            }
        }
        println!();


        buf.clear();
    }


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_papers() {
        read_papers(Path::new("Papers_100.nt")).unwrap();
    }
}