use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use csv;
use serde::Deserialize;

use crate::types::*;

// issn,entity,rank,name,subject,field,distance
#[derive(Deserialize, Debug)]
pub struct Journal {
    pub issn: Issn,
    pub entity: Entity,
    pub rank: u32,
    pub name: Name,
    pub subject: Subject,
    pub field: Field,
    pub distance: f64,
}

pub fn read_wikidata(path: &Path) -> Result<HashMap<Entity, Journal>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;

    let mut map = HashMap::new();

    for journal in reader.deserialize() {
        let journal: Journal = journal?;
        map.insert(journal.entity.clone(), journal);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_wikidata() {
        read_wikidata(Path::new("journals_mag_wikidata_fields.csv")).unwrap();
    }
}
