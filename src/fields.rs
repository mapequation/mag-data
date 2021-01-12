use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use csv;
use serde::Deserialize;

type Issn = String;
type Subject = String;
type Field = String;
type Journal = String;

#[derive(Deserialize, Debug)]
// journal,journalLabel,subjectLabel,fieldLabel,issn
struct Record {
    #[serde(skip)]
    journal: String,
    #[serde(rename(deserialize = "journalLabel"))]
    journal_name: Journal,
    #[serde(rename(deserialize = "subjectLabel"))]
    subject: Subject,
    #[serde(rename(deserialize = "fieldLabel"))]
    field: Option<Field>,
    issn: Issn,
}

pub fn read_wikidata(path: &Path) -> Result<HashMap<Issn, (Journal, Subject)>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;

    let mut map = HashMap::new();

    for record in reader.deserialize() {
        let record: Record = record?;
        map.insert(record.issn, (record.journal_name, record.subject));
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_wikidata() {
        read_wikidata(Path::new("query.csv")).unwrap();
    }
}