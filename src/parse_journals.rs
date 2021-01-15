use std::error::Error;
use std::fs;
use std::path::Path;

use ntriple::parser::triple_line;
use ntriple::{Object, Predicate, Subject, Triple};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Journal {
    pub entity: String,
    pub rank: u32,
    pub name: String,
    pub issn: String,
}

pub fn read_journals(path: &Path) -> Result<Vec<Journal>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;

    let mut journals = vec![];

    let mut journal = None;

    for line in contents.lines() {
        if let Some(Triple {
            subject,
            predicate,
            object,
            ..
        }) = triple_line(line)?
        {
            if let Object::IriRef(lit) = &object {
                if lit.ends_with("Journal") {
                    if let Subject::IriRef(iri) = &subject {
                        if journal.is_some() {
                            journals.push(journal.take().unwrap());
                        }

                        journal = Some(Journal {
                            entity: iri.clone(),
                            ..Journal::default()
                        });

                        continue;
                    }
                }
            }

            if journal.is_none() {
                return Err("Parse error".into());
            } else {
                let journal_ref = journal.as_mut().unwrap();

                let Predicate::IriRef(iri) = &predicate;

                if let Object::Lit(lit) = &object {
                    if iri.ends_with("rank") {
                        journal_ref.rank = lit.data.parse()?;
                    } else if iri.ends_with("name") {
                        journal_ref.name = lit.data.clone();
                    } else if iri.ends_with("issn") {
                        journal_ref.issn = lit.data.clone();
                    }
                }
            }
        }
    }

    Ok(journals)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;

    use serde_json;

    use super::*;

    #[test]
    fn test_read_journals() {
        let journals = read_journals(Path::new("Journals.nt")).unwrap();
        let serialized = serde_json::to_string(&journals).unwrap();

        let mut file = match File::create("Journals.json") {
            Ok(file) => file,
            Err(err) => panic!("Could not open file: {}", err),
        };

        match file.write_all(serialized.as_bytes()) {
            Ok(_) => {}
            Err(err) => panic!("Could not write to file: {}", err),
        }
    }
}
