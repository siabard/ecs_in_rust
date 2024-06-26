pub mod query;
pub mod query_entity;

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use eyre::Result;

use crate::custom_errors::CustomErrors;

pub type Component = Rc<RefCell<dyn Any>>;
pub type Components = HashMap<TypeId, Vec<Option<Component>>>;

#[derive(Debug, Default)]
pub struct Entities {
    components: Components,
    bit_masks: HashMap<TypeId, u32>,
    map: Vec<u32>,
    inserting_into_index: usize,
}

impl Entities {
    pub fn register_component<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, vec![]);
        self.bit_masks.insert(type_id, 1 << self.bit_masks.len());
    }

    pub fn create_entity(&mut self) -> &mut Self {
        if let Some((index, _)) = self
            .map
            .iter()
            .enumerate()
            .find(|(_index, mask)| **mask == 0)
        {
            self.inserting_into_index = index;
        } else {
            self.components
                .iter_mut()
                .for_each(|(_key, component)| component.push(None));

            self.map.push(0);
            self.inserting_into_index = self.map.len() - 1;
        }

        self
    }

    pub fn with_component(&mut self, data: impl Any) -> Result<&mut Self> {
        let type_id = data.type_id();
        let index = self.inserting_into_index;

        if let Some(components) = self.components.get_mut(&type_id) {
            let component = components
                .get_mut(index)
                .ok_or(CustomErrors::CreatComponentNeverCalled)?;
            *component = Some(Rc::new(RefCell::new(data)) as Component);

            let bitmask = self.bit_masks.get(&type_id).unwrap();
            self.map[index] |= *bitmask;
        } else {
            return Err(CustomErrors::ComponentNotRegistered.into());
        }
        Ok(self)
    }

    pub fn get_bitmask(&self, type_id: &TypeId) -> Option<u32> {
        if let Some(bit_mask) = self.bit_masks.get(type_id) {
            Some(*bit_mask)
        } else {
            None
        }
    }

    pub fn delete_component_by_entity_id<T: Any>(&mut self, index: usize) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mask = if let Some(mask) = self.bit_masks.get(&type_id) {
            mask
        } else {
            return Err(CustomErrors::ComponentNotRegistered.into());
        };

        if self.has_component(index, *mask) {
            self.map[index] ^= *mask;
        }

        Ok(())
    }

    pub fn add_component_by_entity_id(&mut self, data: impl Any, index: usize) -> Result<()> {
        let type_id = data.type_id();
        let mask = if let Some(mask) = self.bit_masks.get(&type_id) {
            mask
        } else {
            return Err(CustomErrors::ComponentNotRegistered.into());
        };

        self.map[index] |= *mask;
        let component = self.components.get_mut(&type_id).unwrap();
        component[index] = Some(Rc::new(RefCell::new(data)) as Rc<RefCell<dyn Any>>);
        Ok(())
    }

    pub fn delete_entity_by_id(&mut self, index: usize) -> Result<()> {
        if let Some(map) = self.map.get_mut(index) {
            *map = 0u32;
        } else {
            return Err(CustomErrors::EntityDoesNotExits.into());
        }
        Ok(())
    }

    fn has_component(&self, index: usize, mask: u32) -> bool {
        self.map[index] & mask == mask
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
    fn bitmask_updated_when_registering_entity() {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        let type_id = TypeId::of::<Health>();
        let health_mask = entities.bit_masks.get(&type_id).unwrap();
        assert_eq!(*health_mask, 1);

        entities.register_component::<Speed>();
        let type_id = TypeId::of::<Speed>();
        let speed_mask = entities.bit_masks.get(&type_id).unwrap();
        assert_eq!(*speed_mask, 2);
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
            .with_component(Health(100))
            .unwrap()
            .with_component(Speed(15))
            .unwrap();

        let first_health = &entities.components.get(&TypeId::of::<Health>()).unwrap()[0];
        let wrapped_health = first_health.as_ref().unwrap();
        let borrowed_health = wrapped_health.borrow();
        let health = borrowed_health.downcast_ref::<Health>().unwrap();

        assert_eq!(health.0, 100);
        Ok(())
    }

    #[test]
    fn map_is_updated_when_creating_entities() -> Result<()> {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15))?;

        let entity_map = entities.map[0];
        assert_eq!(entity_map, 3);

        entities.create_entity().with_component(Speed(15))?;

        let entity_map = entities.map[1];
        assert_eq!(entity_map, 2);

        Ok(())
    }

    #[test]
    fn delete_component_by_entity_id() -> Result<()> {
        let mut entities = Entities::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15))?;

        entities.delete_component_by_entity_id::<Health>(0)?;

        let entity_map = entities.map[0];
        assert_eq!(entity_map, 2);

        Ok(())
    }

    #[test]
    fn add_component_to_entity_by_id() -> Result<()> {
        let mut entities = Entities::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities.create_entity().with_component(Health(100))?;
        let entity_map = entities.map[0];
        assert_eq!(entity_map, 1);

        entities.add_component_by_entity_id(Speed(50), 0)?;

        let entity_map = entities.map[0];
        assert_eq!(entity_map, 3);

        let speed_type_id = TypeId::of::<Speed>();

        let wrapped_speeds = entities.components.get(&speed_type_id).unwrap();
        let wrapped_speed = wrapped_speeds[0].as_ref().unwrap();
        let borrwed_speed = wrapped_speed.borrow();
        let speed = borrwed_speed.downcast_ref::<Speed>().unwrap();
        assert_eq!(speed.0, 50);

        Ok(())
    }

    #[test]
    fn delete_entity_by_id() -> Result<()> {
        let mut entities = Entities::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();

        entities.create_entity().with_component(Health(100))?;
        entities.delete_entity_by_id(0)?;

        assert_eq!(entities.map[0], 0);
        Ok(())
    }

    #[test]
    fn create_entities_are_inserted_into_deleted_entities_column() -> Result<()> {
        let mut entities = Entities::default();

        entities.register_component::<Health>();

        entities.create_entity().with_component(Health(100))?;
        entities.create_entity().with_component(Health(50))?;

        entities.delete_entity_by_id(0)?;

        entities.create_entity().with_component(Health(25))?;

        assert_eq!(entities.map[0], 1);

        let type_id = TypeId::of::<Health>();
        let borrowed_health = &entities.components.get(&type_id).unwrap()[0]
            .as_ref()
            .unwrap()
            .borrow();
        let health = borrowed_health.downcast_ref::<Health>().unwrap();

        assert_eq!(health.0, 25);
        Ok(())
    }

    #[test]
    fn should_not_add_component_back_after_deleting_twice() -> Result<()> {
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();

        entities
            .create_entity()
            .with_component(100_u32)?
            .with_component(50.0_f32)?;

        entities.delete_component_by_entity_id::<u32>(0)?;
        entities.delete_component_by_entity_id::<u32>(0)?;
        assert_eq!(entities.map[0], 2);

        Ok(())
    }

    struct Health(pub u32);
    struct Speed(pub u32);
}
