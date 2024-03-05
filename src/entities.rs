use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use eyre::{bail, Result};

#[derive(Debug, Default)]
pub struct Entities {
    components: HashMap<TypeId, Vec<Option<Rc<RefCell<dyn Any>>>>>,
}

impl Entities {
    pub fn register_component<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, vec![]);
    }

    pub fn create_entity(&mut self) -> &mut Self {
        self.components
            .iter_mut()
            .for_each(|(_key, component)| component.push(None));
        self
    }

    pub fn with_component(&mut self, data: impl Any) -> Result<&mut Self> {
        let type_id = data.type_id();
        if let Some(components) = self.components.get_mut(&type_id) {
            let last_component = components
                .last_mut()
                .ok_or_else(|| "Could not find last component")
                .unwrap();
            *last_component = Some(Rc::new(RefCell::new(data)));
        } else {
            bail!("attempted to insert data for component that wasn't registered")
        }
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use super::*;

    #[test]
    fn register_on_entity() {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        let type_id = TypeId::of::<Health>();
        let health_components = entities.components.get(&type_id).unwrap();
        assert!(health_components.is_empty());
    }

    #[test]
    fn create_entity() {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities.create_entity();
        let health = entities.components.get(&TypeId::of::<Health>()).unwrap();
        let speed = entities.components.get(&TypeId::of::<Speed>()).unwrap();

        assert!(health.len() == speed.len() && health.len() == 1);
        assert!(health[0].is_none() && speed[0].is_none());
    }

    #[test]
    fn with_component() -> Result<()> {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities
            .create_entity()
            .with_component(Health(100)).unwrap()
            .with_component(Speed(15)).unwrap();

        let first_health = &entities.components.get(&TypeId::of::<Health>()).unwrap()[0];
        let wrapped_health = first_health.as_ref().unwrap();
        let borrowed_health = wrapped_health.borrow();
        let health = borrowed_health.downcast_ref::<Health>().unwrap();

        assert_eq!(health.0, 100);
        Ok(())
    }

    struct Health(pub u32);
    struct Speed(pub u32);
}
