use std::{fs::{self, File}, io::{Read, Result}, path::{PathBuf}};
use crossterm::event::{self, Event, KeyCode};
use image::ImageReader;
use ratatui::{
    Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Tabs}
};
use ratatui_image::{Resize, StatefulImage, protocol::StatefulProtocol};
use rodio::{Decoder, OutputStream};
use std::io::BufReader;
use rodio::Sink;
use ratatui_image::picker::Picker;
use std::time::{Instant, Duration};
use rand::{seq::SliceRandom, thread_rng};

struct App {
    active_tab: usize,
    songs: Vec<(String, PathBuf)>,
    state: ListState,
    image_state: Option<StatefulProtocol>,
    picker: Picker,
    is_playing: bool,
    sink: Sink,
    start_instant: Option<Instant>,
    total_duration: Option<Duration>,
    elapsed_time: Duration,
    lyrics: Option<PathBuf>,
    lyrics_content: Option<String>
}

fn read_music_folder() -> Vec<PathBuf> {
    let entries = fs::read_dir("/home/saadwasim/Music").unwrap();

    let mp3_files: Vec<_> = entries
        .filter_map(|res| res.ok())
        .map(|e| e.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "mp3"))
        .collect();
    mp3_files
}

fn play_selected_song(app: &mut App, music_list: &[PathBuf]) {
    if let Some(index) = app.state.selected() {
        let path = &music_list[index];

        let mut lyrics_path = path.clone();
        lyrics_path.set_extension("lrc");

        if lyrics_path.exists() {
            let mut file = File::open(lyrics_path).unwrap();
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            app.lyrics_content = Some(buf);
        } else {
            app.lyrics_content = None;
        }

        app.total_duration = mp3_duration::from_path(path).ok();
        app.elapsed_time = Duration::from_secs(0);
        app.start_instant = Some(Instant::now());
        app.is_playing = true;

        let mut image_path = path.clone();
        image_path.set_extension("jpeg");
        if image_path.exists() {
            if let Ok(dyn_img) = ImageReader::open(image_path).unwrap().decode() {
                app.image_state = Some(app.picker.new_resize_protocol(dyn_img));
            }
        } else {
            app.image_state = None;
        }

        app.sink.clear();
        let file = BufReader::new(File::open(path).unwrap());
        if let Ok(source) = Decoder::new(file) {
            app.sink.append(source);
            app.sink.play();
        }
    }
}



fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let music_list = read_music_folder();
    let mut song_names = vec![];

    
    for path in music_list.iter(){
        song_names.push(path.file_name().unwrap().to_str().unwrap().to_string());
    }
    let picker = Picker::halfblocks();
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    
    let sink = Sink::try_new(&stream_handle).unwrap();
    let music_vec: Vec<_> = song_names.into_iter().zip(music_list.into_iter()).collect();
    let mut app = App { 
        active_tab: 1,
        songs: music_vec,
        state: ListState::default().with_selected(Some(0)),
        image_state: Option::None,
        picker,
        is_playing: false,
        sink,
        start_instant: None,
        total_duration: None,
        elapsed_time: Duration::from_secs(0),
        lyrics: None,
        lyrics_content: None
    };

    
    loop {
        let (songs, music_list): (Vec<String>, Vec<PathBuf>) = app.songs.clone().into_iter().unzip();

        if app.is_playing && app.sink.empty() {
            if let Some(current_index) = app.state.selected() {
                let next_index = (current_index + 1) % music_list.len();
                app.state.select(Some(next_index));
                
                play_selected_song(&mut app, &music_list);
            }
        }
        
        terminal.draw(|frame| ui(frame, &mut app, &songs))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                KeyCode::Char('r') => {
                    let mut rng = thread_rng();
                    
                    let current_song_name = app.state.selected().map(|i| app.songs[i].0.clone());

                    app.songs.shuffle(&mut rng);

                    if let Some(name) = current_song_name {
                        if let Some(new_pos) = app.songs.iter().position(|s| s.0 == name) {
                            app.state.select(Some(new_pos));
                        }
                    }
                },
                    KeyCode::Char('1') => app.active_tab = 0,
                    KeyCode::Char('2') => app.active_tab = 1,
                    KeyCode::Char('3') => app.active_tab = 2,
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('w') => app.state.select_previous(),
                    KeyCode::Down | KeyCode::Char('s') => app.state.select_next(),
                    KeyCode::Enter => {
                        app.active_tab = 1;
                        
                        if let Some(index) = app.state.selected() {

                            let path = &music_list[index];
                            let mut lyrics_path = path.clone();
                            lyrics_path.set_extension("lrc");

                            if lyrics_path.exists() {
                                let mut file = File::open(lyrics_path).unwrap();
                                let mut buf = String::new();
                                file.read_to_string(&mut buf).unwrap();
                                app.lyrics_content = Some(buf);
                            } else {
                                app.lyrics_content = None;
                            }

                            let mut image_path = path.clone();
                            image_path.set_extension("jpeg");
                            if image_path.exists(){
                                if let Ok(dyn_img) = ImageReader::open(image_path).unwrap().decode() {
                                    app.image_state = Some(app.picker.new_resize_protocol(dyn_img))
                                } 
                            } else {
                                app.image_state = None;
                            }

                            let mut lyrics_path = path.clone();
                            lyrics_path.set_extension("lrc");
                            if lyrics_path.exists(){
                                app.lyrics = Some(lyrics_path);
                            }

                            app.sink.clear();

                            let file = BufReader::new(File::open(path).unwrap());
                            let source = Decoder::new(file).unwrap();
                            app.total_duration = mp3_duration::from_path(path).ok();
                            app.start_instant = Some(Instant::now());
                            app.elapsed_time = Duration::from_secs(0);
                            
                            app.is_playing = true;

                            app.sink.append(source);
                            app.sink.play();
                        }
                    }
                    KeyCode::Char(' ') => {
                        if app.is_playing {
                            app.sink.pause();
                            app.is_playing = false;

                            if let Some(start) = app.start_instant {
                                app.elapsed_time += start.elapsed();
                            }

                            app.start_instant = None;
                            

                        } else {
                            app.sink.play();
                            app.is_playing = true;
                            app.start_instant = Some(Instant::now());
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if let Some(current) = app.state.selected() {
                            let next = (current + 1) % music_list.len();
                            app.state.select(Some(next));
                            play_selected_song(&mut app, &music_list);
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Left => {
                        if let Some(current) = app.state.selected() {
                            let next = if current == 0 {
                                music_list.len() - 1 // Wrap to the end
                            } else {
                                current - 1
                            };
                            app.state.select(Some(next));
                            play_selected_song(&mut app, &music_list);
                        }
                    }
                    _ => {},

                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App, songs: &Vec<String>) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    render_tabs(frame, app, main_layout[0]);

    match app.active_tab {
        0 => {
            render_library(frame, &songs ,&mut app.state , main_layout[1]);
        
        },
        1 => render_player(frame, app,main_layout[1]),
        2 => lyrics(frame, app, main_layout[1]),
        _ => {}
    }
}

fn lyrics(frame: &mut Frame, app: &App, area: Rect) {
    match &app.lyrics_content {
        None => {
            frame.render_widget(
                Paragraph::new("No lyrics loaded").alignment(Alignment::Center),
                area
            );
        }
        Some(text) => {
            frame.render_widget(
                Paragraph::new(text.clone()).alignment(Alignment::Center),
                area
            );
        }
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        Line::from(vec![Span::raw("1: "), Span::styled("Library", Color::Cyan)]),
        Line::from(vec![Span::raw("2: "), Span::styled("Now Playing", Color::Magenta)]),
        Line::from(vec![Span::raw("3: "), Span::styled("Lyrics", Color::Reset)])
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" SpoTUI "))
        .select(app.active_tab)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Reset));
    frame.render_widget(tabs, area);
}

