use std::ops::BitOr;
use std::{collections::HashSet, fmt::Display};

use cgmath::num_traits::Float;
use cgmath::{Array, Vector2};
use crux_core::{macros::Effect, render::Render};
use serde::{Deserialize, Serialize};

mod capabilities;
pub use capabilities::ExportOperation;
use capabilities::{Alert, FileIO};
#[allow(unused)]
// use log::{debug, error, info, warn};
use uniffi::deps::log::{debug, info};

#[derive(Default)]
pub struct App;

/// [row, column]
type CellCoord = [i32; 2];

type CellSet = HashSet<CellCoord>;

type CellVector = Vec<CellCoord>;
pub type Vec2 = Vector2<f32>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Life {
    state: CellSet,
    buffer: CellVector,
}

const INIT_LIFE: &[u8] = include_bytes!("../../init_life.json");

impl Default for Life {
    fn default() -> Self {
        let mut life = Life::from_bytes(INIT_LIFE);
        // life.flip_rows();
        life.translate(&[-7, -10]);
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
    camera: Camera,
}

struct Camera {
    /// halfsize of screen
    screen_size: Vec2,
    /// camera pos in world space
    pan: Vec2,
    /// drag start in world space
    drag_start: Vec2,
    /// (world_size) * zoom = screen_size
    zoom: f32,
}
impl Default for Camera {
    fn default() -> Self {
        let screen_size = Vec2::new(300.0, 300.0);
        let pan = -screen_size / 2.0;
        Self {
            screen_size,
            pan,
            drag_start: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}
impl Camera {
    /// cell size in world
    const CELL_SIZE: f32 = 30.0;
    /// cell size in screen space
    const fn cell_size(&self) -> f32 {
        Self::CELL_SIZE * self.zoom
    }

    const fn cell2world(&self, cell: &CellCoord) -> Vec2 {
        let x = cell[1] as f32 * Self::CELL_SIZE;
        let y = cell[0] as f32 * Self::CELL_SIZE;
        Vec2::new(x, y)
    }
    fn world2screen(&self, world_pos: &Vec2) -> Vec2 {
        let screen = (world_pos - self.pan) * self.zoom; //+ self.screen_size;
        screen
    }
    fn cell2creen(&self, cell: &CellCoord) -> Vec2 {
        let pos = self.cell2world(cell);
        self.world2screen(&pos)
    }

    fn screen2world(&self, screen_pos: &Vec2) -> Vec2 {
        (screen_pos) / self.zoom + self.pan
    }
    fn world2cell(&self, world_pos: &Vec2) -> CellCoord {
        let column = (world_pos.x / Self::CELL_SIZE).floor() as i32;
        let row = (world_pos.y / Self::CELL_SIZE).floor() as i32;
        [row, column]
    }
    fn screen2cell(&self, screen_pos: &Vec2) -> CellCoord {
        let world_pos = self.screen2world(screen_pos);
        self.world2cell(&world_pos)
    }
    /// returns (min, max_coordd
    fn cell_bounds(&self) -> (CellCoord, CellCoord) {
        let min = self.screen2cell(&Vec2::new(0.0, 0.0));
        let max = self.screen2cell(&(self.screen_size * 2.0));
        (min, max)
    }
    /// camera offset in screen space
    fn pan(&self) -> Vec2 {
        self.pan * self.zoom
    }
    fn set_drag_start(&mut self, screen_pos: Vec2) {
        self.drag_start = self.screen2world(&screen_pos);
    }

    /// camera offset in screen space
    fn drag_start(&self) -> Vec2 {
        self.drag_start * self.zoom
    }

    fn set_cam_pos(&mut self, new_pos: impl Into<Vec2>) {
        let new_pos: Vec2 = new_pos.into();
        self.pan = new_pos / self.zoom;
    }
    fn drag_cam(&mut self, screen_drag: Vec2) {
        let new_pos = -screen_drag / self.zoom + self.drag_start;
        self.pan = new_pos;
    }

    fn set_zoom(&mut self, new_zoom: f32) {
        self.pan += self.screen_size / self.zoom - self.screen_size / new_zoom;
        self.zoom = new_zoom;
    }
    // fn set_pan_zoom(&mut self, new_zoom: f32, new_pan: impl Into<Vec2>) {

    // }

    pub fn grid_mod(&self) -> Vec2 {
        (-self.pan() % self.cell_size()) - Vec2::from_value(self.cell_size())
    }
}
#[cfg(test)]
mod test_camera_transforms {
    use super::*;

    #[test]
    fn test_world2screen_inversion() {
        let mut camera = Camera::default();
        camera.zoom = 1.5;
        camera.pan = Vec2::new(30.0, 11.0);

        let screen_pos = Vec2::new(200.0, 150.0);
        let world_pos = camera.screen2world(&screen_pos);
        assert_ne!(screen_pos, world_pos);
        assert_eq!(screen_pos, camera.world2screen(&world_pos));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[allow(deprecated)]
pub enum Event {
    Render,
    Step,
    Echo(String),
    ToggleCell(CellCoord),
    SpawnGlider(CellCoord),
    SaveWorld,
    CopyWorld,
    LoadWorld(Vec<u8>),
    CameraPan([f32; 2]),
    CameraSize([f32; 2]),
    #[deprecated]
    CameraZoom(f32),
    #[deprecated]
    CameraPanZoom([f32; 3]),
    ChangeZoom(f32),
    /// pan change + new zoom setting
    ChangePanZoom([f32; 3]),
    /// signal camera drag stop
    AnchorDrag([f32; 2]),
    ToggleScreenCoord([f32; 2]),
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
pub struct Capabilites {
    /// capable of telling shell that viewmodel has been updated for the next rendering
    pub render: Render<Event>,
    /// capable of asking shell to preform http requests
    alert: Alert<Event>,
    pub file_io: FileIO<Event>,
}

// #[derive(Serialize, Deserialize, Clone)]
// struct LifeView {
//     coords: [f32; 2],
//     size: f32,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewModel {
    pub cell_coords: Vec<[f32; 2]>,
    /// camera position in screen scale
    pub camera_pan: [f32; 2],
    pub cell_size: f32,
    pub modx: f32,
    pub mody: f32,
}
impl Display for ViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cell_coords.is_empty() {
            return write!(f, "Empty");
        }
        // todo!()
        write!(f, "derp")
        // // let r = self.life.iter().max_by(|a, b| a[0].cmp(&b[0])).unwrap()[0];

        // // let c = self.life.iter().max_by(|a, b| a[1].cmp(&b[1])).unwrap()[1];
    }
}

impl ViewModel {
    // pub fn modx(&self) -> f32 {
    //     -self.camera_pan[0] % self.cell_size - self.cell_size
    // }
    // pub fn mody(&self) -> f32 {
    //     -self.camera_pan[1] % self.cell_size - self.cell_size
    // }
}

impl crux_core::App for App {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        info!("got event: {event:?}");
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
            }
            Event::CopyWorld => {
                caps.file_io.copy(model);
            }
            Event::Echo(msg) => {
                caps.alert.info(msg);
            }
            Event::ToggleCell(coord) => {
                model.life.toggle_cell(coord);
                caps.render.render();
            }
            Event::ToggleScreenCoord(screen_pos) => {
                let world_pos = model.camera.screen2world(&screen_pos.into());
                let coord = model.camera.world2cell(&world_pos);
                model.life.toggle_cell(coord);
                caps.render.render();
            }
            Event::Step => {
                model.life.tick();
                caps.render.render();
            }
            Event::SpawnGlider(_coord) => todo!(),
            Event::CameraSize(size) => {
                let new_size = size.map(|e| e / 2.0).into();
                let world_size_diff = (new_size - model.camera.screen_size) / model.camera.zoom;
                model.camera.pan -= world_size_diff;
                model.camera.screen_size = new_size;
                caps.render.render()
            }
            Event::CameraPan(new_pos) => {
                model.camera.set_cam_pos(new_pos);
                caps.render.render();
            }
            Event::CameraZoom(z) => {
                model.camera.set_zoom(z);
                caps.render.render()
            }
            #[allow(deprecated)]
            Event::CameraPanZoom(data) => {
                // val center = cameraOffset + Offset(cSize.width, cSize.height) * zoom / 2f
                let pan: Vec2 = [data[0], data[1]].into();

                model.camera.set_cam_pos(pan);
                model.camera.set_zoom(data[2]);
                caps.render.render();
            }
            Event::ChangePanZoom(data) => {
                let drag: Vec2 = [data[0], data[1]].into();
                let delta_pan = model.camera.drag_start - drag;
                let zoom_change = data[2];
                let new_pos = model.camera.pan() + delta_pan;
                info!(
                    "pzoom: drag:{:?}, drag_start:{:?}, pos:{:?}, delta:{:?}",
                    drag, model.camera.drag_start, model.camera.pan, delta_pan
                );
                model.camera.drag_cam(drag);
                let new_zoom = model.camera.zoom * zoom_change;
                model.camera.set_zoom(new_zoom);
                caps.render.render();
            }
            Event::AnchorDrag(screen_start) => model.camera.set_drag_start(screen_start.into()),
            Event::ChangeZoom(zchange) => {
                let new_zoom = model.camera.zoom * zchange;
                model.camera.set_zoom(new_zoom);
                caps.render.render();
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let (min_cell, max_cell) = model.camera.cell_bounds();
        let cell_coords = model
            .life
            .state
            .iter()
            .filter(|cell| cell[0] >= min_cell[0])
            .filter(|cell| cell[1] >= min_cell[1])
            .filter(|cell| cell[0] <= max_cell[0])
            .filter(|cell| cell[1] <= max_cell[1])
            .map(|cell| model.camera.cell2creen(cell).into())
            .collect();
        let grid_offset = model.camera.pan().into();
        let [modx, mody] = model.camera.grid_mod().into();
        ViewModel {
            cell_coords,
            cell_size: model.camera.cell_size(),
            camera_pan: grid_offset,
            modx,
            mody,
        }
    }
}
