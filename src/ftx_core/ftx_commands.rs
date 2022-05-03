use anyhow::{bail, Error as AnyHowError, Result as AnyHowResult};
use async_trait::async_trait;
use tui::{
    backend::Backend,
    Terminal,
};
use chrono::Utc;
use crate::{
    UserSpace,
    utils::{yn},
    db::*,
    orders::*,
    command_handling::CommandHandling
};

pub struct FtxStruct {}

#[async_trait]
impl<B: Backend + std::marker::Send> CommandHandling<B> for FtxStruct {
    //...
}