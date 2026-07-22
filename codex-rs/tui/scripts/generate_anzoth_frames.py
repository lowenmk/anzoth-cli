#!/usr/bin/env python3
"""Generate deterministic Anzoth welcome animation frames from the approved mask."""

from __future__ import annotations

import math
from collections import defaultdict
from pathlib import Path

FRAME_COUNT = 36
TEXTURE = "anzoth"
ALLOWED_CHARS = set(TEXTURE)
MASTER_WIDTH = 64
MASTER_HEIGHT = 31
FULL_SIZE = (64, 31)
REDUCED_SIZE = (44, 21)
EDGE_SCALE = 0.42
SHEAR_FRACTION = 0.045
DEPTH_OFFSET_FRACTION = 0.022

# Occupied bounding-box widths from the existing Codex frame sequence. The
# values preserve its asymmetric turn cadence; EDGE_SCALE keeps this line-art
# emblem more legible than Codex's six-column edge-on frame.
CODEX_WIDTH_PROFILE = (
    32,
    32,
    32,
    30,
    30,
    28,
    26,
    24,
    22,
    18,
    14,
    10,
    7,
    8,
    13,
    17,
    21,
    26,
    29,
    31,
    31,
    32,
    29,
    26,
    22,
    16,
    11,
    6,
    9,
    15,
    19,
    24,
    26,
    30,
    31,
    32,
)

TUI_DIR = Path(__file__).resolve().parents[1]
MASTER_PATH = TUI_DIR / "assets" / "anzoth-logo-master-64x31.txt"
OUTPUTS = {
    "full": (TUI_DIR / "frames" / "anzoth_full", FULL_SIZE),
    "reduced": (TUI_DIR / "frames" / "anzoth", REDUCED_SIZE),
}


def load_master() -> tuple[str, ...]:
    lines = MASTER_PATH.read_text(encoding="ascii").splitlines()
    if len(lines) != MASTER_HEIGHT:
        raise ValueError(
            f"master height mismatch: {len(lines)} != {MASTER_HEIGHT}"
        )
    if any(len(line) > MASTER_WIDTH for line in lines):
        raise ValueError(f"master exceeds {MASTER_WIDTH} columns")
    if any(set(line) - {"#", " "} for line in lines):
        raise ValueError("master contains characters other than '#' and space")
    return tuple(line.ljust(MASTER_WIDTH) for line in lines)


def occupied_cells(lines: tuple[str, ...] | list[str]) -> set[tuple[int, int]]:
    return {
        (x, y)
        for y, line in enumerate(lines)
        for x, char in enumerate(line)
        if char != " "
    }


def connected_components(cells: set[tuple[int, int]]) -> list[set[tuple[int, int]]]:
    remaining = set(cells)
    components: list[set[tuple[int, int]]] = []
    while remaining:
        seed = remaining.pop()
        component = {seed}
        pending = [seed]
        while pending:
            x, y = pending.pop()
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    neighbor = (x + dx, y + dy)
                    if (dx or dy) and neighbor in remaining:
                        remaining.remove(neighbor)
                        component.add(neighbor)
                        pending.append(neighbor)
        components.append(component)
    return sorted(components, key=len, reverse=True)


def phase_for(frame_index: int) -> float:
    # Match Codex's edge/front landmarks instead of assuming constant angular
    # speed: front=1, first edge=13, back=22, second edge=28, loop=37.
    if frame_index <= 12:
        return (frame_index / 12.0) * math.pi / 2.0
    if frame_index <= 21:
        return math.pi / 2.0 + ((frame_index - 12) / 9.0) * math.pi / 2.0
    if frame_index <= 27:
        return math.pi + ((frame_index - 21) / 6.0) * math.pi / 2.0
    return 3.0 * math.pi / 2.0 + ((frame_index - 27) / 9.0) * math.pi / 2.0


def compression_for(frame_index: int) -> float:
    minimum = min(CODEX_WIDTH_PROFILE)
    maximum = max(CODEX_WIDTH_PROFILE)
    normalized = (CODEX_WIDTH_PROFILE[frame_index] - minimum) / (maximum - minimum)
    return EDGE_SCALE + (1.0 - EDGE_SCALE) * normalized


def project_cell(
    x: int,
    y: int,
    width: int,
    height: int,
    phase: float,
    compression: float,
) -> tuple[int, int, float]:
    base_x = x * (width - 1) / (MASTER_WIDTH - 1)
    base_y = y * (height - 1) / (MASTER_HEIGHT - 1)
    center_x = (width - 1) / 2.0
    center_y = (height - 1) / 2.0
    half_width = max(center_x, 1.0)
    half_height = max(center_y, 1.0)

    sine = math.sin(phase)
    facing = 1.0 if math.cos(phase) >= 0.0 else -1.0
    local_x = base_x - center_x
    local_y = base_y - center_y

    shear = SHEAR_FRACTION * sine * half_width * (local_y / half_height)
    depth_offset = DEPTH_OFFSET_FRACTION * width * sine
    projected_x = center_x + facing * compression * local_x + shear + depth_offset

    # Source x determines front-to-back ordering when compressed cells collide.
    depth = sine * (x - (MASTER_WIDTH - 1) / 2.0)
    return round(projected_x), round(base_y), depth


