use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rdf::node::Node::{LiteralNode, UriNode};
use rdf::reader::n_triples_parser::NTriplesParser;
use rdf::reader::rdf_parser::RdfParser;

use crate::journals_wikidata_fields::Journal;
use crate::types::*;

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

pub fn filter(
    path: &Path,
    outfile: &Path,
    journals: HashMap<Entity, Journal>,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = String::new();

    let mut writer = BufWriter::new(File::create(outfile)?);

    writeln!(writer, "entity\trank\ttitle\tdate\tcitations\tjournal_issn\tjournal_rank\tjournal_name\tjournal_subject\tjournal_field\tsubject_field_distance")?;

    const ENTITY: &str = "<http://ma-graph.org/entity/";

    let mut curr_entity_opt: Option<String> = None;

    let mut lines: Vec<String> = vec![];

    let mut num_papers: usize = 0;
    let mut num_papers_found: usize = 0;

    while let Some(line) = reader
        .read_line(&mut buf)
        .map(|u| if u == 0 { None } else { Some(&buf) })
        .transpose()
    {
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
                        num_papers += 1;

                        if let Some(journal) = journals.get(&paper.journal) {
                            num_papers_found += 1;

                            writeln!(
                                writer,
                                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3e}",
                                paper.entity,
                                paper.rank,
                                paper.title,
                                paper.date,
                                paper.citations,
                                journal.issn,
                                journal.rank,
                                journal.name,
                                journal.subject,
                                journal.field,
                                journal.distance
                            )?;
                        }

                        if num_papers % 10_000 == 0 {
                            print!("\r {} / {}", num_papers_found, num_papers);
                            stdout().flush()?;
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
    use crate::journals_wikidata_fields::read_wikidata;

    use super::*;

    #[test]
    fn test_read_papers() {
        let journals = read_wikidata(Path::new("journals_mag_wikidata_fields.csv")).unwrap();
        filter(
            Path::new("Papers_valid_1000.nt"),
            Path::new("Papers_journal_field.tsv"),
            journals,
        )
        .unwrap();
    }
}
