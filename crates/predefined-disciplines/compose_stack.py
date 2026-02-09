#!/usr/bin/env python3
"""Compose 8 discipline portraits into a single 1920x1080 image per stack.

Source images SHOULD have consistent framing (eyes ~15%, feet ~90%) but
we verify this by detecting actual character bounds via edge/content analysis.
Falls back to center-crop if detection fails.

Usage:
    python compose_stack.py              # all stacks
    python compose_stack.py 01           # single stack
    python compose_stack.py 01 03        # multiple stacks
"""
from PIL import Image, ImageDraw
from pathlib import Path
import numpy as np
import sys

STACKS = {
    1: "generic",
    2: "desktop",
    3: "saas",
    4: "mobile",
}

BASE = Path(__file__).parent / "src" / "defaults" / "disciplines"
OUT = BASE / "composites"
OUT.mkdir(exist_ok=True)

WIDTH, HEIGHT = 1920, 1080
SLICES = 8
SLICE_W = WIDTH // SLICES
TARGET_EYE_Y = 0.15
TARGET_FEET_Y = 0.90


def find_character_bounds(img_path):
    """Find where the character starts (top) and ends (bottom) in the image.

    Uses edge detection on the center column strip to find the topmost and
    bottommost content that isn't background. Returns (top_pct, bottom_pct)
    as fractions of image height, or None if detection fails.
    """
    img = Image.open(img_path).convert("RGB")
    arr = np.array(img)
    h, w, _ = arr.shape

    center_strip = arr[:, w // 4 : 3 * w // 4, :]
    row_variance = np.var(center_strip.astype(float), axis=(1, 2))

    threshold = np.median(row_variance) * 0.3
    active_rows = np.where(row_variance > threshold)[0]

    if len(active_rows) < h * 0.2:
        return None

    top_row = active_rows[0]
    bottom_row = active_rows[-1]

    top_pct = top_row / h
    bottom_pct = bottom_row / h

    if bottom_pct - top_pct < 0.3:
        return None

    head_offset = (bottom_pct - top_pct) * 0.08
    eye_pct = top_pct + head_offset
    feet_pct = bottom_pct

    return eye_pct, feet_pct


def find_character_center_x(img_path):
    """Find horizontal center of the character using column variance."""
    img = Image.open(img_path).convert("RGB")
    arr = np.array(img)
    h, w, _ = arr.shape

    mid_strip = arr[h // 4 : 3 * h // 4, :, :]
    col_variance = np.var(mid_strip.astype(float), axis=(0, 2))

    threshold = np.median(col_variance) * 0.3
    active_cols = np.where(col_variance > threshold)[0]

    if len(active_cols) < w * 0.1:
        return 0.5

    return (active_cols[0] + active_cols[-1]) / 2 / w


def compose_stack(stack_num, stack_slug):
    stack_dir = BASE / f"{stack_num:02d}_{stack_slug}" / "images"
    if not stack_dir.exists():
        print(f"  Skipping: no images/ dir")
        return

    pngs = sorted(stack_dir.glob("*.png"))
    by_discipline = {}
    for p in pngs:
        by_discipline[p.name[:2]] = p

    if len(by_discipline) < SLICES:
        print(f"  Skipping: only {len(by_discipline)}/{SLICES} disciplines")
        return

    ordered = [by_discipline[f"{i:02d}"] for i in range(SLICES)]
    composite = Image.new("RGB", (WIDTH, HEIGHT))

    target_eye_px = int(HEIGHT * TARGET_EYE_Y)
    target_feet_px = int(HEIGHT * TARGET_FEET_Y)
    target_body_px = target_feet_px - target_eye_px

    for i, img_path in enumerate(ordered):
        img = Image.open(img_path)
        orig_w, orig_h = img.size

        bounds = find_character_bounds(img_path)
        cx_pct = find_character_center_x(img_path)

        if bounds:
            eye_pct, feet_pct = bounds
            body_pct = feet_pct - eye_pct
            body_px_in_orig = body_pct * orig_h
            scale = target_body_px / body_px_in_orig
            method = "detect"
        else:
            eye_pct = TARGET_EYE_Y
            scale = HEIGHT / orig_h
            method = "fallback"

        new_w = int(orig_w * scale)
        new_h = int(orig_h * scale)

        if new_w < SLICE_W:
            scale = (SLICE_W * 1.2) / orig_w
            new_w = int(orig_w * scale)
            new_h = int(orig_h * scale)

        img = img.resize((new_w, new_h), Image.LANCZOS)

        scaled_eye_y = int(eye_pct * new_h)
        top = scaled_eye_y - target_eye_px
        top = max(0, min(top, new_h - HEIGHT))

        scaled_cx = int(cx_pct * new_w)
        left = scaled_cx - SLICE_W // 2
        left = max(0, min(left, new_w - SLICE_W))

        crop = img.crop((left, top, left + SLICE_W, top + HEIGHT))

        if crop.size != (SLICE_W, HEIGHT):
            padded = Image.new("RGB", (SLICE_W, HEIGHT))
            padded.paste(crop, (0, 0))
            crop = padded

        composite.paste(crop, (i * SLICE_W, 0))
        print(f"    [{method}] {img_path.stem}: eye={eye_pct:.0%} cx={cx_pct:.0%} scale={scale:.2f}")

    out_path = OUT / f"{stack_num:02d}_{stack_slug}_composite.png"
    composite.save(out_path, quality=95)
    print(f"  Saved: {out_path} ({out_path.stat().st_size // 1024}KB)")

    debug = composite.copy()
    draw = ImageDraw.Draw(debug)
    draw.line([(0, target_eye_px), (WIDTH, target_eye_px)], fill="red", width=3)
    draw.line([(0, target_feet_px), (WIDTH, target_feet_px)], fill="blue", width=3)
    debug_path = OUT / f"{stack_num:02d}_{stack_slug}_debug.png"
    debug.save(debug_path, quality=95)
    print(f"  Debug: {debug_path}\n")


requested = sys.argv[1:]
if requested:
    stack_nums = [int(s) for s in requested]
else:
    stack_nums = list(STACKS.keys())

for num in stack_nums:
    slug = STACKS.get(num)
    if slug:
        print(f"Stack {num:02d}_{slug}:")
        compose_stack(num, slug)
    else:
        print(f"Unknown stack: {num}")

print("Done!")
