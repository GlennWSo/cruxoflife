use crux_core::capability::{CapabilityContext, Operation};
use crux_core::macros::Capability;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;

use super::{CellVector, Model};

type Data = Vec<u8>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Save(Data),
}

impl Operation for FileOperation {
    type Output = Option<CellVector>;
}

#[derive(Capability)]
pub struct FileIO<Event> {
    context: CapabilityContext<FileOperation, Event>,
}

impl<Event> FileIO<Event> {
    pub fn new(context: CapabilityContext<FileOperation, Event>) -> Self {
        Self { context }
    }
    pub fn save(&self, model: &Model)
    where
        Event: 'static,
    {
        let ctx = self.context.clone();
        let data = model.life.state_as_list();
        let data = to_vec(&data).unwrap();
        self.context.spawn(async move {
            // Instruct Shell to save some bytes of data
            ctx.request_from_shell(FileOperation::Save(data)).await;
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AlertOpereation {
    Info(String),
    Warning(String),
    Error(String),
}

impl Operation for AlertOpereation {
    type Output = ();
}

#[derive(Capability)]
pub struct Alert<Event> {
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
