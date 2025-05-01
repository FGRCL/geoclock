use core::f64;
use std::{io, time::Duration};

use chrono::{DateTime, Local, TimeZone, Timelike, Utc};
use crossbeam_channel::{select, tick, unbounded, Receiver};
use crossterm::{event::DisableMouseCapture, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{backend::CrosstermBackend, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, Paragraph, Wrap}, Terminal};

const FG_COLOR: Color = Color::Rgb(167, 199, 231);
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Position {
    lon: f64,
}

fn main() -> Result<(), io::Error> {
    let mut terminal = setup_terminal()?;
    let ctrl_c_events = ctrl_channel().unwrap();
    let ticks = tick(Duration::from_secs(1));

    let lon = get_longitude();
    loop {
        select! {
            recv(ticks) -> _ => {
                render(&mut terminal, lon)?;
            }
            recv(ctrl_c_events) -> _ => {
                break;
            }
        };
    };
    let _ = cleanup_terminal::<CrosstermBackend<io::Stdout>>(&mut terminal);
    Ok(())
}

fn render(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, lon: f64) -> Result<(), io::Error>{
    let utc = Utc::now();
    let now = Local::now();
    let geograhic = get_geographic_time(utc, lon);
    let utc_s = format_clock(utc)?;
    let now_s = format_clock(now)?;
    let geographic_s = format_clock(geograhic)?;

    let utc_label = "UTC";
    let local_label = "Local";
    let geographic_label = "Geographic";
    let _ = terminal.draw(|f| {
        let size = f.size();
        let clock_text = vec![
            Spans::from(vec![
                Span::raw("UTC: "),
                Span::raw(utc_s),
            ]),
            Spans::from(vec![
                Span::raw("Loc: "),
                Span::raw(now_s),
            ]),
            Spans::from(vec![
                Span::raw("Geo: "),
                Span::raw(geographic_s),
            ]),
        ];
        let vertical_padding = size.height / 2 - clock_text.len() as u16;
        let mut text: Vec<Spans> = vec![""; vertical_padding as usize]
            .into_iter()
            .map(|e| Spans::from(vec![Span::raw(e)]))
            .collect();
        text.extend(clock_text.iter().cloned());

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(FG_COLOR).bg(Color::Black))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, f.size());
    });

    Ok(())
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = unbounded();
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;
    Ok(receiver)
}

fn get_longitude() -> f64 {
    let res: Position = reqwest::blocking::get("http://ip-api.com/json")
        .unwrap()
        .json()
        .unwrap();
    return res.lon
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

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error>{
    enable_raw_mode()?;
    let stdout = io::stdout();
    execute!(io::stdout(), EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    disable_raw_mode()?;
    return Ok(terminal);
}

fn cleanup_terminal<T>(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    // terminal.clear()?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    return Ok(());
}

fn format_clock<T: TimeZone>(time: DateTime<T>) -> Result<String, io::Error> {
    return Ok(format!("{:0>2}:{:0>2}:{:0>2}", time.hour(), time.minute(), time.second()));
}
