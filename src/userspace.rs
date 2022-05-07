use tui::{
	style::{Color, Style},
    backend::{Backend},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
    layout::{Alignment, Layout, Constraint, Direction, Corner},
	text::{Span, Spans, Text},
    Terminal, Frame,
};
use anyhow::{bail, Error, Result};
use std::{
	time::{Duration, Instant},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use unicode_width::UnicodeWidthStr;
use textwrap::wrap;

use crate::{
	//ActiveExchanges,
	UserSpace,
	command_handling::{self, CommandHandling},
	*,
	utils::{
		self,
		sub_strings,
		terminal_width,
	}
};

// Event notification type for the events widget
#[derive(Debug, Clone)]
pub enum EventLogType {
	// Entry filled
	EntryFill,
	// Take-profit filled
	TpFill,
	// Stoploss filled
	SlFill,
	// Significant warning
	Warning,
	// Empty message
	Empty,
}

// UI display arrangements
#[derive(Debug, Clone)]
pub enum UIMode {
	// Default UI (both events and commands)
	Split,
	// Orderinfo (both order information and commands)
	OrderInfo,
	// Events section hidden
	CommandsOnly,
	// Whole UI is for events
	EventsOnly
}

impl<'a> crate::UserSpace {
    // Run the app and UI
	pub async fn run_app<B: Backend + std::marker::Send>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error> {
		let mut last_tick = Instant::now();

		// Ascii art output
        utils::output_ascii(self);
		self.bl();

		// Update data to exchange defaults
		self.use_db_defaults()?;

		loop {
			self.input_prefix = format!("[{}]({})>", self.sub_account, self.pair);

			terminal.draw(|f| Self::ui(f, self))?;
			
			let timeout = self.tick_rate
				.checked_sub(last_tick.elapsed())
				.unwrap_or_else(|| Duration::from_secs(0));
			
			// Only stops updating UI when it needs to
       		if crossterm::event::poll(timeout)? {
                let evvent = event::read()?;
				if let Event::Key(key) = evvent {
					if key.modifiers == KeyModifiers::CONTROL {
						match key.code {
							KeyCode::Char('c') => {
								// Quits parent loop
								return Ok(());
							}
							// CTRL+Backspace clears input
							KeyCode::Char('h') => {
								self.input = "".to_string();
							}
							_ => {}
						}
					} else {
						match key.code {
							KeyCode::Enter => {
								// Scroll back to bottom
								while !self.command_history_scroll_overflow.is_empty() {
                                    let e = self.command_history_scroll_overflow[0].to_owned();
                                    self.command_history.insert(0, e);
									self.command_history_scroll_overflow.remove(0);
								}
								terminal.draw(|f| Self::ui(f, self))?;

								self.command_count += 1;
                                let sustained_input = self.input.to_owned();

                                // Remove user input as the function is processing
                                self.input_old = self.input.to_owned();

                                //self.input = "".to_string();
								//self.input = format!("{}", self.input.to_owned());
								//terminal.draw(|f| Self::ui(f, self))?;

                                let mut formatted_input = format!("{} {}", self.input_prefix, sustained_input);
                                // Make sure not to add command to terminal if clear command was run

                                if sustained_input != "clr" && sustained_input != "clear" && sustained_input != "aclear" {
								    self.command_history.insert(0, formatted_input.drain(..).collect());
                                }

								// Makes terminal look like it is loading
                                let unwiped_prefix = self.input_prefix.to_owned();
								self.input_prefix = "...".to_string();
								self.input = "".to_string();
								terminal.draw(|f| Self::ui(f, self))?;
								
                                // Handle the input for miscellaneous commands and exchanges
								self.handle_input(terminal).await?;

                                self.input_prefix = unwiped_prefix;
							}
							KeyCode::Char(c) => {
								if self.input.width() == 0 && c == ' ' {
									continue;
								}
								self.input.push(c);
							}
							KeyCode::Backspace => {
								self.input.pop();
							}
							_ => {}
						}
					}
				} else if let Event::Mouse(mouse) = evvent {
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            for _x in 1..3 {
                                if !self.command_history_scroll_overflow.is_empty() {
                                    let e = self.command_history_scroll_overflow[0].to_owned();
                                    self.command_history.insert(0, e);
                                    self.command_history_scroll_overflow.remove(0);
                                }
                            }
                            
                        },
                        MouseEventKind::ScrollUp => {
                            if self.command_history.len() >= 10 {
                                for _x in 1..3 {
                                    let e = self.command_history[0].to_owned();
                                    self.command_history_scroll_overflow.insert(0, e);
                                    self.command_history.remove(0);
                                }
                            }
                        },
                        //MouseEventKind::Down(_) => (),
                        //MouseEventKind::Drag(_) => println!("Dragging on : {},{} ", mouse.column, mouse.row ),
                        //MouseEventKind::Moved => println!("Moving on : {},{} ", mouse.column, mouse.row ),
                        //MouseEventKind::Up(_) => println!("Click on : {:?},{} ", mouse.column, mouse.row),
                        _=> ()
                    }
                }
			}
			if last_tick.elapsed() >= self.tick_rate {
				last_tick = Instant::now();
                // Lil thing to display tick rate
				//self.command_count += 1;
			}
		}
	}
	
    // User interface for tui to render to the terminal
	pub fn ui<B: Backend>(f: &mut Frame<B>, app: &UserSpace) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
			.split(f.size());

        let warning_style = Style::default().fg(Color::Yellow);

        // Creating the list of events for the events section
        let events_items: Vec<ListItem> = app
            .event_history
            .iter()
            .map(|(event_msg, event_type)| {
                let style = match event_type {
                    EventLogType::EntryFill => Style::default().fg(Color::Blue), 
                    EventLogType::SlFill => Style::default().fg(Color::Gray),
                    EventLogType::TpFill => Style::default().fg(Color::Green),
                    EventLogType::Warning => warning_style,
                    _ => warning_style,
                };
                let type_wording = match event_type {
                    EventLogType::EntryFill => "ENTRY",
                    EventLogType::TpFill => "TAKEPRFT",
                    EventLogType::SlFill => "STOPLOSS",
                    EventLogType::Warning => "WARNING",
                    EventLogType::Empty => "",
                    //_ => "INFO"
                };
                let content = vec![Spans::from(vec![
                    Span::styled(format!("{:<10}", type_wording), style),
                    Span::raw(event_msg),
                ])];
                ListItem::new(content)
            })
            /*.map(|(_i, m)| {
                let content = vec![Spans::from(Span::raw(m))];
                ListItem::new(content)
            })*/
            .collect();

        let now = chrono::Local::now();

        // Very unneded dot second counter
        let seconds = now.format("%S").to_string().parse::<i32>().unwrap();
        let dots = 
            if seconds % 2 == 0 {
                ":".to_string()
            } else {
                ".".to_string()
            };

        let current_time = format!("{}{dots}", now.format("%b %-d %-I:%M%p"));
		
        // The list section containing background events
		let events_widget =
			List::new(events_items)
				.start_corner(Corner::BottomLeft)
				.block(Block::default()
					.borders(Borders::ALL)
					//.border_type(BorderType::Rounded)
					.title(current_time));
		f.render_widget(events_widget, chunks[0]);

        // The block for the section containing the input bar and output messages
		let commands_section = Block::default()
			.borders(Borders::ALL)
			.title(app.active_exchange.to_string())
			.title_alignment(Alignment::Left);
		f.render_widget(commands_section, chunks[1]);

        // The layout for the section containing the input bar and output messages
		let terminal_interactive_section = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(99), Constraint::Max(1)])
			.split(chunks[1]);

        // Place cursor after the input prefix
		f.set_cursor(
			// Put cursor past the end of the input text
			terminal_interactive_section[1].x + app.input_prefix.width() as u16 + app.input.width() as u16 + 1,
			// Move one line down, from the border to the input line
			terminal_interactive_section[1].y,
		);

        
        // Wrap text before displaying so it is not cut off
        let mut wrapped_command_history: Vec<String> = Vec::new();
        let commands_widget_width = terminal_width()-3;

        for line in app.command_history.iter() {
            for sub_string in wrap(line.as_str(), commands_widget_width as usize).iter().rev() {
                wrapped_command_history.push(sub_string.to_string());
            }
        }

        // Creating the list of events for the past commands section
		let past_commands_widget: Vec<ListItem> = wrapped_command_history
			.iter()
			.enumerate()
			.map(|(_i, m)| {
				let content = vec![Spans::from(Span::raw(m))];
				ListItem::new(content)
			})
			.collect();

        // The main section containing program output (past commands)
		let past_commands_widget =
			List::new(past_commands_widget)
				.start_corner(Corner::BottomLeft)
				.block(Block::default()
					.borders(Borders::NONE)
				);
		f.render_widget(past_commands_widget, terminal_interactive_section[0]);

        // User input section to query commands
		let input = Paragraph::new(format!("{} {}", app.input_prefix, app.input))
			.block(Block::default()
				.borders(Borders::NONE)
				//.border_type(BorderType::Rounded)
				//.title("")
			);
		f.render_widget(input, terminal_interactive_section[1]);
	}

    // In-built function to add to the command_history to effectively "print"/output to the terminal
	pub fn prnt(&mut self, text: String) {
		self.command_history.insert(0, text);
        self.stream_differ += 1;     
    }

	// Print a blank line
	pub fn bl(&mut self) {
		self.prnt(String::new());
	}

	// Print multiple blank lines
	pub fn _mbl(&mut self, count: u16) {
		let mut i = 0;
		while i < count {
			self.prnt(String::new());
			i += 1;
		}
		self.stream_differ += i;
	}

    // Clear past commands from UI
    pub fn clear_commands(&mut self) {
        self.command_history.clear();
        self.command_history_scroll_overflow.clear();
    }

    // In-built function to ask user for input for post-command input
    pub async fn ask_input<B: Backend>(&mut self, prefix: &str, terminal: &mut Terminal<B>, _history_save_name: Option<&str>) -> Result<String, Error> {
        self.input = "".to_string();
		let old_prefix = self.input_prefix.to_owned();
        let mut last_tick = Instant::now();

        self.input_prefix = format!(" [{}]>", prefix);

        loop {
            terminal.draw(|f| Self::ui(f, self))?;
			
			let timeout = self.tick_rate
				.checked_sub(last_tick.elapsed())
				.unwrap_or_else(|| Duration::from_secs(0));
			// Only stops updating UI when it needs to
       		if crossterm::event::poll(timeout)? {
				if let Event::Key(key) = event::read()? {
					if key.modifiers == KeyModifiers::CONTROL {
						match key.code {
							KeyCode::Char('c') => {
								// Quits parent loop
								bail!("User quit input");
							}
							// CTRL+Backspace clears input
							KeyCode::Char('h') => {
								self.input = "".to_string();
							}
							_ => {}
						}
					} else {
						match key.code {
							KeyCode::Enter => {
                                // Check if scroll buffer exists, add all back to commands list
                                //...

                                self.stream_differ = 0;
								self.command_count += 1;

                                let sustained_input = self.input.to_owned();

                                // Remove user input as the function is processing
                                self.input_old = self.input.to_owned();
                                self.input = "".to_string();

                                let mut formatted_input = format!("{} {}", self.input_prefix, sustained_input);

                                // Blank line gap
                                //self.command_history.insert(self.stream_differ as usize, "".to_string());

								self.command_history.insert(self.stream_differ as usize, formatted_input.drain(..).collect());
                                self.input_prefix = old_prefix;

                                terminal.draw(|f| Self::ui(f, self))?;
                                return Ok(self.input_old.to_owned())
							}
							KeyCode::Char(c) => {
								if self.input.width() == 0 && c == ' ' {
									continue;
								}
								self.input.push(c);
							}
							KeyCode::Backspace => {
								self.input.pop();
							}
							_ => {}
						}
					}
				}
            }
            if last_tick.elapsed() >= self.tick_rate {
				last_tick = Instant::now();
			}
        }
    }

    // Handle user input through the input widget
	async fn handle_input<B: Backend + std::marker::Send>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error> {
		// Start exchange instance

		let chosen_exchange: Box<dyn CommandHandling<B>> = match self.active_exchange {
			Exchange::Bybit => {
				let bybit_struct = BybitStruct {
					bybit_api: self.db_info.bybit_api.clone()
				};
				Box::new(bybit_struct) as Box<dyn CommandHandling<B>>
			},
			Exchange::Ftx => {
				Box::new(FtxStruct {}) as Box<dyn CommandHandling<B>>
			}
		};

		//let chosen_exchange = exchange_struct as &dyn command_handling::CommandHandling<B>;

		//let chosen_exchange: Box<dyn command_handling::CommandHandling> = Box::new(obj);
		let mut command = command_handling::Command {
			command: self.input_old.to_string(),
			exchange: chosen_exchange,
			us: self,
			terminal
		};

    	let _real_command = match command.find().await {
			Ok(x) => x,
			Err(_e) => {
				termbug::error(format!("Function Exited: {_e:?}"), self);
				true
			}
		};

		// Use real_command when history is added

		Ok(())
	}

	pub async fn switch_exchange(&mut self, new_exchange: Exchange) -> Result<(), Error>{
		self.db_info = get_db_info().await?;
		self.active_exchange = new_exchange;
		self.use_db_defaults()?;
		Ok(())
	}

	pub fn use_db_defaults(&mut self) -> Result<(), Error> {
		let sub_account: String;
		let pair: String;

		match self.active_exchange {
			Exchange::Bybit => {
				sub_account = self.db_info.bybit_default_sub.to_owned();
				pair = self.db_info.bybit_default_pair.to_owned();
			}
			Exchange::Ftx => {
				sub_account = self.db_info.ftx_default_sub.to_owned();
				pair = self.db_info.ftx_default_pair.to_owned();
			}
		}

		self.sub_account = sub_account;
		self.pair = pair;

		Ok(())
	}
}