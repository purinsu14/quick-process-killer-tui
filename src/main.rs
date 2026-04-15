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

#[derive(Clone)]
struct ProcessEntry {
    name: String,
    pids: Vec<Pid>,
    cpu: f32,
    mem_mb: u64,
}

#[derive(Clone, Copy)]
enum SortMode {
    Name,
    Cpu,
    Memory,
}

// Two distinct input states
enum InputMode {
    Normal,
    Searching,
}

fn process_label(proc: &sysinfo::Process) -> String {
    if let Some(exe) = proc.exe()
        && let Some(name) = Path::new(exe).file_name()
    {
        return name.to_string_lossy().to_string();
    }
    proc.name().to_string_lossy().to_string()
}

fn is_valid(proc: &sysinfo::Process) -> bool {
    let name = proc.name().to_string_lossy();
    !name.is_empty() && !name.contains(':')
}

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
                max_mem = max_mem.max(proc.memory());
            }
        }

        list.push(ProcessEntry {
            name,
            pids,
            cpu: total_cpu,
            mem_mb: max_mem / 1024 / 1024,
        });
    }

    list
}

fn sort_processes(list: &mut [ProcessEntry], mode: SortMode) {
    match mode {
        SortMode::Name => list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortMode::Cpu => list.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Ordering::Equal)),
        SortMode::Memory => list.sort_by(|a, b| b.mem_mb.cmp(&a.mem_mb)),
    }
}

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

fn cpu_color(cpu: f32) -> Color {
    if cpu > 50.0 {
        Color::Red
    } else if cpu > 15.0 {
        Color::Yellow
    } else {
        Color::White
    }
}

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
    let mut input_mode = InputMode::Normal;

    let mut processes = build_process_list(&sys);
    sort_processes(&mut processes, sort_mode);

    let mut search_query = String::new();
    let mut filtered = processes.clone();

    let mut list_state = ListState::default();
    let mut selected_index = 0;
    list_state.select(Some(0));

    let mut last_refresh = Instant::now();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        if last_refresh.elapsed() > Duration::from_millis(2000) {
            sys.refresh_processes(ProcessesToUpdate::All, true);
            processes = build_process_list(&sys);
            sort_processes(&mut processes, sort_mode);
            filtered = filter_processes(&processes, &search_query, &matcher);
            last_refresh = Instant::now();
        }

        // clamp selection
        if filtered.is_empty() {
            selected_index = 0;
            list_state.select(None);
        } else if selected_index >= filtered.len() {
            selected_index = filtered.len() - 1;
            list_state.select(Some(selected_index));
        }

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

            // Title changes based on mode so user always knows their state
            let title = match input_mode {
                InputMode::Searching => format!(
                    " [esc] cancel | [enter] confirm | search: {}_",
                    search_query
                ),
                InputMode::Normal => {
                    if search_query.is_empty() {
                        format!(
                            " [q]uit | [k]ill | [s]ort: {} | [/] search ",
                            sort_label
                        )
                    } else {
                        format!(
                            " [q]uit | [k]ill | [s]ort: {} | [/] search | filter: {} | [esc] clear ",
                            sort_label, search_query
                        )
                    }
                }
            };

            let list = List::new(items)
                .block(Block::default().title(title).borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Blue));

            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            match input_mode {
                InputMode::Searching => match key.code {
                    // confirm search — lock it in and return to normal mode
                    KeyCode::Enter => {
                        input_mode = InputMode::Normal;
                        filtered = filter_processes(&processes, &search_query, &matcher);
                        selected_index = 0;
                        list_state.select(if filtered.is_empty() { None } else { Some(0) });
                    }

                    // cancel — clear query and return to normal
                    KeyCode::Esc => {
                        input_mode = InputMode::Normal;
                        search_query.clear();
                        filtered = processes.clone();
                        selected_index = 0;
                        list_state.select(if filtered.is_empty() { None } else { Some(0) });
                    }

                    KeyCode::Char(c) => {
                        search_query.push(c);
                        // live preview while typing
                        filtered = filter_processes(&processes, &search_query, &matcher);
                        selected_index = 0;
                        list_state.select(if filtered.is_empty() { None } else { Some(0) });
                    }

                    KeyCode::Backspace => {
                        search_query.pop();
                        filtered = filter_processes(&processes, &search_query, &matcher);
                        selected_index = 0;
                        list_state.select(if filtered.is_empty() { None } else { Some(0) });
                    }

                    _ => {}
                },

                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Char('/') => {
                        // enter search mode
                        input_mode = InputMode::Searching;
                    }

                    // esc in normal mode clears a locked-in filter
                    KeyCode::Esc => {
                        search_query.clear();
                        filtered = processes.clone();
                        selected_index = 0;
                        list_state.select(if filtered.is_empty() { None } else { Some(0) });
                    }

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
                        if let Some(entry) = filtered.get(selected_index) {
                            for pid in &entry.pids {
                                if let Some(proc) = sys.process(*pid) {
                                    proc.kill();
                                }
                            }
                            // force immediate refresh after kill
                            std::thread::sleep(Duration::from_millis(100));
                            sys.refresh_processes(ProcessesToUpdate::All, true);
                            processes = build_process_list(&sys);
                            sort_processes(&mut processes, sort_mode);
                            filtered = filter_processes(&processes, &search_query, &matcher);
                            last_refresh = Instant::now();
                        }
                    }

                    KeyCode::Char('s') => {
                        sort_mode = match sort_mode {
                            SortMode::Name => SortMode::Cpu,
                            SortMode::Cpu => SortMode::Memory,
                            SortMode::Memory => SortMode::Name,
                        };
                        sort_processes(&mut processes, sort_mode);
                        filtered = filter_processes(&processes, &search_query, &matcher);
                    }

                    _ => {}
                },
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}

