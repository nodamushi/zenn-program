use std::io;

use crossterm::event::KeyModifiers;
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
    DefaultTerminal,
};

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut msg: String = "No key event".into();
    loop {
        let text = format!(
            "\nKey Input Test! (press 'Ctrl+C' or 'q' to quit)\n\n {}",
            msg
        );
        terminal.draw(|frame| {
            let hello = Paragraph::new(text)
                .fg(Color::Rgb(255, 255, 255))
                .on_black()
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Show Key Event ")
                        .border_type(ratatui::widgets::BorderType::Double),
                );
            frame.render_widget(hello, frame.area());
        })?;
        msg = if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::NONE)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    _ => {}
                }
            }
            format!("{:?}", key)
        } else {
            "No key event".into()
        };
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let ret = run(terminal);
    ratatui::restore();
    ret
}
