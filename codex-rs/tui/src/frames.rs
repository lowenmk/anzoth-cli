use std::time::Duration;

// Embed animation frames for each variant at compile time.
macro_rules! frames_for {
    ($dir:literal) => {
        [
            include_str!(concat!("../frames/", $dir, "/frame_1.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_2.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_3.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_4.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_5.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_6.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_7.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_8.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_9.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_10.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_11.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_12.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_13.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_14.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_15.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_16.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_17.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_18.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_19.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_20.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_21.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_22.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_23.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_24.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_25.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_26.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_27.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_28.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_29.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_30.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_31.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_32.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_33.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_34.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_35.txt")),
            include_str!(concat!("../frames/", $dir, "/frame_36.txt")),
        ]
    };
}

pub(crate) const FRAMES_ANZOTH: [&str; 36] = frames_for!("anzoth");
pub(crate) const FRAMES_DEFAULT: [&str; 36] = frames_for!("default");
pub(crate) const FRAMES_CODEX: [&str; 36] = frames_for!("codex");
pub(crate) const FRAMES_OPENAI: [&str; 36] = frames_for!("openai");
pub(crate) const FRAMES_BLOCKS: [&str; 36] = frames_for!("blocks");
pub(crate) const FRAMES_DOTS: [&str; 36] = frames_for!("dots");
pub(crate) const FRAMES_HASH: [&str; 36] = frames_for!("hash");
pub(crate) const FRAMES_HBARS: [&str; 36] = frames_for!("hbars");
pub(crate) const FRAMES_VBARS: [&str; 36] = frames_for!("vbars");
pub(crate) const FRAMES_SHAPES: [&str; 36] = frames_for!("shapes");
pub(crate) const FRAMES_SLUG: [&str; 36] = frames_for!("slug");

pub(crate) const ALL_VARIANTS: &[&[&str]] = &[
    &FRAMES_ANZOTH,
    &FRAMES_DEFAULT,
    &FRAMES_CODEX,
    &FRAMES_OPENAI,
    &FRAMES_BLOCKS,
    &FRAMES_DOTS,
    &FRAMES_HASH,
    &FRAMES_HBARS,
    &FRAMES_VBARS,
    &FRAMES_SHAPES,
    &FRAMES_SLUG,
];

pub(crate) const FRAME_TICK_DEFAULT: Duration = Duration::from_millis(80);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ascii_animation::AsciiAnimation;
    use crate::tui::FrameRequester;

    fn frame_dimensions(frame: &str) -> (usize, usize) {
        let lines: Vec<&str> = frame.lines().collect();
        let height = lines.len();
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        (width, height)
    }

    fn occupied_cells(frame: &str) -> Vec<(usize, usize, char)> {
        frame
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .filter_map(move |(x, ch)| (ch != ' ').then_some((x, y, ch)))
            })
            .collect()
    }

    fn loop_continuity_score(left: &str, right: &str) -> f32 {
        let left_lines: Vec<&str> = left.lines().collect();
        let right_lines: Vec<&str> = right.lines().collect();
        let mut same = 0usize;
        let mut total = 0usize;
        for (l_line, r_line) in left_lines.iter().zip(right_lines.iter()) {
            for (l_ch, r_ch) in l_line.chars().zip(r_line.chars()) {
                total += 1;
                if (l_ch != ' ') == (r_ch != ' ') {
                    same += 1;
                }
            }
        }
        same as f32 / total as f32
    }

    #[test]
    fn anzoth_frames_are_embedded_first_and_uniform() {
        assert_eq!(FRAMES_ANZOTH.len(), 36);
        assert!(std::ptr::eq(ALL_VARIANTS[0], &FRAMES_ANZOTH));

        let mut expected_dims: Option<(usize, usize)> = None;
        for frame in FRAMES_ANZOTH {
            let dims = frame_dimensions(frame);
            assert_eq!(dims, (44, 21), "embedded Anzoth frame size changed");
            match expected_dims {
                Some(expected) => assert_eq!(dims, expected, "frame dimensions changed"),
                None => expected_dims = Some(dims),
            }

            let occupied = occupied_cells(frame);
            let letters: std::collections::BTreeSet<char> =
                occupied.iter().map(|(_, _, ch)| *ch).collect();
            let allowed: std::collections::BTreeSet<char> =
                ['a', 'n', 'z', 'o', 't', 'h'].into_iter().collect();
            assert!(
                letters.is_subset(&allowed),
                "frame contains unsupported glyphs: {letters:?}"
            );
            assert!(
                !occupied
                    .iter()
                    .any(|(_, _, ch)| matches!(ch, '░' | '▒' | '▓' | '█')),
                "frame contains block/shade glyphs"
            );
            assert!(
                !frame.contains("Anzoth") && !frame.contains("CLI") && !frame.contains("Codex"),
                "frame should not contain branding text"
            );
            let distinct_letters = letters.len();
            let bbox_width = occupied
                .iter()
                .map(|(x, _, _)| *x)
                .min()
                .zip(occupied.iter().map(|(x, _, _)| *x).max())
                .map(|(min_x, max_x)| max_x - min_x + 1)
                .unwrap_or(0);
            if bbox_width > 8 {
                assert!(
                    distinct_letters >= 3,
                    "frame should use at least three distinct letters when not edge-on"
                );
            }
        }

        assert!(
            loop_continuity_score(FRAMES_ANZOTH[0], FRAMES_ANZOTH[35]) >= 0.72,
            "frame 1 and frame 36 should form a clean loop"
        );
    }

    #[test]
    fn anzoth_is_the_initial_animation_variant() {
        let animation = AsciiAnimation::new(FrameRequester::test_dummy());
        assert_eq!(animation.variant_idx_for_tests(), 0);
    }
}
