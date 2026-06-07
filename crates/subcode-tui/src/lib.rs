// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

//! Terminal User Interface for SUB CODE.
//!
//! Provides a full-screen ratatui + crossterm TUI with a chat pane,
//! status bar, agent steps panel, and streaming LLM output. This module
//! is initialised when the user runs `subcode` with no subcommand.

use std::sync::Arc;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use subcode_core::agent::{AgentMode, AgentRunner};
use subcode_core::config::Config;
use subcode_core::context::ProjectContext;
use subcode_core::llm::{ChatMessage, LlmRouter};

/// A single message in the chat history.
#[derive(Debug, Clone)]
struct ChatEntry {
    /// `"user"`, `"assistant"`, or `"status"`.
    role: String,
    /// Full text of the message.
    content: String,
}

/// Full-screen terminal UI for interactive SUB CODE sessions.
pub struct Tui {
    config: Config,
    ctx: ProjectContext,
    llm: Arc<LlmRouter>,
    yolo: bool,
    /// Chat history displayed in the main pane.
    history: Vec<ChatEntry>,
    /// Current user input buffer.
    input: String,
    /// Vertical scroll offset for the chat pane.
    scroll: u16,
    /// Whether a previous session was resumed.
    resumed: bool,
}

impl Tui {
    /// Construct a new TUI, optionally resuming a previous session.
    pub async fn new(
        config: Config,
        ctx: ProjectContext,
        llm: LlmRouter,
        yolo: bool,
        resume: bool,
    ) -> Result<Self> {
        Ok(Self {
            config,
            ctx,
            llm: Arc::new(llm),
            yolo,
            history: Vec::new(),
            input: String::new(),
            scroll: 0,
            resumed: resume,
        })
    }

    /// Run the main TUI event loop until the user exits.
    pub async fn run(&mut self) -> Result<()> {
        // Set up terminal.
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.history.push(ChatEntry {
            role: "status".into(),
            content: format!(
                "SUB CODE v{} — model: {} | {}",
                env!("CARGO_PKG_VERSION"),
                self.config.llm.model,
                if self.yolo { "YOLO mode" } else { "interactive" }
            ),
        });

        if self.resumed {
            self.history.push(ChatEntry {
                role: "status".into(),
                content: "Session resumed.".into(),
            });
        }

        self.history.push(ChatEntry {
            role: "status".into(),
            content: "Type a message and press Enter. Ctrl-C to quit.".into(),
        });

        loop {
            terminal.draw(|f| self.render(f))?;

            // Poll for events with a short timeout so we stay responsive.
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match self.handle_key(key) {
                        KeyAction::Quit => break,
                        KeyAction::Send => {
                            let user_input = self.input.drain(..).collect::<String>();
                            if user_input.trim().is_empty() {
                                continue;
                            }
                            self.history.push(ChatEntry {
                                role: "user".into(),
                                content: user_input.clone(),
                            });

                            // Stream the LLM response.
                            self.stream_response(&user_input, &mut terminal).await?;
                        }
                        KeyAction::Continue => {}
                    }
                }
            }
        }

        // Restore terminal.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }

    /// Render the entire TUI layout.
    fn render(&self, f: &mut Frame<'_>) {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // status bar
                Constraint::Min(5),        // chat pane
                Constraint::Length(3),     // input field
            ])
            .split(size);

        self.render_status_bar(f, chunks[0]);
        self.render_chat(f, chunks[1]);
        self.render_input(f, chunks[2]);
    }

    /// Status bar showing model, project, and mode.
    fn render_status_bar(&self, f: &mut Frame<'_>, area: Rect) {
        let project_name = self
            .config
            .project
            .name
            .as_deref()
            .unwrap_or("(unnamed)");

        let status_text = format!(
            " ⚡ {} │ {} │ {}",
            self.config.llm.model,
            project_name,
            if self.yolo { "YOLO" } else { "interactive" }
        );

        let bar = Paragraph::new(status_text)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default());
        f.render_widget(bar, area);
    }

    /// Chat pane with scrollable history.
    fn render_chat(&self, f: &mut Frame<'_>, area: Rect) {
        let mut lines: Vec<Line<'_>> = Vec::new();

        for entry in &self.history {
            let (prefix, style) = match entry.role.as_str() {
                "user" => (
                    "You: ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                "assistant" => (
                    "SUB CODE: ",
                    Style::default().fg(Color::Yellow),
                ),
                _ => (
                    "• ",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ),
            };

            for (i, text_line) in entry.content.lines().enumerate() {
                let p = if i == 0 { prefix } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(p, style),
                    Span::styled(text_line.to_string(), style),
                ]));
            }
            lines.push(Line::from(""));
        }

        let chat = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Chat ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        f.render_widget(chat, area);
    }

    /// Input field at the bottom.
    fn render_input(&self, f: &mut Frame<'_>, area: Rect) {
        let input_display = format!("▸ {}", self.input);
        let input = Paragraph::new(input_display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input ")
                    .border_style(Style::default().fg(Color::Green)),
            );
        f.render_widget(input, area);
    }

    /// Handle a single key press and return the resulting action.
    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return KeyAction::Quit;
        }
        match key.code {
            KeyCode::Enter => KeyAction::Send,
            KeyCode::Char(c) => {
                self.input.push(c);
                KeyAction::Continue
            }
            KeyCode::Backspace => {
                self.input.pop();
                KeyAction::Continue
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                KeyAction::Continue
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Stream an LLM response into the chat pane, updating the terminal
    /// in real time as tokens arrive.
    async fn stream_response(
        &mut self,
        user_msg: &str,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let ctx_summary = self.ctx.to_llm_string(self.config.context.max_tokens);
        let system = format!(
            "You are SUB CODE, a terminal-native AI coding assistant.\n\
             Project context:\n{}",
            ctx_summary
        );

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system,
            },
            ChatMessage {
                role: "user".into(),
                content: user_msg.to_string(),
            },
        ];

        let stream = self
            .llm
            .chat_stream(messages)
            .await;

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                self.history.push(ChatEntry {
                    role: "status".into(),
                    content: format!("LLM error: {e}"),
                });
                return Ok(());
            }
        };

        // Push an empty assistant entry that we'll append tokens to.
        self.history.push(ChatEntry {
            role: "assistant".into(),
            content: String::new(),
        });
        let idx = self.history.len() - 1;

        tokio::pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                Ok(ev) => {
                    self.history[idx].content.push_str(&ev.token);
                    // Re-render to show streaming tokens.
                    terminal.draw(|f| self.render(f))?;
                    if ev.done {
                        break;
                    }
                }
                Err(e) => {
                    self.history[idx]
                        .content
                        .push_str(&format!("\n[stream error: {e}]"));
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Internal action returned by key handling.
enum KeyAction {
    /// Exit the TUI.
    Quit,
    /// Send the current input.
    Send,
    /// No special action; continue the event loop.
    Continue,
}
