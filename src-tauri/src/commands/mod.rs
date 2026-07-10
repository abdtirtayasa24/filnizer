use serde::Serialize;

use crate::errors::AppError;

pub mod app;
pub mod converter;
pub mod organizer;

pub type CommandResult<T> = Result<CommandResponse<T>, AppError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse<T>
where
    T: Serialize,
{
    pub data: T,
}

impl<T> CommandResponse<T>
where
    T: Serialize,
{
    pub fn new(data: T) -> Self {
        Self { data }
    }
}
