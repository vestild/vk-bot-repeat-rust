use std::cmp::PartialEq;
use serde::{Deserialize};
use serde_json;
use crate::error::{*};
use crate::client::{ServerConfig};

pub async fn get_events(client: &reqwest::Client, config: &ServerConfig) -> SimpleResult<Result> {
    let query = [
        ("act", "a_check"),
        ("wait", "25"),
        ("key", &config.key),
        ("ts", &config.ts)
    ];
    let r = client.get(&config.server)
        .query(&query)
        .send()
        .await
        .map_err(|e| Error::new(format!("got error {} on long poll request", e)))?;
    let status = r.status();
    let text = r.text().await.map_err(|e| Error::new(format!("got error {} on long poll request", e)))?;
    let r: Response = serde_json::from_str(&text)
        .map_err(|e| Error::new(format!("{:?} on deserialize <{}> from long poll request, status {}", e, &text, status)))?;
    Ok(r.into())
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum Response {
    Ok { ts: String, updates: Vec<ResponseEvent> },
    Fail { failed: u8, ts: Option<u64> },
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "object")]
enum ResponseEvent {
    #[serde(rename = "board_post_new")]
    BoardPost(BoardPost),
    #[serde(rename = "wall_post_new")]
    WallPost(WallPost),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, PartialEq)]
struct WallPost {
    pub id: i64,
}

#[derive(Debug, Deserialize, PartialEq)]
struct BoardPost {
    pub from_id: i64,
    pub text: String,
    pub topic_id: i64,
    pub id: i64,
}

pub struct Result {
    pub ts: Option<String>,
    pub refresh_key: bool,
    pub refresh_all: bool,
    pub events: Vec<Event>,
}

pub enum Event {
    BoardPost {from_id: i64, text: String, topic_id: i64, id: i64},
    WallPost {id: i64},
}

impl From<ResponseEvent> for Option<Event> {
    fn from(source: ResponseEvent) -> Option<Event> {
        match source {
            ResponseEvent::BoardPost(p) => Some(
                Event::BoardPost {
                    from_id: p.from_id,
                    text: p.text,
                    topic_id: p.topic_id,
                    id: p.id,
                }
            ),
            ResponseEvent::WallPost(o) => Some(
                Event::WallPost {id: o.id}
            ),
            ResponseEvent::Other => None,
        }
    }
}

impl From<Response> for Result {
    fn from(source: Response) -> Result {
        match source {
            Response::Fail {failed, ts} => {
                if failed == 1 && ts.is_some() {
                    Result {
                        ts: Some(ts.unwrap().to_string()),
                        events: vec!(),
                        refresh_all: false,
                        refresh_key: false,
                    }
                } else if failed == 2 {
                    Result {
                        ts: None,
                        events: vec!(),
                        refresh_all: false,
                        refresh_key: true,
                    }
                } else {
                    Result {
                        ts: None,
                        events: vec!(),
                        refresh_all: true,
                        refresh_key: true,
                    }
                }
            },
            Response::Ok {ts, updates} => {
                Result {
                    ts: Some(ts),
                    events: updates.into_iter().filter_map(|x| x.into()).collect(),
                    refresh_all: false,
                    refresh_key: false,
                }
            },
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn deserialize_ok() {
        let source = r#"
{
   "ts":"4",
   "updates":[
      {
         "type":"wall_post_new",
         "object":{
            "id":28
         },
         "group_id":123456
      },
      {
         "type":"board_post_new",
         "object":{
            "from_id":1000,
            "text":"some text",
            "id": 123,
            "topic_id": 456
         },
         "group_id":123456
      }
   ]
}
        "#;
        let result: Response = serde_json::from_str(source).unwrap();
        assert_eq!(Response::Ok{
            ts: "4".to_owned(),
            updates: vec!(
                ResponseEvent::WallPost(WallPost{
                    id: 28,
                }),
                ResponseEvent::BoardPost(BoardPost{
                    from_id: 1000,
                    text: "some text".to_owned(),
                    id: 123,
                    topic_id: 456,
                }),
            )
        }, result);
    }

    #[test]
    fn deserialize_fail() {
        let source = r#"{"failed":1,"ts":30}"#;
        let result: Response = serde_json::from_str(source).unwrap();
        assert_eq!(Response::Fail{
            failed: 1,
            ts: Some(30),
        }, result);
    }
}
