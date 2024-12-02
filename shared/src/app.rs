use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::BitOr;

use chrono::serde::ts_milliseconds_option::deserialize as ts_milliseconds_option;
use chrono::{DateTime, Utc};
use crux_core::capability::{CapabilityContext, Operation};
use crux_core::macros::Capability;
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

type CellSet = HashSet<CellCoord>;

type CellVector = Vec<CellCoord>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Life {
    state: CellSet,
    buffer: CellVector,
}

impl Default for Life {
    fn default() -> Self {
        let mut life = Self::glider();
        life.flip_rows();
        life.translate(&[5, 5]);
        life
    }
}

impl BitOr for Life {
    type Output = Life;
    fn bitor(mut self, mut rhs: Self) -> Self::Output {
        // let cells = self.cells.union(&rhs.cells).into();
        self.state.extend(rhs.state.drain());
        // let cells = self.cells.extend()
        let buffer = self.buffer;
        Self {
            state: self.state,
            buffer,
        }
    }
}

impl Life {
    // fn merge(&self, other: Life) {
    //     let derp = self.or(other);
    // }
    fn translate(&mut self, delta: &CellCoord) {
        self.buffer.clear();
        self.buffer.extend(
            self.state
                .drain()
                .map(|cell| [cell[0] + delta[0], cell[1] + delta[1]]),
        );
        self.state.extend(self.buffer.drain(..));
    }
    fn flip_rows(&mut self) {
        self.buffer.clear();
        self.buffer
            .extend(self.state.drain().map(|cell| [-cell[0], cell[1]]));
        self.state.extend(self.buffer.drain(..));
    }
    fn empty() -> Self {
        Self {
            state: HashSet::new(),
            buffer: Vec::new(),
        }
    }
    fn add_cells(&mut self, spawns: &[CellCoord]) {
        for cell in spawns {
            self.state.insert(*cell);
        }
    }
    fn new(init_life: &[CellCoord]) -> Self {
        let mut game = Self::empty();
        game.add_cells(init_life);
        game
    }
    fn blinker() -> Self {
        Self::new(&[[0, -1], [0, 0], [0, 1]])
    }
    fn tub() -> Self {
        Self::new(&[[0, -1], [0, 1], [-1, 0], [1, 0]])
    }
    fn glider() -> Self {
        Self::new(&[
            [0, 0], //
            [-1, 1],
            [-1, 2],
            [0, 2],
            [1, 2],
        ])
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
            .filter(|c| self.state.get(c).is_some())
            .count() as u8;
        count == 3
    }
    fn save_spawns(&mut self) {
        // self.spawns.clear();
        self.buffer = self
            .state
            .iter()
            .flat_map(|cell| Self::adjecents(cell))
            .filter(|cell| self.state.get(cell).is_none())
            .filter(|cell| self.cell_birth(cell))
            .collect();
    }
    fn insert_saved(&mut self) {
        for cell in self.buffer.drain(..) {
            self.state.insert(cell);
        }
    }
    fn cell_survive(&self, coord: &CellCoord) -> bool {
        let adjs = Self::adjecents(coord);
        let count = adjs
            .into_iter()
            .filter(|c| self.state.get(c).is_some())
            .count() as u8;
        count == 2 || count == 3
    }
    fn kill_cells(&mut self) {
        let survivors: Box<[_]> = self
            .state
            .iter()
            .filter(|cell| self.cell_survive(cell))
            .copied()
            .collect();
        self.state.clear();
        self.state.extend(survivors);
    }
    fn tick(&mut self) {
        self.save_spawns();
        self.kill_cells();
        self.insert_saved();
    }
    fn toggle_cell(&mut self, coord: CellCoord) {
        if !self.state.remove(&coord) {
            self.state.insert(coord);
        }
    }
    fn state_as_list(&self) -> CellVector {
        self.state.iter().copied().collect()
    }
}

