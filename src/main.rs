use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    io,
    path::Path,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessesToUpdate, System};

// process data
#[derive(Clone)]
struct ProcessEntry {
    name: String,
    pids: Vec<Pid>,
    cpu: f32,
    mem_mb: u64,
}

// sort modes
#[derive(Clone, Copy)]
enum SortMode {
    Name,
    Cpu,
    Memory,
}

// get the actual app name
fn process_label(proc: &sysinfo::Process) -> String {
    if let Some(exe) = proc.exe()
        && let Some(name) = Path::new(exe).file_name()
    {
        return name.to_string_lossy().to_string();
    }
    proc.name().to_string_lossy().to_string()
}

// ignore weird system processes
fn is_valid(proc: &sysinfo::Process) -> bool {
    let name = proc.name().to_string_lossy();
    !name.is_empty() && !name.contains(':')
}

// group processes and calc stats
fn build_process_list(sys: &System) -> Vec<ProcessEntry> {
    let mut groups: HashMap<String, Vec<Pid>> = HashMap::new();

    for (pid, process) in sys.processes() {
        if !is_valid(process) {
            continue;
        }

        let key = process_label(process);
        groups.entry(key).or_default().push(*pid);
    }

    let mut list = Vec::new();

    for (name, pids) in groups {
        let mut total_cpu = 0.0;
        let mut max_mem = 0;

        for pid in &pids {
            if let Some(proc) = sys.process(*pid) {
                total_cpu += proc.cpu_usage();
                // get the highest memory used by a child process
                max_mem = max_mem.max(proc.memory());
            }
        }

        list.push(ProcessEntry {
            name,
            pids,
            cpu: total_cpu,
            // using mb instead of kb or bytes
            mem_mb: max_mem / 1024 / 1024,
        });
    }

    list
}

// sort stuff
fn sort_processes(list: &mut [ProcessEntry], mode: SortMode) {
    match mode {
        SortMode::Name => list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortMode::Cpu => list.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Ordering::Equal)),
        SortMode::Memory => list.sort_by(|a, b| b.mem_mb.cmp(&a.mem_mb)),
    }
}

// fuzzy search logic
fn filter_processes(
    all: &[ProcessEntry],
    query: &str,
    matcher: &SkimMatcherV2,
) -> Vec<ProcessEntry> {
    if query.is_empty() {
        return all.to_vec();
    }

    let mut scored: Vec<(i64, ProcessEntry)> = all
        .iter()
        .filter_map(|p| {
            matcher
                .fuzzy_match(&p.name, query)
                .map(|score| (score, p.clone()))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, p)| p).collect()
}

// colors for cpu
fn cpu_color(cpu: f32) -> Color {
    if cpu > 50.0 {
        Color::Red
    } else if cpu > 15.0 {
        Color::Yellow
    } else {
        Color::White
    }
}

// colors for ram
fn mem_color(mem: u64) -> Color {
    if mem > 2000 {
        Color::Red
    } else if mem > 500 {
        Color::Yellow
    } else {
        Color::White
    }
}

fn main() -> Result<(), io::Error> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let matcher = SkimMatcherV2::default();

    let mut sort_mode = SortMode::Name;

    let mut processes = build_process_list(&sys);
    sort_processes(&mut processes, sort_mode);

    let mut search_query = String::new();
    let mut filtered = processes.clone();

    // list states
    let mut list_state = ListState::default();
    let mut selected_index = 0;
    list_state.select(Some(0));

    let mut last_refresh = Instant::now();

    let mut search_mode = false;

    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // main loop
    loop {
        // refresh data every 2000ms
        if last_refresh.elapsed() > Duration::from_millis(2000) {
            sys.refresh_processes(ProcessesToUpdate::All, true);

            processes = build_process_list(&sys);
            sort_processes(&mut processes, sort_mode);
            filtered = filter_processes(&processes, &search_query, &matcher);

            last_refresh = Instant::now();
        }

        //selection bounds
        if filtered.is_empty() {
            selected_index = 0;
            list_state.select(None);
        } else if selected_index >= filtered.len() {
            selected_index = filtered.len() - 1;
            list_state.select(Some(selected_index));
        }

        // draw ui
        terminal.draw(|f| {
            let size = f.area();

            let items: Vec<ListItem> = filtered
                .iter()
                .map(|p| {
                    let name = format!("{} ({})", p.name, p.pids.len());
                    let cpu = format!("{:.1}%", p.cpu);
                    let mem = format!("{} MB", p.mem_mb);

                    let line = Line::from(vec![
                        Span::raw(format!("{:<30}", name)),
                        Span::styled(
                            format!("{:<10}", cpu),
                            Style::default().fg(cpu_color(p.cpu)),
                        ),
                        Span::styled(
                            format!("{:<15}", mem),
                            Style::default().fg(mem_color(p.mem_mb)),
                        ),
                    ]);

                    ListItem::new(line)
                })
                .collect();

            let sort_label = match sort_mode {
                SortMode::Name => "NAME",
                SortMode::Cpu => "CPU",
                SortMode::Memory => "MEM",
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!(
                            " [q] quit | [k] kill process | [s] sort: {} | [/] search: {} ",
                            sort_label, search_query
                        ))
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().bg(Color::Blue));

            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        // check key press
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('/') => {
                    search_mode = true;
                }

                KeyCode::Char('q') => break,

                KeyCode::Down => {
                    if selected_index < filtered.len().saturating_sub(1) {
                        selected_index += 1;
                        list_state.select(Some(selected_index));
                    }
                }

                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                        list_state.select(Some(selected_index));
                    }
                }

                KeyCode::Char('k') => {
                    // kill software
                    if let Some(entry) = filtered.get(selected_index) {
                        for pid in &entry.pids {
                            if let Some(proc) = sys.process(*pid) {
                                proc.kill();
                            }
                        }
                    }
                }

                KeyCode::Char('s') => {
                    // toggle sort
                    sort_mode = match sort_mode {
                        SortMode::Name => SortMode::Cpu,
                        SortMode::Cpu => SortMode::Memory,
                        SortMode::Memory => SortMode::Name,
                    };
                }

                KeyCode::Char(c) => {
                    // type to search
                    if c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ' {
                        search_query.push(c);
                        filtered = filter_processes(&processes, &search_query, &matcher);
                    }
                }

                KeyCode::Backspace => {
                    if search_mode {
                        search_query.pop();
                        filtered = filter_processes(&processes, &search_query, &matcher);
                    }
                }

                KeyCode::Esc => {
                    // clear search
                    if search_mode {
                        search_mode = false;
                        search_query.clear();
                        filtered = filter_processes(&processes, &search_query, &matcher);
                    }
                }

                _ => {}
            }
        }
    }

    // terminal cleanup
    crossterm::terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}
