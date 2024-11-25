use crux_core::{
    macros::{Effect, Export},
    render::Render,
    App,
};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Hello;

#[derive(Default)]
pub struct Model;

#[derive(Serialize, Deserialize)]
pub enum Event {
    None,
}

#[derive(Effect)]
pub struct Capabilites {
    render: Render<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
    greeting: String,
}
impl App for Hello {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        caps.render.render()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            greeting: "Hello, World!".to_string(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::testing::AppTester;

    #[test]
    fn hello_says_hello_world() {
        let hello = AppTester::<Hello, _>::default();
        let mut model = Model;

        // Call 'update' and request effects
        let update = hello.update(Event::None, &mut model);

        // Check update asked us to `Render`
        update.expect_one_effect().expect_render();

        // Make sure the view matches our expectations
        let actual_view = &hello.view(&model).greeting;
        let expected_view = "Hello, World!";
        assert_eq!(actual_view, expected_view);
    }
}
