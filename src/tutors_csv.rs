use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[allow(unused)]
struct Record {
    #[serde(rename = "ID")]
    #[serde(deserialize_with = "deserialize_id")]
    id: u32,
    #[serde(rename = "Vollständiger Name")]
    name: String,
    #[serde(rename = "ID-Nummer")]
    id_number: u32,
    #[serde(rename = "E-Mail-Adresse")]
    email: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(deserialize_with = "deserialize_rating")]
    rating: f32,
    #[serde(rename = "Bestwertung")]
    #[serde(deserialize_with = "deserialize_rating")]
    best_rating: f32,
    #[serde(rename = "Bewertung kann geändert werden")]
    rating_changeable: String,
    #[serde(rename = "Zuletzt geändert (Abgabe)")]
    last_change_submission: String,
    #[serde(rename = "Zuletzt geändert (Bewertung)")]
    last_change_rating: String,
    #[serde(rename = "Feedback als Kommentar")]
    feedback: String,
}


pub const ID_PATTERN: &str = r"([\d]+)";
fn deserialize_id<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let re = Regex::new(ID_PATTERN).unwrap();
    if re.is_match(&s) {
        let caps = re.captures(&s).unwrap();
        let id = caps.get(1).unwrap().as_str().parse().unwrap();
        Ok(id)
    } else {
        Err(serde::de::Error::custom("Invalid ID"))
    }
}

fn deserialize_rating<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let s = s.replace(",", ".");

    match s.parse() {
        Ok(f) => Ok(f),
        Err(_) => Ok(0f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deserialize() {
        let data = r#"
        Teilnehmer/in1234567,"asdf ghjklö",12345678,K12345678@students.jku.at,"Zur Bewertung abgegeben",,"24,00",Ja,"Mittwoch, 13. März 2024, 10:15","Mittwoch, 13. März 2024, 19:59",
        Teilnehmer/in7654321,"fdsa ölkjhg",87654321,K87654321@students.jku.at,"Zur Bewertung abgegeben",,"24,00",Ja,"Mittwoch, 13. März 2024, 15:10","Mittwoch, 13. März 2024, 19:59",
        "#;

        let record0 = Record {
            id: 1234567,
            name: "asdf ghjklö".to_string(),
            id_number: 12345678,
            email: "K12345678@students.jku.at".to_string(),
            status: "Zur Bewertung abgegeben".to_string(),
            rating: 0.0,
            best_rating: 24.0,
            rating_changeable: "Ja".to_string(),
            last_change_submission: "Mittwoch, 13. März 2024, 10:15".to_string(),
            last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
            feedback: "".to_string(),
        };

        let record1 = Record {
            id: 7654321,
            name: "fdsa ölkjhg".to_string(),
            id_number: 87654321,
            email: "K87654321@students.jku.at".to_string(),
            status: "Zur Bewertung abgegeben".to_string(),
            rating: 0.0,
            best_rating: 24.0,
            rating_changeable: "Ja".to_string(),
            last_change_submission: "Mittwoch, 13. März 2024, 15:10".to_string(),
            last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
            feedback: "".to_string(),
        };
        
        let records = vec![record0, record1];

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .has_headers(false)
            .from_reader(data.as_bytes());

        for (i, record) in reader.deserialize::<Record>().filter_map(Result::ok).take(2).enumerate() {
            assert_eq!(record, records[i])
        }
    }

    #[test]
    fn test_serialize() {
        let record0 = Record {
            id: 1234567,
            name: "asdf ghjklö".to_string(),
            id_number: 12345678,
            email: "K12345678@students.jku.at".to_string(),
            status: "Zur Bewertung abgegeben".to_string(),
            rating: 0.0,
            best_rating: 24.0,
            rating_changeable: "Ja".to_string(),
            last_change_submission: "Mittwoch, 13. März 2024, 10:15".to_string(),
            last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
            feedback: "".to_string(),
        };

        let record1 = Record {
            id: 7654321,
            name: "fdsa ölkjhg".to_string(),
            id_number: 87654321,
            email: "K87654321@students.jku.at".to_string(),
            status: "Zur Bewertung abgegeben".to_string(),
            rating: 0.0,
            best_rating: 24.0,
            rating_changeable: "Ja".to_string(),
            last_change_submission: "Mittwoch, 13. März 2024, 15:10".to_string(),
            last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
            feedback: "".to_string(),
        };

        let data = r#"Teilnehmer/in1234567,"asdf ghjklö",12345678,K12345678@students.jku.at,"Zur Bewertung abgegeben",,"24,00",Ja,"Mittwoch, 13. März 2024, 10:15","Mittwoch, 13. März 2024, 19:59",
Teilnehmer/in7654321,"fdsa ölkjhg",87654321,K87654321@students.jku.at,"Zur Bewertung abgegeben",,"24,00",Ja,"Mittwoch, 13. März 2024, 15:10","Mittwoch, 13. März 2024, 19:59","#;
        
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(false)
            .from_writer(vec![]);
        wtr.serialize(record0).unwrap();
        wtr.serialize(record1).unwrap();
        
        assert_eq!(String::from_utf8(wtr.into_inner().unwrap()).unwrap(), data);
    }
}
