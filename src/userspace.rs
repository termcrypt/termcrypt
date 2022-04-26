use tui::{
	style::{Color, Style},
    backend::{Backend},
    widgets::{Widget, Block, Borders, Paragraph, List, ListItem, Wrap},
    layout::{Alignment, Layout, Constraint, Direction, Corner},
	text::{Span, Spans, Text},
    Terminal, Frame
};
use anyhow::{bail, Error as AnyHowError, Result as AnyHowResult};
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

use crate::*;

impl crate::UserSpace {
    // Run the app and UI
	pub async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> AnyHowResult<(), AnyHowError> {
		let mut last_tick = Instant::now();

        // Outputs version and ascii art
        /*
        if self.desktop_terminal {
            utils::wideversion(self);
        } else {
            utils::slimversion(self);
        };
        */
        bl(self);
        utils::output_ascii(self);

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
								self.command_count += 1;

                                let sustained_input = self.input.to_owned();

                                // Remove user input as the function is processing
                                self.input_old = self.input.to_owned();
                                self.input = "".to_string();

                                let mut formatted_input = format!("{} {}", self.input_prefix, sustained_input);
                                // Make sure not to add command to terminal if clear command was run
                                if sustained_input != "clr" && sustained_input != "clear" && sustained_input != "aclear" {
								    self.command_history.insert(0, formatted_input.drain(..).collect());
                                }

                                let unwiped_prefix = self.input_prefix.to_owned();
                                self.input_prefix = String::new();
                                terminal.draw(|f| Self::ui(f, self))?;

                                // Handle the input for miscellaneous commands and exchanges
								self.handle_input(terminal).await?;

                                self.input_prefix = unwiped_prefix;
                                self.command_history.insert(0, " ".to_string());
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
				} else if let Event::Mouse(mouse) = event::read()? {
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            for _x in 1..4 {
                                if !self.command_history_scroll_overflow.is_empty() {
                                    let e = self.command_history_scroll_overflow[0].to_owned();
                                    self.command_history.insert(0, e);
                                    self.command_history_scroll_overflow.remove(0);
                                }
                            }
                            
                        },
                        MouseEventKind::ScrollUp => {
                            if self.command_history.len() >= 10 {
                                for _x in 1..4 {
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
	fn ui<B: Backend>(f: &mut Frame<B>, app: &UserSpace) {
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
                    EventType::EntryFill => Style::default().fg(Color::Blue), 
                    EventType::SlFill => Style::default().fg(Color::Gray),
                    EventType::TpFill => Style::default().fg(Color::Green),
                    EventType::Warning => warning_style,
                    _ => warning_style,
                };
                let type_wording = match event_type {
                    EventType::EntryFill => "ENTRY",
                    EventType::TpFill => "TAKEPRFT",
                    EventType::SlFill => "STOPLOSS",
                    EventType::Warning => "WARNING",
                    EventType::Empty => "",
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
			.title("Commands")
			.title_alignment(Alignment::Left);
		f.render_widget(commands_section, chunks[1]);

        // The layout for the section containing the input bar and output messages
		let terminal_interactive_section = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(95), Constraint::Max(1)])
			.split(chunks[1]);

        // Place cursor after the input prefix
		f.set_cursor(
			// Put cursor past the end of the input text
			terminal_interactive_section[1].x + app.input_prefix.width() as u16 + app.input.width() as u16 + 1,
			// Move one line down, from the border to the input line
			terminal_interactive_section[1].y,
		);

        // Creating the list of events for the past commands section
		let past_commands_widget: Vec<ListItem> = app
			.command_history
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

    // Clear past commands from UI
    pub fn clear_commands(&mut self) {
        self.command_history.clear();
        self.command_history_scroll_overflow.clear();
    }

    // In-built function to ask user for input for post-command input
    pub async fn ask_input<B: Backend>(&mut self, prefix: &str, terminal: &mut Terminal<B>, _history_save_name: Option<&str>) -> AnyHowResult<String, AnyHowError> {
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
	async fn handle_input<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> AnyHowResult<(), AnyHowError> {
        // Match against miscellaneous commands
		match misc::handle_commands(
            self
		).await {
			Ok(x) => {
				if x {
					//is_real_command = true
				}
			}
			Err(_e) => {
				termbug::error(format!("Function Exited: {_e:?}"), self);
				//continue;
			}
		}

        // Match against Bybit commands
        match bybit_inter::handle_commands(
            self,
            terminal
		).await {
			Ok(x) => {
				if x {
					//is_real_command = true
				}
			}
			Err(_e) => {
				termbug::error(format!("Function Exited: {_e:?}"), self);
				//continue;
			}
		}

		Ok(())
	}



    /*
	async fn loopy(mut self) -> AnyHowResult<(), AnyHowError> {
		// Check user input history location exists
		let line_main_location = format!("{}main.txt", history_location().as_str());
		if !std::path::Path::new(&line_main_location).exists() {
			std::fs::File::create(&line_main_location)?;
		}

		// User input space with history
		let mut line_main = Editor::<()>::new();
		line_main.load_history(&line_main_location)?;

		// Loop iteration number (amount of commands in session)
		let mut command_count: u32 = 1;

		let mut is_real_command = false;
		// Takes input through CLI
		let read_line =
			line_main.readline(format!("[{}]({})> ", self.sub_account.as_str(), self.pair.as_str()).as_str());

		match read_line {
			Ok(read_line) => {
				// Add command to command history
				line_main.add_history_entry(read_line.as_str());

				// Command handling for Bybit exchange
				match bybit_inter::handle_commands(bybit_inter::CommandHandling {
					command_input: read_line.as_str(),
					current_sub_account: &mut self.sub_account,
					current_pair: &mut self.pair,
					bybit_api: &mut self.bybit_api,
					//&mut q_account,
					_terminal_is_wide: &mut self.desktop_terminal,
					database_info: &self.db_info,
				})
				.await
				{
					Ok(x) => {
						if x {
							is_real_command = true
						}
					}
					Err(_e) => {
						bl();
						// Typing "q" can cause this
						termbug::error("Function Exit: {_e:?}");
						bl();
						//continue;
					}
				};

				// Exchange-unrelated miscellaneous commands
				match misc::handle_commands(
					// Make this a struct one day (surely pepelaugh)
					read_line.as_str(),
					&mut self.desktop_terminal,
					command_count,
					//&mut db_info,
				)
				.await
				{
					Ok(x) => {
						if x {
							is_real_command = true
						}
					}
					Err(_e) => {
						termbug::error("Function Exited: {_e:?}");
						//continue;
					}
				}

				// Add padding
				bl();
			}
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
				//break;
			}
			Err(_e) => {
				termbug::error("Cannot Readline: {_e:?}");
				//break;
			}
		}
		if is_real_command {
			line_main.append_history(&line_main_location)?;
		}
		command_count += 1;
		
		Ok(())
	}
    */
}