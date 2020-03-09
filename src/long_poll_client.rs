use crate::client::ServerConfig;
use crate::error::*;
use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_json;
use std::cmp::PartialEq;

pub async fn get_events(client: &reqwest::Client, config: &ServerConfig) -> SimpleResult<Result> {
    let query = [
        ("act", "a_check"),
        ("wait", "25"),
        ("key", &config.key),
        ("ts", &config.ts),
    ];
    let r = client
        .get(&config.server)
        .query(&query)
        .send()
        .await
        .map_err(|e| Error::new(format!("got error {} on long poll request", e)))?;
    let status = r.status();
    let text = r
        .text()
        .await
        .map_err(|e| Error::new(format!("got error {} on long poll request", e)))?;
    let r: Response = serde_json::from_str(&text).map_err(|e| {
        Error::new(format!(
            "{:?} on deserialize <{}> from long poll request, status {}",
            e, &text, status
        ))
    })?;
    Ok(r.into())
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum Response {
    Ok {
        ts: String,
        updates: Vec<ResponseEventWrapper>,
    },
    Fail {
        failed: u8,
        ts: Option<u64>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponseEventWrapper {
    Some(ResponseEvent),
    Unknown(IgnoredAny),
}

impl PartialEq for ResponseEventWrapper {
    fn eq(&self, other: &Self) -> bool {
        if let ResponseEventWrapper::Unknown(_) = self {
            if let ResponseEventWrapper::Unknown(_) = other {
                return true;
            }
        }
        if let ResponseEventWrapper::Some(e) = self {
            if let ResponseEventWrapper::Some(e1) = other {
                return e.eq(e1);
            }
        }
        false
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "object")]
enum ResponseEvent {
    #[serde(rename = "board_post_new")]
    BoardPost {
        from_id: i64,
        text: String,
        topic_id: i64,
        id: i64,
    },
    #[serde(rename = "wall_post_new")]
    WallPost { id: i64 },
}

pub struct Result {
    pub ts: Option<String>,
    pub refresh_key: bool,
    pub refresh_all: bool,
    pub events: Vec<Event>,
}

pub enum Event {
    BoardPost {
        from_id: i64,
        text: String,
        topic_id: i64,
        id: i64,
    },
    WallPost {
        id: i64,
    },
}

impl From<ResponseEventWrapper> for Option<Event> {
    fn from(source: ResponseEventWrapper) -> Option<Event> {
        match source {
            ResponseEventWrapper::Some(e) => match e {
                ResponseEvent::BoardPost {
                    from_id,
                    text,
                    topic_id,
                    id,
                } => Some(Event::BoardPost {
                    from_id,
                    text,
                    topic_id,
                    id,
                }),
                ResponseEvent::WallPost { id } => Some(Event::WallPost { id }),
            },
            ResponseEventWrapper::Unknown(_) => None,
        }
    }
}

impl From<Response> for Result {
    fn from(source: Response) -> Result {
        match source {
            Response::Fail { failed, ts } => {
                if failed == 1 && ts.is_some() {
                    Result {
                        ts: Some(ts.unwrap().to_string()),
                        events: vec![],
                        refresh_all: false,
                        refresh_key: false,
                    }
                } else if failed == 2 {
                    Result {
                        ts: None,
                        events: vec![],
                        refresh_all: false,
                        refresh_key: true,
                    }
                } else {
                    Result {
                        ts: None,
                        events: vec![],
                        refresh_all: true,
                        refresh_key: true,
                    }
                }
            }
            Response::Ok { ts, updates } => Result {
                ts: Some(ts),
                events: updates.into_iter().filter_map(|x| x.into()).collect(),
                refresh_all: false,
                refresh_key: false,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn wall(id: i64) -> ResponseEventWrapper {
        ResponseEventWrapper::Some(ResponseEvent::WallPost { id })
    }

    fn board(from_id: i64, text: String, topic_id: i64, id: i64) -> ResponseEventWrapper {
        ResponseEventWrapper::Some(ResponseEvent::BoardPost {
            id,
            from_id,
            text,
            topic_id,
        })
    }

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
        assert_eq!(
            Response::Ok {
                ts: "4".to_owned(),
                updates: vec!(wall(28), board(1000, "some text".to_owned(), 456, 123),)
            },
            result
        );
    }

    #[test]
    fn deserialize_fail() {
        let source = r#"{"failed":1,"ts":30}"#;
        let result: Response = serde_json::from_str(source).unwrap();
        assert_eq!(
            Response::Fail {
                failed: 1,
                ts: Some(30),
            },
            result
        );
    }

    #[test]
    fn deserialize_event() {
        let source = r#"
        {
         "type":"wall_post_new",
         "object":{
            "id":28
         },
         "group_id":123456
        }"#;
        let result: ResponseEvent = serde_json::from_str(source).unwrap();
        assert_eq!(ResponseEvent::WallPost { id: 28 }, result);
    }

    #[test]
    fn deserialize_other_event() {
        let source = r#"
        {
         "type":"other",
         "object":"",
         "group_id":123456
        }"#;
        let result: ResponseEventWrapper = serde_json::from_str(source).unwrap();
        assert_eq!(ResponseEventWrapper::Unknown(IgnoredAny::default()), result);
    }

    #[test]
    fn deserialize_other() {
        let source = r#"
{
   "ts":"4",
   "updates":[
      {
         "type":"message_new",
         "object":{
            "date":1578870439,
            "from_id":5848319,
            "id":0,
            "out":0,
            "peer_id":2000000001,
            "text":"sdfsdf",
            "conversation_message_id":29,
            "fwd_messages":[

            ],
            "important":false,
            "random_id":0,
            "attachments":[

            ],
            "is_hidden":false
         },
         "group_id":121322600,
         "event_id":"d5ad121479d4814cb01dc700cdd25f1d0b806355"
      }
   ]
}"#;
        let result: Response = serde_json::from_str(source).unwrap();
        assert_eq!(
            Response::Ok {
                ts: "4".to_owned(),
                updates: vec!(ResponseEventWrapper::Unknown(IgnoredAny::default()))
            },
            result
        );
    }
}
