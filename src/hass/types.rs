use std::borrow::Cow;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

pub trait Service: Serialize {
    type Output: for<'de> Deserialize<'de>;
    fn domain(&self) -> &str;
    fn service(&self) -> &str;
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct State<'a, Attrs = serde_json::Map<String, serde_json::Value>> {
    #[serde(flatten)]
    pub entity: Entity<'a>,
    pub state: Cow<'a, str>,
    pub last_changed: DateTime<FixedOffset>,
    pub last_updated: DateTime<FixedOffset>,
    pub context: Context<'a>,
    pub attributes: Attrs,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Context<'a> {
    id: Cow<'a, str>,
    parent_id: Option<Cow<'a, str>>,
    user_id: Option<Cow<'a, str>>,
}

/// ## Note:
/// You'll usually use this with `#[serde(flatten)]`
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Entity<'a> {
    #[serde(rename = "entity_id")]
    pub id: Cow<'a, str>,
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::{Context, Entity, State};

    #[test]
    fn serialize_state_simple() {
        let state = State {
            entity: Entity { id: "test".into() },
            state: "some state".into(),
            last_changed: DateTime::UNIX_EPOCH.fixed_offset(),
            last_updated: DateTime::<Utc>::MIN_UTC.fixed_offset(),
            context: Context {
                id: "some id".into(),
                parent_id: None,
                user_id: Some("some user id".into()),
            },
            attributes: json!({}),
        };
        assert_eq!(
            serde_json::to_value(state).unwrap(),
            json! ({
                "entity_id": "test",
                "state": "some state",
                "last_changed": DateTime::UNIX_EPOCH.fixed_offset(),
                "last_updated": DateTime::<Utc>::MIN_UTC.fixed_offset(),
                "context": {
                    "id": "some id",
                    "parent_id": null,
                    "user_id": "some user id",
                },
                "attributes": {},
            }),
        );
    }

    #[allow(deprecated)]
    #[test]
    fn deserialize_state_arr() {
        let states = json!([
            {
                "attributes": {},
                "entity_id": "sun.sun",
                "last_changed": "2016-05-30T21:43:32.418320+00:00",
                "last_updated": "2016-05-30T21:43:32.418320+00:00",
                "state": "below_horizon",
                "context": {
                    "id": "",
                    "parent_id": null,
                    "user_id": null,
                },
            },
            {
                "attributes": {},
                "entity_id": "process.Dropbox",
                "last_changed": "2016-05-30T21:43:32.418320+00:00",
                "last_updated": "2016-05-30T21:43:32.418320+00:00",
                "state": "on",
                "context": {
                    "id": "",
                    "parent_id": null,
                    "user_id": null,
                },
            },
        ]);
        assert_eq!(
            serde_json::from_value::<Vec<State<serde_json::Value>>>(states).unwrap(),
            vec![
                State {
                    entity: Entity {
                        id: "sun.sun".into()
                    },
                    last_changed: NaiveDate::from_ymd(2016, 5, 30)
                        .and_hms_micro(21, 43, 32, 418_320)
                        .and_utc()
                        .fixed_offset(),
                    last_updated: NaiveDate::from_ymd(2016, 5, 30)
                        .and_hms_micro(21, 43, 32, 418_320)
                        .and_utc()
                        .fixed_offset(),
                    attributes: json!({}),
                    state: "below_horizon".into(),
                    context: Context {
                        id: "".into(),
                        parent_id: None,
                        user_id: None
                    }
                },
                State {
                    entity: Entity {
                        id: "process.Dropbox".into()
                    },
                    last_changed: NaiveDate::from_ymd(2016, 5, 30)
                        .and_hms_micro(21, 43, 32, 418_320)
                        .and_utc()
                        .fixed_offset(),
                    last_updated: NaiveDate::from_ymd(2016, 5, 30)
                        .and_hms_micro(21, 43, 32, 418_320)
                        .and_utc()
                        .fixed_offset(),
                    attributes: json!({}),
                    state: "on".into(),
                    context: Context {
                        id: "".into(),
                        parent_id: None,
                        user_id: None
                    }
                }
            ]
        );
    }

    #[allow(deprecated)]
    #[test]
    fn deserialize_state() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct SunAttrs {
            azimuth: f64,
            elevation: f64,
            friendly_name: String,
            next_rising: DateTime<FixedOffset>,
            next_setting: DateTime<FixedOffset>,
        }
        let val = json!({
           "attributes":{
              "azimuth": 336.34_f64,
              "elevation": -17.67_f64,
              "friendly_name": "Sun",
              "next_rising": "2016-05-31T03:39:14+00:00",
              "next_setting": "2016-05-31T19:16:42+00:00"
           },
           "entity_id": "sun.sun",
           "last_changed": "2016-05-30T21:43:29.204838+00:00",
           "last_updated": "2016-05-30T21:50:30.529465+00:00",
           "state":"below_horizon",
            "context": {
                "id": "",
                "parent_id": null,
                "user_id": null,
            },

        });
        assert_eq!(
            serde_json::from_value::<State<_>>(val).unwrap(),
            State {
                entity: Entity {
                    id: "sun.sun".into()
                },
                last_changed: NaiveDate::from_ymd(2016, 5, 30)
                    .and_hms_micro(21, 43, 29, 204_838)
                    .and_utc()
                    .fixed_offset(),
                last_updated: NaiveDate::from_ymd(2016, 5, 30)
                    .and_hms_micro(21, 50, 30, 529_465)
                    .and_utc()
                    .fixed_offset(),
                state: "below_horizon".into(),
                attributes: SunAttrs {
                    azimuth: 336.34,
                    elevation: -17.67,
                    friendly_name: "Sun".to_owned(),
                    next_rising: NaiveDate::from_ymd(2016, 5, 31)
                        .and_hms(3, 39, 14)
                        .and_utc()
                        .fixed_offset(),
                    next_setting: NaiveDate::from_ymd(2016, 5, 31)
                        .and_hms(19, 16, 42)
                        .and_utc()
                        .fixed_offset(),
                },
                context: Context {
                    id: "".into(),
                    user_id: None,
                    parent_id: None,
                }
            }
        );
    }

    #[test]
    fn serialize_entity() {
        #[derive(Serialize, Default)]
        struct Payload<'a> {
            #[serde(flatten)]
            entity: Option<Entity<'a>>,
        }

        impl<'a> From<Entity<'a>> for Payload<'a> {
            fn from(value: Entity<'a>) -> Self {
                Self {
                    entity: Some(value),
                }
            }
        }

        assert_eq!(
            serde_json::to_value(Payload::from(Entity {
                id: "button.open_door".into()
            }))
            .unwrap(),
            json!({
                "entity_id": "button.open_door",
            })
        );

        assert_eq!(serde_json::to_value(Payload::default()).unwrap(), json!({}));
    }
}
