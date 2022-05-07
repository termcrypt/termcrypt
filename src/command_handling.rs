use anyhow::{bail, Error, Result};
use async_trait::async_trait;
use tui::{backend::Backend, Terminal};

use crate::{UserSpace, ForceOption, misc_commands};

pub fn api_check<T>(api: Option<T>) -> Result<T, Error> {
    if let Some(api) = api{
        Ok(api)
    } else {
        bail!("api keys needed")
    }
}

#[async_trait]
pub trait CommandHandling<B: Backend + std::marker::Send>: Sync {
    async fn price(&self, _us: &mut UserSpace) -> Result<(), Error> {Ok(())}
    async fn balance(&self, _us: &mut UserSpace) -> Result<(), Error> {Ok(())}
    async fn order(&self, _us: &mut UserSpace, _terminal: &mut Terminal<B>) -> Result<(), Error> {Ok(())}
    async fn config_defaults(&self, _us: &mut UserSpace, _terminal: &mut Terminal<B>) -> Result<(), Error> {Ok(())}
    async fn setup_api_keys(&self, _us: &mut UserSpace, _terminal: &mut Terminal<B>) -> Result<(), Error> {Ok(())}
    // Commands with args
    async fn search(&self, _us: &mut UserSpace, _terminal: &mut Terminal<B>, _command: &str) -> Result<(), Error> {Ok(())}
    async fn change_pair(&self, _us: &mut UserSpace, _terminal: &mut Terminal<B>, _command: &str) -> Result<(), Error> {Ok(())}
}

pub struct Command<'a, B: Backend + std::marker::Send> {
    pub command: String,
    pub exchange: Box<dyn CommandHandling<B>>,
    pub us: &'a mut UserSpace,
    pub terminal: &'a mut Terminal<B>
}

impl<B: Backend + std::marker::Send> Command<'_, B> {
    pub async fn find(&mut self) -> Result<bool, Error> {

        let commands = self.command.as_str();
        let mut all_real_command = true;

        let split_commands = commands.split(" && ");
        let multiple_commands = split_commands.clone().count() > 1;
        if multiple_commands {self.us.bl()}

        for (_num, command) in split_commands.enumerate() {
            if multiple_commands {
                self.us.prnt(format!("[Running: {}]", command.split_whitespace().next().force()?.to_uppercase()));
                self.terminal.draw(|f| UserSpace::ui(f, self.us))?;
            }

            match command {

                // ..........................
                // Exchange specific commands
                // ..........................

                // Get prices for current pair
                "price" | "prices" | "p" => {self.exchange.price(self.us).await?}
                // Get balance of current account/sub-account
                "balance" | "balances" | "bal" => {self.exchange.balance(self.us).await?}
                // Open a trade/order
                "order" | "o" | "m" | "l" | "ob" => {self.exchange.order(self.us, self.terminal).await?}
                // Change defaults (exchange specific)
                "defaults" | "default" | "def" => {self.exchange.config_defaults(self.us, self.terminal).await?}
                // Change API keys
                "keys" => {self.exchange.setup_api_keys(self.us, self.terminal).await?}

                // Commands with args

                // Search exchange pairs with query
                command if command.starts_with("search") => {self.exchange.search(self.us, self.terminal, command).await?}
                // Change current pair
                command if command.starts_with("pair") => {self.exchange.change_pair(self.us, self.terminal, command).await?}

                // ...........................
                // Exchange unrelated commands
                // ...........................

                // Lists all available commands
                "help" => misc_commands::help(self.us).await?,
                // Gives info about the project
                "about" | "info" => misc_commands::about(self.us).await?,
                // Clear terminal output
                "clear" | "clr" => self.us.clear_commands(),
                // Change configuration settings
                "config" | "conf" => misc_commands::config(self.us, self.terminal).await?,
                // Get trade session opens and closes for key locations
                "sessions" | "ses" => misc_commands::sessions(self.us).await?,
                // Output current date
                "date" => misc_commands::date(self.us).await?,
                // Output current time
                "time" => misc_commands::time(self.us).await?,
                // Outputs ascii logo and info
                "afetch" | "aclear" => misc_commands::afetch(self.us, command).await?,
                // Ping --> Pong (checks commands work)
                "ping" => misc_commands::ping_pong(self.us).await?,
                // Get amount of commands used this session
                "count" => misc_commands::command_count(self.us).await?,
                
                // Commmands with args

                // Switches active exchange to user choice
                command if command.starts_with("switch ") => misc_commands::switch_exchange(self.us, command).await?,
                // MOOOOOOOOOOOOOO
                command if command.starts_with("cowsay ") => misc_commands::cowsay(self.us, command).await?,

                // Commands for testing/dev

                "trade_fetch" => misc_commands::trade_fetch(self.us).await?,
                "trade_wipe" => misc_commands::trade_wipe(self.us).await?,

                // If command does not exist return false
                _ => {all_real_command = false}
            }

            self.us.bl()
        }

        Ok(all_real_command)
    }
}