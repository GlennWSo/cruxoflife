use std::borrow::BorrowMut;
use std::collections::HashSet;

use chrono::serde::ts_milliseconds_option::deserialize as ts_milliseconds_option;
use chrono::{DateTime, Utc};
use crux_core::{macros::Effect, render::Render};
use crux_http::Http;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://crux-counter.fly.dev";
const INC_API: &str = "https://crux-counter.fly.dev/inc";
const DEC_API: &str = "https://crux-counter.fly.dev/dec";

#[derive(Default)]
pub struct App;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Count {
    value: i32,
    #[serde(deserialize_with = "ts_milliseconds_option")]
    updated_at: Option<DateTime<Utc>>,
}

/// [row, col]
/// rows are top to bottom
/// cols are left to right
type CellCoord = [i32; 2];

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Life {
    cells: HashSet<CellCoord>,
    spawns: Vec<CellCoord>,
}

impl Life {
    fn add_cells(&mut self, spawns: &[CellCoord]) {
        for cell in spawns {
            self.cells.insert(*cell);
        }
    }
    fn new(init_life: &[CellCoord]) -> Self {
        let mut game = Self::default();
        game.add_cells(init_life);
        game
    }
    fn adjecents(coord: &CellCoord) -> [CellCoord; 8] {
        let [row, col] = coord;
        [
            [row - 1, col - 1],
            [row - 1, col + 0],
            [row - 1, col + 1],
            //
            [row + 0, col - 1],
            // [row + 0, col + 0],
            [row + 0, col + 1],
            //
            [row + 1, col - 1],
            [row + 1, col + 0],
            [row + 1, col + 1],
        ]
    }
    fn cell_birth(&self, coord: &CellCoord) -> bool {
        let adjs = Self::adjecents(coord);
        let count = adjs
            .into_iter()
            .filter(|c| self.cells.get(c).is_some())
            .count() as u8;
        count == 3
    }
    fn save_spawns(&mut self) {
        // self.spawns.clear();
        self.spawns = self
            .cells
            .iter()
            .flat_map(|cell| Self::adjecents(cell))
            .filter(|cell| self.cells.get(cell).is_none())
            .filter(|cell| self.cell_birth(cell))
            .collect();
    }
    fn insert_saved(&mut self) {
        for cell in self.spawns.drain(..) {
            self.cells.insert(cell);
        }
    }
    fn cell_survive(&self, coord: &CellCoord) -> bool {
        let adjs = Self::adjecents(coord);
        let count = adjs
            .into_iter()
            .filter(|c| self.cells.get(c).is_some())
            .count() as u8;
        count == 2 || count == 3
    }
    fn kill_cells(&mut self) {
        let survivors: Box<[_]> = self
            .cells
            .iter()
            .filter(|cell| self.cell_survive(cell))
            .copied()
            .collect();
        self.cells.clear();
        self.cells.extend(survivors);
    }
    fn tick(&mut self) {
        self.save_spawns();
        self.kill_cells();
        self.insert_saved();
    }
}

#[cfg(test)]
mod test_life {
    use super::*;
    fn blinker() -> Life {
        let mut life = Life::default();
        life.add_cells(&[[0, -1], [0, 0], [0, 1]]);
        life
    }
    #[test]
    fn test_blinker_tick() {
        let mut life = blinker();
        {
            life.tick();
            let mut tick: Vec<_> = life.cells.iter().copied().collect();
            tick.sort();
            insta::assert_ron_snapshot!(tick, @r#"
            [
              (-1, 0),
              (0, 0),
              (1, 0),
            ]
            "#);
        }
        {
            life.tick();
            let mut tick: Vec<_> = life.cells.iter().copied().collect();
            tick.sort();
            insta::assert_ron_snapshot!(tick, @r#"
            [
              (0, -1),
              (0, 0),
              (0, 1),
            ]
            "#);
        }
    }
}

#[derive(Default)]
pub struct Model {
    count: Count,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Increment,
    Decrement,
    Get,

    /// this event is private to the core
    #[serde(skip)]
    Set(crux_http::Result<crux_http::Response<Count>>),
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
pub struct Capabilites {
    /// capable of telling shell that viewmodel has been updated for the next rendering
    pub render: Render<Event>,
    /// capable of asking shell to preform http requests
    pub http: Http<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
    count: String,
    confirmed: bool,
}
impl crux_core::App for App {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::Get => {
                caps.http.get(API_URL).expect_json().send(Event::Set);
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
        let app = AppTester::<App, _>::default();
        let mut model = Model::default();

        let update = app.update(Event::Increment, &mut model);

        // Check update asked us to `Render`
        assert_effect!(update, Effect::Render(_));
    }

    #[test]
    fn shows_initial_count() {
        let app = AppTester::<App, _>::default();
        let model = Model::default();

        let actual_view = app.view(&model).count;
        let expected_view = "0 Pending...";
        assert_eq!(actual_view, expected_view);
    }

    #[test]
    fn resets_count() {
        let app = AppTester::<App, _>::default();
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
        let app = AppTester::<App, _>::default();
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
