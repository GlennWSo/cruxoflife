use std::ops::BitOr;
use std::{collections::HashSet, fmt::Display};

use crux_core::{macros::Effect, render::Render};
use crux_http::Http;
use serde::{Deserialize, Serialize};

mod capabilities;
use capabilities::{Alert, FileIO};

#[derive(Default)]
pub struct App;

type CellCoord = [i32; 2];

type CellSet = HashSet<CellCoord>;

type CellVector = Vec<CellCoord>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Life {
    state: CellSet,
    buffer: CellVector,
}

const INIT_LIFE: &[u8] = include_bytes!("../../init_life.json");

impl Default for Life {
    fn default() -> Self {
        let life = Life::from_bytes(INIT_LIFE);
        // life.flip_rows();
        // life.translate(&[5, 5]);
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

/// Life forms
#[allow(dead_code)]
impl Life {
    fn from_bytes(data: &[u8]) -> Life {
        let coords: CellVector = serde_json::from_slice(data).unwrap();
        let mut life = Self::empty();
        life.add_cells(&coords);
        life
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
}

impl Life {
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
    fn clear(&mut self) {
        self.state.clear();
        self.buffer.clear();
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
    life: Life,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Render,
    Step,
    Echo(String),
    ToggleCell(CellCoord),
    SpawnGlider(CellCoord),
    SaveWorld,
    LoadWorld(Vec<u8>),
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
pub struct Capabilites {
    /// capable of telling shell that viewmodel has been updated for the next rendering
    pub render: Render<Event>,
    /// capable of asking shell to preform http requests
    alert: Alert<Event>,
    file_io: FileIO<Event>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub life: Vec<CellCoord>,
}
impl Display for ViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.life.is_empty() {
            return write!(f, "Empty");
        }
        let r = self.life.iter().max_by(|a, b| a[0].cmp(&b[0])).unwrap()[0];

        let c = self.life.iter().max_by(|a, b| a[1].cmp(&b[1])).unwrap()[1];
        write!(f, "r: {r} c: {c}")
    }
}

impl crux_core::App for App {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::Render => {
                caps.render.render();
            }
            Event::LoadWorld(data) => {
                let coords: CellVector = serde_json::from_slice(data.as_slice()).unwrap();
                model.life.clear();
                model.life.add_cells(&coords);
                caps.render.render();
            }
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
            Event::Step => {
                model.life.tick();
                caps.render.render();
            }
            Event::SpawnGlider(_coord) => todo!(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            life: model.life.state.iter().copied().collect(),
        }
    }
}
