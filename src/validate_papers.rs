use std::error::Error;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rdf::node::Node::{LiteralNode, UriNode};
use rdf::reader::n_triples_parser::NTriplesParser;
use rdf::reader::rdf_parser::RdfParser;

fn validate_paper(lines: &Vec<String>) -> Result<(), Box<dyn Error>> {
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

    entity.ok_or("Entity missing")?;
    rank.ok_or("Rank missing")?;
    title.ok_or("Title missing")?;
    date.ok_or("Date missing")?;
    citations.ok_or("Citations missing")?;
    journal.ok_or("Journal missing")?;

    Ok(())
}

pub fn parse(path: &Path, outfile: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = String::new();

    let mut writer = BufWriter::new(File::create(outfile)?);

    const ENTITY: &str = "<http://ma-graph.org/entity/";

    let mut curr_entity_opt: Option<String> = None;

    let mut lines: Vec<String> = vec![];

    let mut valid_papers: usize = 0;
    let mut line_no: usize = 0;

    while let Some(line) = reader
        .read_line(&mut buf)
        .map(|u| if u == 0 { None } else { Some(&buf) })
        .transpose()
    {
        let line = line?;

        line_no += 1;

        let entity = if line.starts_with(ENTITY) {
            line.split_once(' ').map(|split| split.0).unwrap()
        } else {
            ""
        };

        if let Some(curr_entity) = curr_entity_opt.as_ref() {
            if curr_entity != entity {
                curr_entity_opt = Some(entity.into());

                match validate_paper(&lines) {
                    Ok(_) => {
                        valid_papers += 1;

                        if valid_papers % 10_000 == 0 {
                            print!("\r{} valid papers, line {}", valid_papers, line_no);
                            stdout().flush()?;
                        }

                        for line in &lines {
                            write!(writer, "{}", line)?;
                        }
                    }
                    _ => {}
                }

                lines.clear();
            }
        } else {
            curr_entity_opt = Some(entity.into());
        }

        lines.push(line.clone());
        buf.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_papers() {
        parse(Path::new("Papers_100.nt"), Path::new("Papers_100_valid.nt")).unwrap();
    }
}