fn render_library(frame: &mut Frame, songs: &[String], state: &mut ListState, area: Rect) {
    
    let items: Vec<ListItem> = songs.iter().map(|s| ListItem::new(format!(" 🎵 {}", s))).collect();
    let list = List::new(items).block(Block::default().title(" Library ").borders(Borders::ALL)).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    frame.render_stateful_widget(list, area, state);
}

fn render_player(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(area);

    let banner_block = Block::default()
        .borders(Borders::ALL)
        .title(" Album Art ");

    let inner_area = banner_block.inner(layout[0]);
    frame.render_widget(banner_block, layout[0]);
    
    if let Some(ref mut protocol) = app.image_state {
        let image_widget = StatefulImage::new().resize(Resize::Scale(None));
        frame.render_stateful_widget(image_widget, inner_area.centered(Constraint::Percentage(80), Constraint::Percentage(80)), protocol);
    } else {
        frame.render_widget(Paragraph::new("\nNo music playing rn.").alignment(Alignment::Center), layout[0]);
    }

    // let placeholder_text = Paragraph::new("\n\n[ KITTY IMAGE DATA RENDERS HERE ]\n(Use ratatui-image crate for this Rect)")
    //     .alignment(Alignment::Center)
    //     .block(banner_block)
    //     .dim();
    
    // frame.render_widget(placeholder_text, layout[0]);

    let total_elapsed = if app.is_playing {
        app.elapsed_time + app.start_instant.map(|i| i.elapsed()).unwrap_or_default()
    } else {
        app.elapsed_time
    };

    let elapsed_secs = total_elapsed.as_secs_f64();
    let total_secs = app.total_duration.map(|d| d.as_secs_f64()).unwrap_or(1.0);
    let ratio = (elapsed_secs / total_secs).min(1.0);

    let elapsed_secs = total_elapsed.as_secs();
    let duration_secs = app.total_duration.unwrap_or_default().as_secs();
    
    let song_label = format!(
        " {:02}:{:02} / {:02}:{:02} ",
        elapsed_secs / 60, elapsed_secs % 60,
        duration_secs / 60, duration_secs % 60
    );

    let progress_bar = Gauge::default()
        .block(Block::default().title(" Song Progress ").borders(Borders::LEFT | Borders::RIGHT))
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray))
        .ratio(ratio)
        .label(song_label);
    
    frame.render_widget(progress_bar, layout[1]);

    let play_pause_label = if app.is_playing { "⏹ PAUSE" } else { "▶ PLAY" };

    let controls = Line::from(vec![
        Span::styled(" [J] ← BACK ", Style::default().bg(Color::Blue).fg(Color::White)),
        Span::raw("  "),
        Span::styled(format!(" [SPACE] {} ", play_pause_label), Style::default().bg(Color::Red).fg(Color::White).bold()),
        Span::raw("  "),
        Span::styled(" [L] → NEXT ", Style::default().bg(Color::Blue).fg(Color::White)),
    ]);

    let control_panel = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(control_panel, layout[2]);
}
