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

pub(crate) const FRAMES_ANZOTH_SMALL_ALT: [&str; 92] = [
    include_str!("../frames/anzoth_small_alt/frame_0.txt"),
    include_str!("../frames/anzoth_small_alt/frame_1.txt"),
    include_str!("../frames/anzoth_small_alt/frame_2.txt"),
    include_str!("../frames/anzoth_small_alt/frame_3.txt"),
    include_str!("../frames/anzoth_small_alt/frame_4.txt"),
    include_str!("../frames/anzoth_small_alt/frame_5.txt"),
    include_str!("../frames/anzoth_small_alt/frame_6.txt"),
    include_str!("../frames/anzoth_small_alt/frame_7.txt"),
    include_str!("../frames/anzoth_small_alt/frame_8.txt"),
    include_str!("../frames/anzoth_small_alt/frame_9.txt"),
    include_str!("../frames/anzoth_small_alt/frame_10.txt"),
    include_str!("../frames/anzoth_small_alt/frame_11.txt"),
    include_str!("../frames/anzoth_small_alt/frame_12.txt"),
    include_str!("../frames/anzoth_small_alt/frame_13.txt"),
    include_str!("../frames/anzoth_small_alt/frame_14.txt"),
    include_str!("../frames/anzoth_small_alt/frame_15.txt"),
    include_str!("../frames/anzoth_small_alt/frame_16.txt"),
    include_str!("../frames/anzoth_small_alt/frame_17.txt"),
    include_str!("../frames/anzoth_small_alt/frame_18.txt"),
    include_str!("../frames/anzoth_small_alt/frame_19.txt"),
    include_str!("../frames/anzoth_small_alt/frame_20.txt"),
    include_str!("../frames/anzoth_small_alt/frame_21.txt"),
    include_str!("../frames/anzoth_small_alt/frame_22.txt"),
    include_str!("../frames/anzoth_small_alt/frame_23.txt"),
    include_str!("../frames/anzoth_small_alt/frame_24.txt"),
    include_str!("../frames/anzoth_small_alt/frame_25.txt"),
    include_str!("../frames/anzoth_small_alt/frame_26.txt"),
    include_str!("../frames/anzoth_small_alt/frame_27.txt"),
    include_str!("../frames/anzoth_small_alt/frame_28.txt"),
    include_str!("../frames/anzoth_small_alt/frame_29.txt"),
    include_str!("../frames/anzoth_small_alt/frame_30.txt"),
    include_str!("../frames/anzoth_small_alt/frame_31.txt"),
    include_str!("../frames/anzoth_small_alt/frame_32.txt"),
    include_str!("../frames/anzoth_small_alt/frame_33.txt"),
    include_str!("../frames/anzoth_small_alt/frame_34.txt"),
    include_str!("../frames/anzoth_small_alt/frame_35.txt"),
    include_str!("../frames/anzoth_small_alt/frame_36.txt"),
    include_str!("../frames/anzoth_small_alt/frame_37.txt"),
    include_str!("../frames/anzoth_small_alt/frame_38.txt"),
    include_str!("../frames/anzoth_small_alt/frame_39.txt"),
    include_str!("../frames/anzoth_small_alt/frame_40.txt"),
    include_str!("../frames/anzoth_small_alt/frame_41.txt"),
    include_str!("../frames/anzoth_small_alt/frame_42.txt"),
    include_str!("../frames/anzoth_small_alt/frame_43.txt"),
    include_str!("../frames/anzoth_small_alt/frame_44.txt"),
    include_str!("../frames/anzoth_small_alt/frame_45.txt"),
    include_str!("../frames/anzoth_small_alt/frame_46.txt"),
    include_str!("../frames/anzoth_small_alt/frame_47.txt"),
    include_str!("../frames/anzoth_small_alt/frame_48.txt"),
    include_str!("../frames/anzoth_small_alt/frame_49.txt"),
    include_str!("../frames/anzoth_small_alt/frame_50.txt"),
    include_str!("../frames/anzoth_small_alt/frame_51.txt"),
    include_str!("../frames/anzoth_small_alt/frame_52.txt"),
    include_str!("../frames/anzoth_small_alt/frame_53.txt"),
    include_str!("../frames/anzoth_small_alt/frame_54.txt"),
    include_str!("../frames/anzoth_small_alt/frame_55.txt"),
    include_str!("../frames/anzoth_small_alt/frame_56.txt"),
    include_str!("../frames/anzoth_small_alt/frame_57.txt"),
    include_str!("../frames/anzoth_small_alt/frame_58.txt"),
    include_str!("../frames/anzoth_small_alt/frame_59.txt"),
    include_str!("../frames/anzoth_small_alt/frame_60.txt"),
    include_str!("../frames/anzoth_small_alt/frame_61.txt"),
    include_str!("../frames/anzoth_small_alt/frame_62.txt"),
    include_str!("../frames/anzoth_small_alt/frame_63.txt"),
    include_str!("../frames/anzoth_small_alt/frame_64.txt"),
    include_str!("../frames/anzoth_small_alt/frame_65.txt"),
    include_str!("../frames/anzoth_small_alt/frame_66.txt"),
    include_str!("../frames/anzoth_small_alt/frame_67.txt"),
    include_str!("../frames/anzoth_small_alt/frame_68.txt"),
    include_str!("../frames/anzoth_small_alt/frame_69.txt"),
    include_str!("../frames/anzoth_small_alt/frame_70.txt"),
    include_str!("../frames/anzoth_small_alt/frame_71.txt"),
    include_str!("../frames/anzoth_small_alt/frame_72.txt"),
    include_str!("../frames/anzoth_small_alt/frame_73.txt"),
    include_str!("../frames/anzoth_small_alt/frame_74.txt"),
    include_str!("../frames/anzoth_small_alt/frame_75.txt"),
    include_str!("../frames/anzoth_small_alt/frame_76.txt"),
    include_str!("../frames/anzoth_small_alt/frame_77.txt"),
    include_str!("../frames/anzoth_small_alt/frame_78.txt"),
    include_str!("../frames/anzoth_small_alt/frame_79.txt"),
    include_str!("../frames/anzoth_small_alt/frame_80.txt"),
    include_str!("../frames/anzoth_small_alt/frame_81.txt"),
    include_str!("../frames/anzoth_small_alt/frame_82.txt"),
    include_str!("../frames/anzoth_small_alt/frame_83.txt"),
    include_str!("../frames/anzoth_small_alt/frame_84.txt"),
    include_str!("../frames/anzoth_small_alt/frame_85.txt"),
    include_str!("../frames/anzoth_small_alt/frame_86.txt"),
    include_str!("../frames/anzoth_small_alt/frame_87.txt"),
    include_str!("../frames/anzoth_small_alt/frame_88.txt"),
    include_str!("../frames/anzoth_small_alt/frame_89.txt"),
    include_str!("../frames/anzoth_small_alt/frame_90.txt"),
    include_str!("../frames/anzoth_small_alt/frame_91.txt"),
];
pub(crate) const ANZOTH_SMALL_ALT_FRAME_WIDTH: u16 = 25;
pub(crate) const ANZOTH_SMALL_ALT_FRAME_HEIGHT: u16 = 13;
pub(crate) const ANZOTH_SMALL_ALT_SEQUENCE: [usize; 186] = [
    91, 91, 90, 89, 88, 87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 69,
    68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45,
    44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21,
    20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 1, 2, 3, 4,
    5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
    54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
    78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
];

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
    fn anzoth_small_alt_frames_are_embedded_and_uniform() {
        assert_eq!(FRAMES_ANZOTH_SMALL_ALT.len(), 92);
        let mut expected_dims: Option<(usize, usize)> = None;
        for frame in FRAMES_ANZOTH_SMALL_ALT {
            let dims = frame_dimensions(frame);
            assert_eq!(
                dims,
                (
                    usize::from(ANZOTH_SMALL_ALT_FRAME_WIDTH),
                    usize::from(ANZOTH_SMALL_ALT_FRAME_HEIGHT)
                )
            );
            match expected_dims {
                Some(expected) => assert_eq!(dims, expected),
                None => expected_dims = Some(dims),
            }
            assert!(!frame.contains('\t'));
            assert!(!frame.contains("\r\n"));
            assert!(!frame.contains('\r'));
        }
    }

    #[test]
    fn anzoth_small_alt_sequence_matches_spec() {
        assert_eq!(ANZOTH_SMALL_ALT_SEQUENCE.len(), 186);
        assert_eq!(&ANZOTH_SMALL_ALT_SEQUENCE[0..2], &[91, 91]);
        assert_eq!(ANZOTH_SMALL_ALT_SEQUENCE[92], 0);
        assert_eq!(&ANZOTH_SMALL_ALT_SEQUENCE[92..96], &[0, 0, 0, 0]);
        assert_eq!(ANZOTH_SMALL_ALT_SEQUENCE[96], 1);
        assert_eq!(ANZOTH_SMALL_ALT_SEQUENCE[185], 90);
    }

    #[test]
    fn anzoth_small_alt_has_non_ascii_glyphs() {
        let frame = FRAMES_ANZOTH_SMALL_ALT[0];
        assert!(
            frame
                .chars()
                .any(|ch| !ch.is_ascii() && ch != '\n' && ch != ' ')
        );
    }

    #[test]
    fn anzoth_is_the_initial_animation_variant() {
        let animation = AsciiAnimation::new(FrameRequester::test_dummy());
        assert_eq!(animation.variant_idx_for_tests(), 0);
    }
}
