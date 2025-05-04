use core::f64;
use std::{io, time::Duration};

use chrono::{DateTime, Local, TimeZone, Timelike, Utc};
use crossterm::event::{self, Event};
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::Frame;
use ratatui::{layout::Alignment, style::{Color, Style}, text::{Line, Text}, widgets::{Block, Borders, Paragraph, Wrap}};

const FG_COLOR: Color = Color::Rgb(167, 199, 231);
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Position {
    lon: f64,
    lat: f64,
    city: String,
}

fn main() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();


    let pos = get_position();
    loop {
        let _ = terminal.draw(|f| render(f, &pos).unwrap());

        if event::poll(Duration::from_millis(100))?  {
            if matches!(event::read()?, Event::Key(_)){
                break;
            }

        }
    };
    ratatui::restore();
    Ok(())
}

fn render(f: &mut Frame, pos: &Position) -> Result<(), io::Error>{
    let title = format!(" {}[{}, {}] ", pos.city, pos.lat, pos.lon);
    let utc = Utc::now();
    let now = Local::now();
    let geograhic = get_geographic_time(utc, pos.lon);

    let lines = vec![
        format!("{} {}", format_clock(utc)?, "UTC"),
        format!("{} {}", format_clock(now)?, "Local"),
        format!("{} {}", format_clock(geograhic)?, "Geographic"),
    ];

    let clock_text = Text::from(
        lines.iter().map(|l|
            Line::from(l.clone())
        ).collect::<Vec<Line>>()
    );

    let length = lines.len();
    let width = lines.iter().fold(0, |a, l| {
        if l.len() > a {
            l.len()
        } else {
            a
        }
    });

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(length as u16),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .split(f.area());

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width as u16),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .split(vertical_layout[1]);

    let paragraph = Paragraph::new(clock_text)
        .style(Style::default().fg(FG_COLOR).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    let block = Block::default()
        .title(title)
        .style(Style::default().fg(FG_COLOR).bg(Color::Black))
        .borders(Borders::ALL);

    f.render_widget(block, f.area());
    f.render_widget(paragraph, horizontal_layout[1]);

    Ok(())
}

fn get_position() -> Position {
    let res: Position = reqwest::blocking::get("http://ip-api.com/json")
        .unwrap()
        .json()
        .unwrap();
    return res
}

fn longitude_to_second(lat: f64) -> i64 {
    return (((43200/180) as f64)*lat) as i64
}

fn get_geographic_time(utc: DateTime<Utc>, lon: f64) -> DateTime<Utc> {
    let offset = longitude_to_second(lon);
    if offset > 0 {
        return utc + Duration::from_secs(offset as u64)
    } else {
        return utc - Duration::from_secs((-1*offset) as u64)
    };
}

fn format_clock<T: TimeZone>(time: DateTime<T>) -> Result<String, io::Error> {
    return Ok(format!("{:0>2}:{:0>2}:{:0>2}", time.hour(), time.minute(), time.second()));
}
