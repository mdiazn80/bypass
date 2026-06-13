use std::collections::HashSet;
use std::io::stdout;

use anyhow::Result;
use bypass_core::{CredentialContext, Vault};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Focus {
    Contexts,
    Vars,
}

struct App {
    vault: Vault,
    contexts: Vec<CredentialContext>,
    active: Option<String>,
    ctx_state: ListState,
    vars: Vec<(String, String)>,
    var_state: ListState,
    revealed: HashSet<String>,
    focus: Focus,
    should_quit: bool,
}

impl App {
    fn new(vault: Vault) -> Result<Self> {
        let contexts = vault.list_contexts()?;
        let active = vault.get_active()?;
        let mut ctx_state = ListState::default();
        if !contexts.is_empty() {
            ctx_state.select(Some(0));
        }
        let mut app = App {
            vault,
            contexts,
            active,
            ctx_state,
            vars: Vec::new(),
            var_state: ListState::default(),
            revealed: HashSet::new(),
            focus: Focus::Contexts,
            should_quit: false,
        };
        app.reload_vars()?;
        Ok(app)
    }

    fn selected_context(&self) -> Option<&CredentialContext> {
        self.ctx_state.selected().and_then(|i| self.contexts.get(i))
    }

    fn reload_vars(&mut self) -> Result<()> {
        self.revealed.clear();
        self.vars = match self.selected_context() {
            Some(ctx) => self
                .vault
                .vars(&ctx.name)?
                .into_iter()
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };
        self.var_state
            .select((!self.vars.is_empty()).then_some(0));
        Ok(())
    }

    fn move_selection(&mut self, delta: i32) -> Result<()> {
        match self.focus {
            Focus::Contexts => {
                if self.contexts.is_empty() {
                    return Ok(());
                }
                let next = step(self.ctx_state.selected(), delta, self.contexts.len());
                self.ctx_state.select(Some(next));
                self.reload_vars()?;
            }
            Focus::Vars => {
                if self.vars.is_empty() {
                    return Ok(());
                }
                let next = step(self.var_state.selected(), delta, self.vars.len());
                self.var_state.select(Some(next));
            }
        }
        Ok(())
    }

    fn toggle_active(&mut self) -> Result<()> {
        if let Some(ctx) = self.selected_context() {
            let name = ctx.name.clone();
            if self.active.as_deref() == Some(name.as_str()) {
                self.vault.set_active(None)?;
                self.active = None;
            } else {
                self.vault.set_active(Some(&name))?;
                self.active = Some(name);
            }
        }
        Ok(())
    }

    fn toggle_reveal(&mut self) {
        if let Some(i) = self.var_state.selected() {
            if let Some((key, _)) = self.vars.get(i) {
                if !self.revealed.remove(key) {
                    self.revealed.insert(key.clone());
                }
            }
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.ui(f))?;
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    KeyCode::Tab => {
                        self.focus = match self.focus {
                            Focus::Contexts => Focus::Vars,
                            Focus::Vars => Focus::Contexts,
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => self.move_selection(1)?,
                    KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1)?,
                    KeyCode::Enter => self.toggle_active()?,
                    KeyCode::Char('r') => self.toggle_reveal(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(f.area());

        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(outer[0]);

        self.render_contexts(f, panes[0]);
        self.render_vars(f, panes[1]);
        self.render_footer(f, outer[1]);
    }

    fn render_contexts(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .contexts
            .iter()
            .map(|c| {
                let marker = if self.active.as_deref() == Some(c.name.as_str()) {
                    "● "
                } else {
                    "  "
                };
                ListItem::new(Line::from(vec![
                    Span::styled(marker, Style::default().fg(Color::Green)),
                    Span::raw(c.name.clone()),
                ]))
            })
            .collect();

        let border = focus_style(self.focus == Focus::Contexts);
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border)
                    .title(" Contexts "),
            )
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.ctx_state);
    }

    fn render_vars(&mut self, f: &mut Frame, area: Rect) {
        let title = match self.selected_context() {
            Some(c) if !c.description.is_empty() => format!(" {} - {} ", c.name, c.description),
            Some(c) => format!(" {} ", c.name),
            None => " Variables ".to_string(),
        };

        let items: Vec<ListItem> = self
            .vars
            .iter()
            .map(|(key, value)| {
                let shown = if self.revealed.contains(key) {
                    value.clone()
                } else {
                    "••••••••".to_string()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{key} "), Style::default().fg(Color::Green)),
                    Span::styled("= ", Style::default().fg(Color::DarkGray)),
                    Span::raw(shown),
                ]))
            })
            .collect();

        let border = focus_style(self.focus == Focus::Vars);
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border)
                    .title(title),
            )
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.var_state);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let hint = "Tab switch pane  •  j/k move  •  Enter set active  •  r reveal/hide  •  q quit";
        let footer = Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, area);
    }
}

fn step(current: Option<usize>, delta: i32, len: usize) -> usize {
    let cur = current.unwrap_or(0) as i32;
    let next = (cur + delta).rem_euclid(len as i32);
    next as usize
}

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

/// Entry point for the interactive TUI.
pub fn run() -> Result<()> {
    let vault = crate::cmd::open_vault()?;
    let mut app = App::new(vault)?;

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
