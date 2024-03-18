use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[allow(unused)]
struct Record {
    #[serde(rename = "ID")]
    #[serde(deserialize_with = "deserialize_id")]
    #[serde(serialize_with = "serialize_id")]
    id: String,
    #[serde(rename = "Vollständiger Name")]
    name: String,
    #[serde(rename = "ID-Nummer")]
    id_number: String,
    #[serde(rename = "E-Mail-Adresse")]
    email: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Bewertung")]
    #[serde(deserialize_with = "deserialize_rating")]
    #[serde(serialize_with = "serialize_rating")]
    rating: Option<f32>,
    #[serde(rename = "Bestwertung")]
    #[serde(deserialize_with = "deserialize_best_rating")]
    #[serde(serialize_with = "serialize_best_rating")]
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

const ID_PATTERN: &str = r"([\d]+)";
const PREFIX_ID: &str = "Teilnehmer/in";

fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let re = Regex::new(ID_PATTERN).unwrap();
    if re.is_match(&s) {
        let caps = re.captures(&s).unwrap();
        let id = caps.get(1).unwrap().as_str().to_string();
        Ok(id)
    } else {
        Err(serde::de::Error::custom("Invalid ID"))
    }
}
fn deserialize_best_rating<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.replace(",", ".");
    return match s.parse() {
        Ok(f) => Ok(f),
        Err(_) => Err(serde::de::Error::custom("Invalid rating")),
    };
}

fn deserialize_rating<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        s.replace(",", ".")
            .parse()
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

fn serialize_id<S>(id: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let id = format!("{}{}", PREFIX_ID, id);
    serializer.serialize_str(&id.to_string())
}

fn serialize_best_rating<S>(rating: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{:.2}", rating).replace(".", ","))
}

fn serialize_rating<S>(rating: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match rating {
        Some(rating) => serialize_best_rating(rating, serializer),
        None => serializer.serialize_str(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let records = get_records();

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_reader(DATA.as_bytes());

        for (i, record) in reader
            .deserialize::<Record>()
            .filter_map(Result::ok)
            .take(2)
            .enumerate()
        {
            assert_eq!(record, records[i]);
        }
    }

    #[test]
    fn test_serialize() {
        let records = get_records();

        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_writer(vec![]);
        wtr.serialize(&records[0]).unwrap();
        wtr.serialize(&records[1]).unwrap();

        assert_eq!(DATA, String::from_utf8(wtr.into_inner().unwrap()).unwrap());
    }

    fn get_records() -> [Record; 2] {
        [
            Record {
                id: 1234567.to_string(),
                name: "asdf ghjklö".to_string(),
                id_number: 12345678.to_string(),
                email: "K12345678@students.jku.at".to_string(),
                status: "Zur Bewertung abgegeben".to_string(),
                rating: Some(13.5),
                best_rating: 24.0,
                rating_changeable: "Ja".to_string(),
                last_change_submission: "Mittwoch, 13. März 2024, 10:15".to_string(),
                last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
                feedback: "".to_string(),
            },
            Record {
                id: 7654321.to_string(),
                name: "fdsa ölkjh,g".to_string(),
                id_number: 87654321.to_string(),
                email: "K87654321@students.jku.at".to_string(),
                status: "Zur Bewertung abgegeben".to_string(),
                rating: None,
                best_rating: 24.0,
                rating_changeable: "Ja".to_string(),
                last_change_submission: "Mittwoch, 13. März 2024, 15:10".to_string(),
                last_change_rating: "Mittwoch, 13. März 2024, 19:59".to_string(),
                feedback: "".to_string(),
            },
        ]
    }

    const DATA: &str = r#"ID,Vollständiger Name,ID-Nummer,E-Mail-Adresse,Status,Bewertung,Bestwertung,Bewertung kann geändert werden,Zuletzt geändert (Abgabe),Zuletzt geändert (Bewertung),Feedback als Kommentar
Teilnehmer/in1234567,asdf ghjklö,12345678,K12345678@students.jku.at,Zur Bewertung abgegeben,"13,50","24,00",Ja,"Mittwoch, 13. März 2024, 10:15","Mittwoch, 13. März 2024, 19:59",
Teilnehmer/in7654321,"fdsa ölkjh,g",87654321,K87654321@students.jku.at,Zur Bewertung abgegeben,,"24,00",Ja,"Mittwoch, 13. März 2024, 15:10","Mittwoch, 13. März 2024, 19:59",
"#;
}
