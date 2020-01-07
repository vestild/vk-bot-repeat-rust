use std::cmp::PartialEq;
use serde::{Deserialize};
use serde_json;

#[derive(Debug, Deserialize, PartialEq)]
struct Response {
    pub ts: String,
    pub updates: Vec<Event>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "object")]
enum Event {
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
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn deserialize() {
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
            "text":"some text"
         },
         "group_id":123456
      }
   ]
}
        "#;
        let result: Response = serde_json::from_str(source).unwrap();
        assert_eq!(Response{
            ts: "4".to_owned(),
            updates: vec!(
                Event::WallPost(WallPost{
                    id: 28,
                }),
                Event::BoardPost(BoardPost{
                    from_id: 1000,
                    text: "some text".to_owned()
                }),
            )
        }, result);
    }
}

pub struct Client {
    client: reqwest::Client,
}