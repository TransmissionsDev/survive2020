pub mod hornets;
pub mod main_menu;
pub mod wildfires;

use crate::resources::high_scores::update_high_score_if_greater;
use crate::states::main_menu::MainMenuState;

use amethyst::core::Time;
use amethyst::input::{is_key_down, VirtualKeyCode};

use crate::{delete_all_entities_with_component, get_main_font, load_sprite};
use amethyst::ui::{Anchor, LineMode, UiText, UiTransform};
use amethyst::{
    core::transform::Transform,
    core::ArcThreadPool,
    ecs::prelude::Join,
    ecs::{Component, DenseVecStorage, Dispatcher, DispatcherBuilder},
    prelude::*,
    renderer::Camera,
    window::ScreenDimensions,
};

/// A component to tag a timer text component.
pub struct TimerComponent;
impl Component for TimerComponent {
    type Storage = DenseVecStorage<Self>;
}

/// Update the elapsed time using delta seconds and set the high score if max time is passed and the score is the highest.
pub fn update_timer_and_set_high_score(
    world: &mut World,
    elapsed_time: &mut f32,
    max_time: f32,
    score: u64,
    high_score_key: &str,
) -> SimpleTrans {
    // Old time + delta seconds.
    let new_time = *elapsed_time + world.read_resource::<Time>().delta_seconds();

    // Whether or not a full second has changed.
    let time_changed_by_a_second = new_time.floor() > elapsed_time.floor();

    // If the timer is maxed out.
    let level_is_over = *elapsed_time >= max_time;

    // Update the elapsed time
    *elapsed_time = new_time;

    let timer_entity = {
        if time_changed_by_a_second || level_is_over {
            let mut ui_texts = world.write_storage::<UiText>();
            let timer_components = world.read_storage::<TimerComponent>();
            let entities = world.entities();

            let mut timer_entity = None;

            for (ui_text, _, entity) in (&mut ui_texts, &timer_components, &entities).join() {
                ui_text.text = format!("{}s / {}s", elapsed_time.floor(), max_time);

                if level_is_over {
                    timer_entity = Some(entity);
                }
            }

            timer_entity
        } else {
            None
        }
    };

    if level_is_over {
        update_high_score_if_greater(world, high_score_key, score);

        // Delete the timer entity.
        if let Some(entity) = timer_entity {
            world
                .delete_entity(entity)
                .expect("Couldn't delete timer text entity!");
        }

        Trans::Replace(Box::new(MainMenuState::default()))
    } else {
        Trans::None
    }
}

/// Create timer text with default value of "0s / {max_seconds}s"
/// Tagged with TimerComponent.
/// It will automatically get deleted when used with `update_timer_and_set_high_score` when the timer ends.
pub fn init_timer_text(world: &mut World, max_seconds: f32) {
    let font = get_main_font(world);

    let transform = UiTransform::new(
        "timer_text".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        0.0,
        -65.0,
        0.0,
        600.0,
        50.0,
    );
    let ui_text = UiText::new(
        font,
        format!("0s /{}s", max_seconds),
        [1.0, 1.0, 1.0, 1.0],
        25.0,
        LineMode::Single,
        Anchor::Middle,
    );

    world
        .create_entity()
        .with(TimerComponent)
        .with(transform)
        .with(ui_text)
        .build();
}

/// Creates the 2D camera.
pub fn init_camera(world: &mut World) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems.
pub fn create_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder),
) -> Dispatcher<'a, 'b> {
    let mut builder = DispatcherBuilder::new();

    add_systems(&mut builder);

    let mut dispatcher = builder
        .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
        .build();
    dispatcher.setup(world);

    dispatcher
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems. Returns a Some(DispatchBuilder).
pub fn create_optional_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder),
) -> Option<Dispatcher<'a, 'b>> {
    Some(create_systems_dispatcher(world, add_systems))
}

/// Take's a state's dispatcher and if it exists, runs all of its systems.
pub fn run_systems(world: &World, dispatcher: &mut Option<Dispatcher>) {
    if let Some(dispatcher) = dispatcher.as_mut() {
        dispatcher.dispatch(world);
    }
}

/// Return to main menu on escape.
pub fn return_to_main_menu_on_escape(event: StateEvent) -> SimpleTrans {
    if let StateEvent::Window(event) = &event {
        if is_key_down(event, VirtualKeyCode::Escape) {
            Trans::Replace(Box::new(MainMenuState::default()))
        } else {
            Trans::None
        }
    } else {
        Trans::None
    }
}

/// Tag a component as the title of a level.
/// Since all levels have this (including the main menu), you do not need to delete them in your `on_stop` func.
pub struct LevelTitle;
impl Component for LevelTitle {
    type Storage = DenseVecStorage<Self>;
}

/// Displays the level title at the top of the screen.
pub fn init_level_title(world: &mut World, filename: &str) {
    // Delete previous titles
    delete_all_entities_with_component::<LevelTitle>(world);

    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let sprite = load_sprite(world, filename, 0);

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.93, 0.);

    world
        .create_entity()
        .with(LevelTitle)
        .with(transform)
        .with(sprite)
        .build();
}
