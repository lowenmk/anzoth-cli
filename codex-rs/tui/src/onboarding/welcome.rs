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
use std::time::Duration;
use std::time::Instant;

use crate::ascii_animation::AsciiAnimation;
use crate::frames::ANZOTH_SMALL_ALT_FRAME_HEIGHT;
use crate::frames::ANZOTH_SMALL_ALT_FRAME_WIDTH;
use crate::frames::ANZOTH_SMALL_ALT_SEQUENCE;
use crate::frames::FRAME_TICK_DEFAULT;
use crate::frames::FRAMES_ANZOTH_SMALL_ALT;
use crate::key_hint::KeyBindingListExt;
use crate::onboarding::keys;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::tui::FrameRequester;

use super::onboarding_screen::StepState;

pub(crate) const MIN_ANIMATED_LOGO_HEIGHT: u16 = 37;
pub(crate) const MIN_ANIMATED_LOGO_WIDTH: u16 = 60;
pub(crate) const MIN_SMALL_LOGO_HEIGHT: u16 = ANZOTH_SMALL_ALT_FRAME_HEIGHT;
pub(crate) const MIN_SMALL_LOGO_WIDTH: u16 = ANZOTH_SMALL_ALT_FRAME_WIDTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WelcomeLogoMode {
    Animated,
    SmallAnimation,
    Hidden,
}

struct SmallWelcomeAnimation {
    request_frame: FrameRequester,
    start: Cell<Instant>,
}

impl SmallWelcomeAnimation {
    fn new(request_frame: FrameRequester) -> Self {
        Self {
            request_frame,
            start: Cell::new(Instant::now()),
        }
    }

    fn reset(&self) {
        self.start.set(Instant::now());
    }

    fn schedule_next_frame(&self) {
        let tick_ms = FRAME_TICK_DEFAULT.as_millis();
        if tick_ms == 0 {
            self.request_frame.schedule_frame();
            return;
        }
        let elapsed_ms = self.start.get().elapsed().as_millis();
        let rem_ms = elapsed_ms % tick_ms;
        let delay_ms = if rem_ms == 0 {
            tick_ms
        } else {
            tick_ms - rem_ms
        };
        self.request_frame
            .schedule_frame_in(Duration::from_millis(delay_ms as u64));
    }

    fn current_frame(&self) -> &'static str {
        let tick_ms = FRAME_TICK_DEFAULT.as_millis();
        let tick = if tick_ms == 0 {
            0usize
        } else {
            (self.start.get().elapsed().as_millis() / tick_ms) as usize
        };
        FRAMES_ANZOTH_SMALL_ALT[ANZOTH_SMALL_ALT_SEQUENCE[tick % ANZOTH_SMALL_ALT_SEQUENCE.len()]]
    }

    #[cfg(test)]
    fn frame_index_for_tick(tick: usize) -> usize {
        ANZOTH_SMALL_ALT_SEQUENCE[tick % ANZOTH_SMALL_ALT_SEQUENCE.len()]
    }
}

pub(crate) struct WelcomeWidget {
    pub is_logged_in: bool,
    animation: AsciiAnimation,
    small_animation: SmallWelcomeAnimation,
    animations_enabled: bool,
    animations_suppressed: Cell<bool>,
    layout_area: Cell<Option<Rect>>,
    last_logo_mode: Cell<Option<WelcomeLogoMode>>,
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
            animation: AsciiAnimation::new(request_frame.clone()),
            small_animation: SmallWelcomeAnimation::new(request_frame),
            animations_enabled,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
            last_logo_mode: Cell::new(None),
        }
    }

    pub(crate) fn update_layout_area(&self, area: Rect) {
        self.layout_area.set(Some(area));
    }

    pub(crate) fn set_animations_suppressed(&self, suppressed: bool) {
        self.animations_suppressed.set(suppressed);
    }

    fn logo_mode(&self, layout_area: Rect) -> WelcomeLogoMode {
        if !self.animations_enabled || self.animations_suppressed.get() {
            return WelcomeLogoMode::Hidden;
        }
        if layout_area.height >= MIN_ANIMATED_LOGO_HEIGHT
            && layout_area.width >= MIN_ANIMATED_LOGO_WIDTH
        {
            WelcomeLogoMode::Animated
        } else if layout_area.height >= MIN_SMALL_LOGO_HEIGHT
            && layout_area.width >= MIN_SMALL_LOGO_WIDTH
        {
            WelcomeLogoMode::SmallAnimation
        } else {
            WelcomeLogoMode::Hidden
        }
    }
}

impl WidgetRef for &WelcomeWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let layout_area = self.layout_area.get().unwrap_or(area);
        let mode = self.logo_mode(layout_area);
        if self.last_logo_mode.get() != Some(mode) {
            if matches!(mode, WelcomeLogoMode::SmallAnimation) {
                self.small_animation.reset();
            }
            self.last_logo_mode.set(Some(mode));
        }
        match mode {
            WelcomeLogoMode::Animated => {
                self.animation.schedule_next_frame();
                let frame = self.animation.current_frame();
                let lines: Vec<Line> = frame.lines().map(Into::into).collect();
                Paragraph::new(lines).render(area, buf);
            }
            WelcomeLogoMode::SmallAnimation => self.render_small_animation(area, buf),
            WelcomeLogoMode::Hidden => {}
        }
    }
}

