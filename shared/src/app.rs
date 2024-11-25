use chrono::serde::ts_milliseconds_option::deserialize as ts_milliseconds_option;
use chrono::{DateTime, Utc};
use crux_core::{macros::Effect, render::Render, App};
use crux_http::Http;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://crux-counter.fly.dev";
const INC_API: &str = "https://crux-counter.fly.dev/inc";
const DEC_API: &str = "https://crux-counter.fly.dev/dec";

#[derive(Default)]
pub struct Counter;

#[derive(Default, Serialize, Deserialize)]
pub struct Count {
    value: i32,
    #[serde(deserialize_with = "ts_milliseconds_option")]
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct Model {
    count: Count,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Increment,
    Decrement,
    Get,

    /// this event is private to the core
    #[serde(skip)]
    Set(crux_http::Result<crux_http::Response<Count>>),
}

#[derive(Effect)]
pub struct Capabilites {
    pub render: Render<Event>,
    pub http: Http<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
    count: String,
    confirmed: bool,
}
impl App for Counter {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::Get => {
                caps.http.get(API_URL).expect_json().send(Event::Set);
                return;
            }
            Event::Increment => {
                model.count.value += 1;
                model.count.updated_at = None;
                caps.render.render();

                caps.http.post(INC_API).expect_json().send(Event::Set);
            }
            Event::Decrement => {
                model.count.value -= 1;
                model.count.updated_at = None;
                caps.render.render();

                caps.http.post(DEC_API).expect_json().send(Event::Set);
            }
            Event::Set(Ok(mut response)) => {
                model.count = response.take_body().unwrap();
                caps.render.render()
            }
            Event::Set(Err(e)) => {
                panic!("Oh no: {}", e);
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let suffix = match model.count.updated_at {
            Some(date) => format!("Confirmed: {date}"),
            None => "Pending...".to_string(),
        };

        ViewModel {
            count: format!("{} {}", model.count.value, suffix),
            confirmed: model.count.updated_at.is_some(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::testing::ResponseBuilder;

    #[test]
    fn renders() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();

        let update = app.update(Event::Increment, &mut model);

        // Check update asked us to `Render`
        assert_effect!(update, Effect::Render(_));
    }

    #[test]
    fn shows_initial_count() {
        let app = AppTester::<Counter, _>::default();
        let model = Model::default();

        let actual_view = app.view(&model).count;
        let expected_view = "0 Pending...";
        assert_eq!(actual_view, expected_view);
    }

    #[test]
    fn resets_count() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();
        let _ = app.update(Event::Increment, &mut model);
        let expected_view = "0 Pending...";
        {
            let view0 = app.view(&model).count;
            assert_ne!(view0, expected_view);
        }
        let count = Count {
            value: 0,
            updated_at: Some(DateTime::from_timestamp_nanos(0)),
        };
        let fake_response = ResponseBuilder::ok().body(count).build();
        let _ = app.update(Event::Set(Ok(fake_response)), &mut model);
        // assert_eq!(reset_view, expected_view);
        // check that the view has been updated correctly
        insta::assert_ron_snapshot!(app.view(&model), @r#"
        ViewModel(
          count: "0 Confirmed: 1970-01-01 00:00:00 UTC",
          confirmed: true,
        )
        "#);
    }

    #[test]
    fn counts_up_and_down() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();

        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Decrement, &mut model);
        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Increment, &mut model);

        let actual_view = app.view(&model).count;
        let expected_view = "2 Pending...";
        assert_eq!(actual_view, expected_view);
    }
}
