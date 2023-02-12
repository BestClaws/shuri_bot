use reqwest;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub word: String,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
    pub example: Option<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

pub async fn dict(query: &str) -> Result<Vec<QueryResponse>, reqwest::Error> {
    let meaning: Vec<QueryResponse> = reqwest::Client::new()
        .get(format!(
            "https://api.dictionaryapi.dev/api/v2/entries/en/{query}"
        ))
        .send()
        .await?
        .json()
        .await?;

    Ok(meaning)
}


#[cfg(test)]
mod test {

    use crate::dictionary::QueryResponse;
    #[test]
    fn test() {
        let raw_data = r#"[{"word":"hello","phonetics":[{"audio":"https://api.dictionaryapi.dev/media/pronunciations/en/hello-au.mp3","sourceUrl":"https://commons.wikimedia.org/w/index.php?curid=75797336","license":{"name":"BY-SA 4.0","url":"https://creativecommons.org/licenses/by-sa/4.0"}},{"text":"/həˈləʊ/","audio":"https://api.dictionaryapi.dev/media/pronunciations/en/hello-uk.mp3","sourceUrl":"https://commons.wikimedia.org/w/index.php?curid=9021983","license":{"name":"BY 3.0 US","url":"https://creativecommons.org/licenses/by/3.0/us"}},{"text":"/həˈloʊ/","audio":""}],"meanings":[{"partOfSpeech":"noun","definitions":[{"definition":"\"Hello!\" or an equivalent greeting.","synonyms":[],"antonyms":[]}],"synonyms":["greeting"],"antonyms":[]},{"partOfSpeech":"verb","definitions":[{"definition":"To greet with \"hello\".","synonyms":[],"antonyms":[]}],"synonyms":[],"antonyms":[]},{"partOfSpeech":"interjection","definitions":[{"definition":"A greeting (salutation) said when meeting someone or acknowledging someone’s arrival or presence.","synonyms":[],"antonyms":[],"example":"Hello, everyone."},{"definition":"A greeting used when answering the telephone.","synonyms":[],"antonyms":[],"example":"Hello? How may I help you?"},{"definition":"A call for response if it is not clear if anyone is present or listening, or if a telephone conversation may have been disconnected.","synonyms":[],"antonyms":[],"example":"Hello? Is anyone there?"},{"definition":"Used sarcastically to imply that the person addressed or referred to has done something the speaker or writer considers to be foolish.","synonyms":[],"antonyms":[],"example":"You just tried to start your car with your cell phone. Hello?"},{"definition":"An expression of puzzlement or discovery.","synonyms":[],"antonyms":[],"example":"Hello! What’s going on here?"}],"synonyms":[],"antonyms":["bye","goodbye"]}],"license":{"name":"CC BY-SA 3.0","url":"https://creativecommons.org/licenses/by-sa/3.0"},"sourceUrls":["https://en.wiktionary.org/wiki/hello"]}]"#;
        // TODO: this is not even a test. hello? fix it.
        match serde_json::from_str::<Vec<QueryResponse>>(raw_data) {
            Ok(v) => println!("{v:?}"),
            Err(e) => {
                println!("{e:?}");
            }
        }
    }
}
