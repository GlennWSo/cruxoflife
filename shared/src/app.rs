use crux_core::{macros::Effect, render::Render, App};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Counter;

#[derive(Default)]
pub struct Model {
    count: i32,
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Increment,
    Decrement,
    Reset,
}

#[derive(Effect)]
pub struct Capabilites {
    render: Render<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
    count: String,
}
impl App for Counter {
    type Model = Model;
    type Capabilities = Capabilites;
    type Event = Event;
    type ViewModel = ViewModel;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::Increment => model.count += 1,
            Event::Decrement => model.count -= 1,
            Event::Reset => model.count = 0,
        }
        caps.render.render()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            count: format!("Count is: {}", model.count),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::{assert_effect, testing::AppTester};

    #[test]
    fn renders() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();

        let update = app.update(Event::Reset, &mut model);

        // Check update asked us to `Render`
        assert_effect!(update, Effect::Render(_));
    }

    #[test]
    fn shows_initial_count() {
        let app = AppTester::<Counter, _>::default();
        let model = Model::default();

        let actual_view = app.view(&model).count;
        let expected_view = "Count is: 0";
        assert_eq!(actual_view, expected_view);
    }

    #[test]
    fn resets_count() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();
        let _ = app.update(Event::Increment, &mut model);
        let expected_view = "Count is: 0";
        {
            let view0 = app.view(&model).count;
            assert_ne!(view0, expected_view);
        }
        let _ = app.update(Event::Reset, &mut model);
        let reset_view = app.view(&model).count;
        assert_eq!(reset_view, expected_view);
    }

    #[test]
    fn counts_up_and_down() {
        let app = AppTester::<Counter, _>::default();
        let mut model = Model::default();

        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Reset, &mut model);
        let _ = app.update(Event::Decrement, &mut model);
        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Increment, &mut model);

        let actual_view = app.view(&model).count;
        let expected_view = "Count is: 1";
        assert_eq!(actual_view, expected_view);
    }
}