def texture_char(source_x: int, source_y: int) -> str:
    # The texture is fixed in source coordinates, so it deforms with the surface.
    index = source_x + 2 * source_y
    return TEXTURE[index % len(TEXTURE)]


def render_frame(
    master: tuple[str, ...],
    frame_index: int,
    size: tuple[int, int],
) -> list[str]:
    width, height = size
    phase = phase_for(frame_index)
    compression = compression_for(frame_index)
    candidates: dict[tuple[int, int], list[tuple[float, int, int]]] = defaultdict(list)

    for source_y, line in enumerate(master):
        for source_x, char in enumerate(line):
            if char == " ":
                continue
            target_x, target_y, depth = project_cell(
                source_x,
                source_y,
                width,
                height,
                phase,
                compression,
            )
            if 0 <= target_x < width and 0 <= target_y < height:
                candidates[(target_x, target_y)].append(
                    (depth, source_x, source_y)
                )

    rows = [[" "] * width for _ in range(height)]
    for (target_x, target_y), samples in candidates.items():
        _, source_x, source_y = max(samples)
        rows[target_y][target_x] = texture_char(source_x, source_y)
    return ["".join(row) for row in rows]


def frame_dimensions(frame: list[str]) -> tuple[int, int]:
    return max((len(line) for line in frame), default=0), len(frame)


def occupancy_iou(left: list[str], right: list[str]) -> float:
    left_cells = occupied_cells(left)
    right_cells = occupied_cells(right)
    union = left_cells | right_cells
    return len(left_cells & right_cells) / len(union) if union else 1.0


def cell_agreement(left: list[str], right: list[str]) -> float:
    same = sum(
        (left_char != " ") == (right_char != " ")
        for left_line, right_line in zip(left, right)
        for left_char, right_char in zip(left_line, right_line)
    )
    total = len(left) * len(left[0])
    return same / total


def validate_sequence(
    name: str,
    frames: list[list[str]],
    size: tuple[int, int],
) -> dict[str, float | int]:
    if len(frames) != FRAME_COUNT:
        raise ValueError(f"{name}: expected {FRAME_COUNT} frames")
    if any(frame_dimensions(frame) != size for frame in frames):
        raise ValueError(f"{name}: frame dimensions are not uniformly {size}")

    front_occupied = len(occupied_cells(frames[0]))
    occupied_counts: list[int] = []
    for index, frame in enumerate(frames, start=1):
        glyphs = {char for line in frame for char in line if char != " "}
        if not glyphs or not glyphs <= ALLOWED_CHARS:
            raise ValueError(f"{name} frame {index}: unsupported glyphs {glyphs}")
        if len(glyphs) < 3:
            raise ValueError(f"{name} frame {index}: insufficient texture variety")

        occupied = occupied_cells(frame)
        occupied_counts.append(len(occupied))
        components = connected_components(occupied)
        if not components or len(components[0]) < len(occupied) * 0.80:
            raise ValueError(
                f"{name} frame {index}: logo fragmented into disconnected clusters"
            )
        if len(occupied) < front_occupied * 0.30:
            raise ValueError(f"{name} frame {index}: logo is too sparse")

    loop_iou = occupancy_iou(frames[-1], frames[0])
    loop_agreement = cell_agreement(frames[-1], frames[0])
    if loop_iou < 0.60:
        raise ValueError(f"{name}: loop IoU is too low: {loop_iou:.3f}")

    adjacent_ious = [
        occupancy_iou(frames[index], frames[(index + 1) % FRAME_COUNT])
        for index in range(FRAME_COUNT)
    ]
    return {
        "min_occupied": min(occupied_counts),
        "max_occupied": max(occupied_counts),
        "loop_iou": loop_iou,
        "loop_agreement": loop_agreement,
        "min_adjacent_iou": min(adjacent_ious),
        "mean_adjacent_iou": sum(adjacent_ious) / len(adjacent_ious),
    }


def write_sequence(out_dir: Path, frames: list[list[str]]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    for index, frame in enumerate(frames, start=1):
        path = out_dir / f"frame_{index}.txt"
        path.write_text("\n".join(frame) + "\n", encoding="ascii", newline="\n")


def main() -> int:
    master = load_master()
    master_components = connected_components(occupied_cells(master))
    if len(master_components) != 3:
        raise ValueError(
            f"approved master component count changed: {len(master_components)} != 3"
        )

    for name, (out_dir, size) in OUTPUTS.items():
        frames = [render_frame(master, index, size) for index in range(FRAME_COUNT)]
        metrics = validate_sequence(name, frames, size)
        write_sequence(out_dir, frames)
        print(
            f"{name}: {FRAME_COUNT} frames at {size[0]}x{size[1]}, "
            f"occupied={metrics['min_occupied']}-{metrics['max_occupied']}, "
            f"loop_iou={metrics['loop_iou']:.3f}, "
            f"loop_agreement={metrics['loop_agreement']:.3f}, "
            f"adjacent_iou={metrics['min_adjacent_iou']:.3f}-"
            f"{metrics['mean_adjacent_iou']:.3f}"
        )
        print(f"wrote {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
