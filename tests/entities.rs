use ecs_in_rust::*;
use eyre::Result;

#[test]
fn create_entity() -> Result<()>{
    // register component to register
    let location = Location(42.0, 24.0);

    let mut world = World::new();
    world.register_component::<Location>();
    world.register_component::<Size>();

    world
        .create_entity()
        .with_component(Location(42.0, 24.0))?
        .with_component(Size(10.0))?;

    Ok(())
}

struct Location(pub f32, pub f32);
struct Size(pub f32);