#[cfg(test)]
mod test_life {
    use super::*;
    #[test]
    /// make sure static life is static
    fn test_tub() {
        let mut life = Life::tub();
        let expected = life.clone();
        for _ in 0..17 {
            life.tick();
            assert_eq!(life, expected);
        }
    }

    #[test]
    fn test_translate() {
        let mut life = Life::tub();
        life.translate(&[5, 5]);
        life.translate(&[5, 5]);
        let mut tick: Vec<_> = life.state.iter().copied().collect();
        tick.sort();
        insta::assert_ron_snapshot!(tick, @r#"
        [
          (9, 10),
          (10, 9),
          (10, 11),
          (11, 10),
        ]
        "#);
    }

    #[test]
    fn test_blinker_tick() {
        let mut life = Life::blinker();
        {
            life.tick();
            let mut cells: Vec<_> = life.state.iter().copied().collect();
            cells.sort();
            insta::assert_ron_snapshot!(cells, @r#"
            [
              (-1, 0),
              (0, 0),
              (1, 0),
            ]
            "#);
        }
        {
            life.tick();
            let mut tick: Vec<_> = life.state.iter().copied().collect();
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
    life: Life,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Increment,
    Decrement,
    Get,
    Step,
    Echo(String),
    ToggleCell(CellCoord),
    SpawnGlider(CellCoord),
    SaveWorld,

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
    alert: Alert<Event>,
    file_io: FileIO<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
    count: String,
    confirmed: bool,
    life: HashSet<CellCoord>,
}
impl crux_core::App for App {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::SaveWorld => {
                caps.file_io.save(model);
                caps.alert.info("save data sent to shell".to_string());
            }
            Event::Echo(msg) => {
                caps.alert.info(msg);
            }
            Event::ToggleCell(coord) => {
                model.life.toggle_cell(coord);
                caps.render.render();
            }
            Event::Get => {
                caps.http.get(API_URL).expect_json().send(Event::Set);
            }
            Event::Step => {
                model.life.tick();
                caps.render.render();
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
            Event::SpawnGlider(coord) => todo!(),
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
            life: model.life.state.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
enum FileOperation {
    Save(CellVector),
}

impl Operation for FileOperation {
    type Output = Option<CellVector>;
}

#[derive(Capability)]
struct FileIO<Event> {
    context: CapabilityContext<FileOperation, Event>,
}

impl<Event> FileIO<Event> {
    fn new(context: CapabilityContext<FileOperation, Event>) -> Self {
        Self { context }
    }
    fn save(&self, model: &Model)
    where
        Event: 'static,
    {
        let ctx = self.context.clone();
        let save_state = model.life.state_as_list();
        self.context.spawn(async move {
            // Instruct Shell to get ducks in a row and await the ducks
            ctx.request_from_shell(FileOperation::Save(save_state))
                .await;
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
enum AlertOpereation {
    Info(String),
    Warning(String),
    Error(String),
}

impl Operation for AlertOpereation {
    type Output = ();
}

#[derive(Capability)]
struct Alert<Event> {
    context: CapabilityContext<AlertOpereation, Event>,
}

impl<Event> Alert<Event> {
    pub fn new(context: CapabilityContext<AlertOpereation, Event>) -> Self {
        Self { context }
    }
    pub fn info(&self, msg: String)
    where
        Event: 'static,
    {
        let ctx = self.context.clone();
        // Start a shell interaction
        self.context.spawn(async move {
            // Instruct Shell to get ducks in a row and await the ducks
            ctx.request_from_shell(AlertOpereation::Info(msg)).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::testing::ResponseBuilder;

    #[test]
    fn json_life() {
        let life = Life::glider();
        let mut cells_list = dbg!(life.state_as_list());
        cells_list.sort();

        insta::assert_json_snapshot!(dbg!(cells_list), @r#"
        [
          [
            -1,
            1
          ],
          [
            -1,
            2
          ],
          [
            0,
            0
          ],
          [
            0,
            2
          ],
          [
            1,
            2
          ]
        ]
        "#);
        // panic!();
    }

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