impl WelcomeWidget {
    fn render_small_animation(&self, area: Rect, buf: &mut Buffer) {
        self.small_animation.schedule_next_frame();
        let lines: Vec<Line> = self
            .small_animation
            .current_frame()
            .lines()
            .map(Into::into)
            .collect();
        Paragraph::new(lines).centered().render(area, buf);
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
    use tokio::sync::broadcast;
    use tokio::time::Duration;
    use tokio_util::time::FutureExt;

    use crate::frames::FRAME_TICK_DEFAULT;

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
        let area = Rect::new(0, 0, MIN_ANIMATED_LOGO_WIDTH, MIN_ANIMATED_LOGO_HEIGHT);
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
    fn welcome_uses_small_animation_in_medium_viewport() {
        let widget = WelcomeWidget {
            is_logged_in: false,
            animation: AsciiAnimation::with_variants(
                FrameRequester::test_dummy(),
                &VARIANTS,
                /*variant_idx*/ 0,
            ),
            small_animation: SmallWelcomeAnimation::new(FrameRequester::test_dummy()),
            animations_enabled: true,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
            last_logo_mode: Cell::new(None),
        };
        widget.small_animation.start.set(
            std::time::Instant::now()
                - Duration::from_millis(FRAME_TICK_DEFAULT.as_millis() as u64 * 100),
        );
        let area = Rect::new(0, 0, 80, 30);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        assert_eq!(widget.logo_mode(area), WelcomeLogoMode::SmallAnimation);
        let mut rendered = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(!rendered.contains("Codex"));
        assert!(!rendered.contains("Welcome"));
    }

    #[test]
    fn welcome_omits_logo_in_tiny_viewport() {
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::test_dummy(),
            /*animations_enabled*/ true,
        );
        let area = Rect::new(0, 0, MIN_SMALL_LOGO_WIDTH - 1, MIN_SMALL_LOGO_HEIGHT - 1);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        assert_eq!(occupied_rows(&buf), 0);
    }

    #[test]
    fn welcome_small_animation_sequence_matches_spec() {
        let expected = {
            let mut seq = vec![91usize, 91];
            seq.extend((1usize..=90).rev());
            seq.extend([0usize, 0, 0, 0]);
            seq.extend(1usize..=90);
            seq
        };
        let actual = (0..ANZOTH_SMALL_ALT_SEQUENCE.len())
            .map(SmallWelcomeAnimation::frame_index_for_tick)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn welcome_small_animation_schedules_animation_frames_only_when_visible() {
        let (draw_tx, mut draw_rx) = broadcast::channel(16);
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::new(draw_tx),
            /*animations_enabled*/ true,
        );
        let area = Rect::new(0, 0, 80, 30);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        tokio::time::advance(Duration::from_millis(100)).await;
        let draw = draw_rx
            .recv()
            .timeout(Duration::from_millis(10))
            .await
            .expect("timed out waiting for small animated redraw");
        assert!(draw.is_ok(), "small animation should schedule redraws");

        let tiny_area = Rect::new(0, 0, MIN_SMALL_LOGO_WIDTH - 1, MIN_SMALL_LOGO_HEIGHT - 1);
        let mut tiny_buf = Buffer::empty(tiny_area);
        (&widget).render(tiny_area, &mut tiny_buf);

        tokio::time::advance(Duration::from_millis(100)).await;
        let draw = draw_rx.recv().timeout(Duration::from_millis(10)).await;
        assert!(draw.is_err(), "hidden logo should not schedule redraws");
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn welcome_animated_logo_schedules_animation_frames() {
        let (draw_tx, mut draw_rx) = broadcast::channel(16);
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::new(draw_tx),
            /*animations_enabled*/ true,
        );
        let area = Rect::new(0, 0, MIN_ANIMATED_LOGO_WIDTH, MIN_ANIMATED_LOGO_HEIGHT);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        tokio::time::advance(Duration::from_millis(100)).await;
        let draw = draw_rx
            .recv()
            .timeout(Duration::from_millis(10))
            .await
            .expect("timed out waiting for animated redraw");
        assert!(draw.is_ok(), "animated logo should schedule redraws");
    }

    #[test]
    fn welcome_logo_mode_thresholds_are_explicit() {
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::test_dummy(),
            /*animations_enabled*/ true,
        );
        assert_eq!(
            widget.logo_mode(Rect::new(
                0,
                0,
                MIN_ANIMATED_LOGO_WIDTH,
                MIN_ANIMATED_LOGO_HEIGHT
            )),
            WelcomeLogoMode::Animated
        );
        assert_eq!(
            widget.logo_mode(Rect::new(0, 0, 80, 30)),
            WelcomeLogoMode::SmallAnimation
        );
        assert_eq!(
            widget.logo_mode(Rect::new(
                0,
                0,
                MIN_SMALL_LOGO_WIDTH - 1,
                MIN_SMALL_LOGO_HEIGHT - 1
            )),
            WelcomeLogoMode::Hidden
        );
    }

    #[test]
    fn welcome_animations_disabled_no_longer_selects_old_miniature() {
        let widget = WelcomeWidget::new(
            /*is_logged_in*/ false,
            FrameRequester::test_dummy(),
            /*animations_enabled*/ false,
        );
        let area = Rect::new(0, 0, 80, 30);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        assert_eq!(occupied_rows(&buf), 0);
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
            small_animation: SmallWelcomeAnimation::new(FrameRequester::test_dummy()),
            animations_enabled: true,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
            last_logo_mode: Cell::new(None),
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
            small_animation: SmallWelcomeAnimation::new(FrameRequester::test_dummy()),
            animations_enabled: true,
            animations_suppressed: Cell::new(false),
            layout_area: Cell::new(None),
            last_logo_mode: Cell::new(None),
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
