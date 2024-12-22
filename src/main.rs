use std::{io, thread, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend}, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem}, Terminal
};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let menu_items = vec!["Install Arch Linux", "Exit"];
    let mut selected_index = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [Constraint::Percentage(80), Constraint::Percentage(20)].as_ref(),
                )
                .split(size);

            // Create menu items
            let items: Vec<ListItem> = menu_items
                .iter()
                .map(|m| ListItem::new(*m))
                .collect();

            // Highlight the selected menu item
            let menu = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Main Menu"))
                .highlight_symbol(">> ");

            // Render menu
            let mut state = tui::widgets::ListState::default();
            state.select(Some(selected_index));
            f.render_stateful_widget(menu, chunks[0], &mut state);
        })?;

        // Handle user input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < menu_items.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => match selected_index {
                    0 => {
                        let selected_disk = select_disk(terminal)?;
                        println!("Selected disk: {}", selected_disk);
                        // TODO: Proceed with formating
                    }
                    2 => break, // Exit
                    _ => {}
                },
                KeyCode::Esc => break,
                _ => {}
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn get_available_disks() -> Vec<String> {
    let output = std::process::Command::new("lsblk")
        .arg("-d") // List only top level devices
        .arg("-n") // Ignore headings
        .arg("-o") // Output only name and type
        .arg("NAME")
        .arg("TYPE")
        .output()
        .expect("Failed to get disks");



    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 && parts[1] == "disk" {
                Some(format!("/dev/{}", parts[0])) // Add the dev prefix
            } else {
                None
            }
        })
        .collect()
}

fn select_disk<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<String> {
    let disks = get_available_disks();
    if disks.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No disks found"));
    }

    let mut selected_index = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create Disk menu
            let items: Vec<ListItem> = disks
                .iter()
                .map(|disk| ListItem::new(disk.as_str()))
                .collect();

            let menu = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select a disk"))
                .highlight_symbol(">> ");

            let mut state = tui::widgets::ListState::default();
            state.select(Some(selected_index));
            f.render_stateful_widget(menu, size, &mut state);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < disks.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    return Ok(disks[selected_index].clone());
                }
                KeyCode::Esc => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Disk selection canceled"));
                }
                _ => {}
            }
        }
    }
}
