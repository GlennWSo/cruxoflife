use shared::{App, Effect};

use std::sync::Arc;
pub type Core = Arc<shared::Core<Effect, App>>;

pub fn new() -> Core {
    Core::default()
}
