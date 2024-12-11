use std::rc::Rc;

use shared::{App, Effect};

pub type Core = Rc<shared::Core<Effect, App>>;

pub fn new() -> Core {
    Core::default()
}
