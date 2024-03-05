use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomErrors {
    #[error("Attemping to add component to an enityty without calling create component first")]
    CreatComponentNeverCalled,
    #[error("attempted to insert data for component that wasn't registered")]
    ComponentNotRegistered,
}