#!/usr/bin/env python3
"""Compose 8 discipline portraits into a single 1920x1080 image per stack.

Uses YOLO pose detection to find actual eye and ankle positions in each
source image, then scales and crops each character so eyelines and feet
align across the composite.

Usage:
    python compose_stack.py              # all stacks
    python compose_stack.py 01           # single stack
    python compose_stack.py 01 03        # multiple stacks
"""
from PIL import Image, ImageDraw
from pathlib import Path
from ultralytics import YOLO
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

MODEL = None

def get_model():
    global MODEL
    if MODEL is None:
        MODEL = YOLO("yolov8n-pose.pt")
    return MODEL


def detect_pose(img_path):
    """Use YOLO pose to find eye and ankle positions.

    Returns (eye_y_pct, feet_y_pct, center_x_pct) or None if detection fails.
    Keypoint indices: 0=nose, 1=left_eye, 2=right_eye, 15=left_ankle, 16=right_ankle
    """
    model = get_model()
    results = model(str(img_path), verbose=False)

    if not results or len(results[0].keypoints) == 0:
        return None

    kpts = results[0].keypoints.xy[0].cpu().numpy()
    confs = results[0].keypoints.conf[0].cpu().numpy() if results[0].keypoints.conf is not None else None

    img = Image.open(img_path)
    w, h = img.size

    left_eye = kpts[1] if confs is None or confs[1] > 0.3 else None
    right_eye = kpts[2] if confs is None or confs[2] > 0.3 else None
    nose = kpts[0] if confs is None or confs[0] > 0.3 else None
    left_ankle = kpts[15] if confs is None or confs[15] > 0.3 else None
    right_ankle = kpts[16] if confs is None or confs[16] > 0.3 else None

    eye_points = [p for p in [left_eye, right_eye] if p is not None and p[1] > 0]
    if not eye_points:
        if nose is not None and nose[1] > 0:
            eye_y = nose[1] - (h * 0.02)
            eye_x = nose[0]
        else:
            return None
    else:
        eye_y = np.mean([p[1] for p in eye_points])
        eye_x = np.mean([p[0] for p in eye_points])

    ankle_points = [p for p in [left_ankle, right_ankle] if p is not None and p[1] > 0]
    if ankle_points:
        feet_y = np.max([p[1] for p in ankle_points])
    else:
        bbox = results[0].boxes.xyxy[0].cpu().numpy() if len(results[0].boxes) > 0 else None
        if bbox is not None:
            feet_y = bbox[3]
        else:
            return None

    all_x = [p[0] for p in [left_eye, right_eye, nose] if p is not None and p[0] > 0]
    if len(all_x) > 0:
        center_x = np.mean(all_x)
    else:
        center_x = w / 2

    return eye_y / h, feet_y / h, center_x / w


def fallback_detect(img_path):
    """Variance-based fallback when YOLO can't find a person."""
    img = Image.open(img_path).convert("RGB")
    arr = np.array(img)
    h, w, _ = arr.shape

    mid_strip = arr[h // 4 : 3 * h // 4, :, :]
    col_variance = np.var(mid_strip.astype(float), axis=(0, 2))
    threshold = np.median(col_variance) * 0.5
    active_cols = np.where(col_variance > threshold)[0]
    cx = (active_cols[0] + active_cols[-1]) / 2 / w if len(active_cols) > w * 0.1 else 0.5

    return 0.15, 0.90, cx


def compose_stack(stack_num, stack_slug):
    stack_dir = BASE / f"{stack_num:02d}_{stack_slug}" / "images"
    if not stack_dir.exists():
        print(f"  Skipping: no images/ dir")
        return

    pngs = sorted(stack_dir.glob("*.png"))
    by_discipline = {}
    for p in pngs:
        if "_composite" in p.name or "_debug" in p.name:
            continue
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

        pose = detect_pose(img_path)
        if pose:
            eye_pct, feet_pct, cx_pct = pose
            method = "pose"
        else:
            eye_pct, feet_pct, cx_pct = fallback_detect(img_path)
            method = "fallback"

        body_pct = feet_pct - eye_pct
        if body_pct < 0.1:
            body_pct = 0.75
            eye_pct = 0.15

        body_px_orig = body_pct * orig_h
        scale = target_body_px / body_px_orig

        new_w = int(orig_w * scale)
        new_h = int(orig_h * scale)

        if new_w < SLICE_W:
            scale = (SLICE_W * 1.1) / orig_w
            new_w = int(orig_w * scale)
            new_h = int(orig_h * scale)

        img = img.resize((new_w, new_h), Image.LANCZOS)

        scaled_eye_y = int(eye_pct * new_h)
        top = scaled_eye_y - target_eye_px

        scaled_cx = int(cx_pct * new_w)
        left = scaled_cx - SLICE_W // 2
        left = max(0, min(left, max(0, new_w - SLICE_W)))

        if top < 0:
            top = 0
        if top + HEIGHT > new_h:
            top = max(0, new_h - HEIGHT)

        crop_bottom = min(top + HEIGHT, new_h)
        crop = img.crop((left, top, left + SLICE_W, crop_bottom))

        if crop.size[1] < HEIGHT:
            padded = Image.new("RGB", (SLICE_W, HEIGHT))
            padded.paste(crop, (0, 0))
            crop = padded

        composite.paste(crop, (i * SLICE_W, 0))

        actual_eye_in_composite = scaled_eye_y - top
        actual_eye_pct = actual_eye_in_composite / HEIGHT
        print(f"    [{method}] {img_path.stem}: eye={eye_pct:.0%} feet={feet_pct:.0%} cx={cx_pct:.0%} scale={scale:.2f} eyeline@{actual_eye_pct:.0%}")

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
