use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::text::Line;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use std::cell::Cell;

use crate::ascii_animation::AsciiAnimation;
use crate::key_hint::KeyBindingListExt;
use crate::onboarding::keys;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::tui::FrameRequester;

use super::onboarding_screen::StepState;

const MIN_ANIMATION_HEIGHT: u16 = 37;
const MIN_ANIMATION_WIDTH: u16 = 60;

pub(crate) struct WelcomeWidget {
    pub is_logged_in: bool,
    animation: AsciiAnimation,
    animations_enabled: bool,
    animations_suppressed: Cell<bool>,
    layout_area: Cell<Option<Rect>>,
}

impl KeyboardHandler for WelcomeWidget {
    /// Rotate the welcome animation when the fixed toggle shortcut fires.
    ///
    /// The key list includes compatibility variants for terminals that report
    /// modifier bits differently.
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !self.animations_enabled {
            return;
        }
        if key_event.kind == KeyEventKind::Press && keys::TOGGLE_ANIMATION.is_pressed(key_event) {
            tracing::warn!("Welcome background to press '.'");
            let _ = self.animation.pick_random_variant();
        }
    }
}

impl WelcomeWidget {
    pub(crate) fn new(
        is_logged_in: bool,
        request_frame: FrameRequester,
        animations_enabled: bool,
    ) -> Self {
        Self {
            is_logged_in,
            animation: AsciiAnimation::new(request_frame),
            animations_enabled,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
        }
    }

    pub(crate) fn update_layout_area(&self, area: Rect) {
        self.layout_area.set(Some(area));
    }

    pub(crate) fn set_animations_suppressed(&self, suppressed: bool) {
        self.animations_suppressed.set(suppressed);
    }
}

impl WidgetRef for &WelcomeWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        if self.animations_enabled && !self.animations_suppressed.get() {
            self.animation.schedule_next_frame();
        }

        let layout_area = self.layout_area.get().unwrap_or(area);
        // Skip the animation entirely when the viewport is too small so we don't clip frames.
        let show_animation = self.animations_enabled
            && !self.animations_suppressed.get()
            && layout_area.height >= MIN_ANIMATION_HEIGHT
            && layout_area.width >= MIN_ANIMATION_WIDTH;

        if show_animation {
            let frame = self.animation.current_frame();
            let lines: Vec<Line> = frame.lines().map(Into::into).collect();
            Paragraph::new(lines).render(area, buf);
        } else {
            self.render_minimal_fallback(area, buf);
        }
    }
}

impl WelcomeWidget {
    fn render_minimal_fallback(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol("o");
        }
    }
}

impl StepStateProvider for WelcomeWidget {
    fn get_step_state(&self) -> StepState {
        match self.is_logged_in {
            true => StepState::Hidden,
            false => StepState::Complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    static VARIANT_A: [&str; 1] = ["frame-a"];
    static VARIANT_B: [&str; 1] = ["frame-b"];
    static VARIANTS: [&[&str]; 2] = [&VARIANT_A, &VARIANT_B];

    fn occupied_rows(buf: &Buffer) -> usize {
        (0..buf.area.height)
            .filter(|&y| {
                (0..buf.area.width).any(|x| {
                    let cell = &buf[(x, y)];
                    let has_symbol = !cell.symbol().trim().is_empty();
                    let has_style = cell.fg != ratatui::style::Color::Reset
                        || cell.bg != ratatui::style::Color::Reset
                        || !cell.modifier.is_empty();
                    has_symbol || has_style
                })
            })
            .count()
    }

    #[test]
    fn welcome_renders_logo_on_first_draw() {
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::test_dummy(),
            /*animations_enabled*/ true,
        );
        let area = Rect::new(0, 0, MIN_ANIMATION_WIDTH, MIN_ANIMATION_HEIGHT);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(occupied_rows(&buf) > 0);
        assert!(!rendered.contains("Welcome"));
        assert!(!rendered.contains("Codex"));
        assert!(
            rendered
                .chars()
                .any(|ch| matches!(ch, 'a' | 'n' | 'z' | 'o' | 't' | 'h')),
            "expected visible logo glyphs"
        );
    }

    #[test]
    fn welcome_uses_minimal_fallback_below_height_breakpoint() {
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::test_dummy(),
            /*animations_enabled*/ true,
        );
        let area = Rect::new(0, 0, MIN_ANIMATION_WIDTH, MIN_ANIMATION_HEIGHT - 1);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        assert_eq!(occupied_rows(&buf), 1);
        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(rendered.contains("o"));
        assert!(!rendered.contains("Welcome"));
    }

    #[test]
    fn ctrl_dot_changes_animation_variant() {
        let mut widget = WelcomeWidget {
            is_logged_in: false,
            animation: AsciiAnimation::with_variants(
                FrameRequester::test_dummy(),
                &VARIANTS,
                /*variant_idx*/ 0,
            ),
            animations_enabled: true,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
        };

        let before = widget.animation.current_frame();
        widget.handle_key_event(KeyEvent::new(KeyCode::Char('.'), KeyModifiers::CONTROL));
        let after = widget.animation.current_frame();

        assert_ne!(
            before, after,
            "expected ctrl+. to switch welcome animation variant"
        );
    }

    #[test]
    fn ctrl_shift_dot_changes_animation_variant() {
        let mut widget = WelcomeWidget {
            is_logged_in: false,
            animation: AsciiAnimation::with_variants(
                FrameRequester::test_dummy(),
                &VARIANTS,
                /*variant_idx*/ 0,
            ),
            animations_enabled: true,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
        };

        let before = widget.animation.current_frame();
        widget.handle_key_event(KeyEvent::new(
            KeyCode::Char('.'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
        let after = widget.animation.current_frame();

        assert_ne!(
            before, after,
            "expected ctrl+shift+. to switch welcome animation variant"
        );
    }
}
