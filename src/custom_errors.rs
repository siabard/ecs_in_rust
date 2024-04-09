use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomErrors {
    #[error("Attemping to add component to an enityty without calling create component first")]
    CreatComponentNeverCalled,
    #[error("attempting to reference a component that wasn't registered")]
    ComponentNotRegistered,
    #[error("attempting to reference a entity that doesn't exist")]
    EntityDoesNotExits,
    #[error("attempting to reference a component that doesn't exist")]
    ComponentDoesNotExists,
    #[error("attempting to downcast to a wrong type")]
    DowncastsToWrongType,
}
