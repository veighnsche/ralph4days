#!/usr/bin/env python3
"""Detect face bounding boxes in discipline images using YOLO pose detection.

For each stack's discipline, finds the latest generated image, runs YOLO pose
detection to locate the face/head region, and updates the `face_box:` field
in the corresponding YAML file.

Uses yolov8x-pose.pt for best accuracy on stylized art (auto-downloads on first run).

Dependencies: ultralytics, numpy, Pillow

Usage:
    python crates/predefined-disciplines/detect_faces.py [--dry-run]
"""

import re
import sys
from pathlib import Path

import numpy as np
from ultralytics import YOLO

DISCIPLINES_DIR = Path(__file__).parent / "src" / "defaults" / "disciplines"

STACKS = {
    "01_generic": list(range(8)),
    "02_desktop": list(range(8)),
    "03_saas": list(range(8)),
    "04_mobile": list(range(8)),
}

IMAGE_RE = re.compile(r"^(\d{2})_(.+?)_\d+_\d+x\d+_([a-z0-9]+)\.png$")

MODEL = None


def get_model():
    global MODEL
    if MODEL is None:
        MODEL = YOLO("yolov8x-pose.pt")
    return MODEL


def find_latest_image(images_dir: Path, disc_index: int) -> Path | None:
    prefix = f"{disc_index:02d}_"
    candidates = []
    for f in images_dir.iterdir():
        if not f.name.startswith(prefix) or f.suffix != ".png":
            continue
        m = IMAGE_RE.match(f.name)
        if m:
            candidates.append((m.group(3), f))
    if not candidates:
        return None
    candidates.sort(key=lambda x: x[0])
    return candidates[-1][1]


def detect_face_box(img_path: Path) -> dict | None:
    """Detect face bounding box from YOLO pose keypoints.

    Keypoints: 0=nose, 1=left_eye, 2=right_eye, 3=left_ear, 4=right_ear

    Returns {x, y, w, h} normalized 0.0-1.0, or None if detection fails.
    """
    from PIL import Image

    model = get_model()
    results = model(str(img_path), verbose=False)

    if not results or len(results[0].keypoints) == 0:
        return None

    kpts = results[0].keypoints.xy[0].cpu().numpy()
    confs = results[0].keypoints.conf[0].cpu().numpy() if results[0].keypoints.conf is not None else None

    img = Image.open(img_path)
    w, h = img.size

    nose = kpts[0] if (confs is None or confs[0] > 0.3) and kpts[0][1] > 0 else None
    left_eye = kpts[1] if (confs is None or confs[1] > 0.3) and kpts[1][1] > 0 else None
    right_eye = kpts[2] if (confs is None or confs[2] > 0.3) and kpts[2][1] > 0 else None
    left_ear = kpts[3] if (confs is None or confs[3] > 0.3) and kpts[3][1] > 0 else None
    right_ear = kpts[4] if (confs is None or confs[4] > 0.3) and kpts[4][1] > 0 else None

    # Face center from eyes, fallback to nose
    eye_points = [p for p in [left_eye, right_eye] if p is not None]
    if len(eye_points) >= 2:
        center_x = np.mean([p[0] for p in eye_points])
        center_y = np.mean([p[1] for p in eye_points])
    elif len(eye_points) == 1:
        center_x, center_y = eye_points[0]
    elif nose is not None:
        center_x, center_y = nose
    else:
        return None

    # Face width from ears, fallback to eye-to-nose distance
    ear_points = [p for p in [left_ear, right_ear] if p is not None]
    if len(ear_points) == 2:
        face_width = abs(ear_points[1][0] - ear_points[0][0])
    elif len(eye_points) == 2 and nose is not None:
        eye_dist = abs(eye_points[1][0] - eye_points[0][0])
        face_width = eye_dist * 2.5
    elif len(eye_points) == 2:
        face_width = abs(eye_points[1][0] - eye_points[0][0]) * 3.0
    else:
        face_width = w * 0.25

    # Padding to include hair/forehead
    padding = 1.5
    box_size = face_width * padding
    face_height = box_size

    # Top-left corner
    x = center_x - box_size / 2
    y = center_y - face_height * 0.6  # bias upward to include forehead

    # Clamp to image bounds
    x = max(0, min(x, w - box_size))
    y = max(0, min(y, h - face_height))
    box_w = min(box_size, w - x)
    box_h = min(face_height, h - y)

    return {
        "x": round(x / w, 4),
        "y": round(y / h, 4),
        "w": round(box_w / w, 4),
        "h": round(box_h / h, 4),
    }


def fallback_face_box() -> dict:
    """Top 25% of image as face region when YOLO fails."""
    return {"x": 0.25, "y": 0.0, "w": 0.50, "h": 0.25}


def find_yaml_for_discipline(stack_dir: Path, disc_index: int) -> Path | None:
    prefix = f"{disc_index:02d}_"
    for f in stack_dir.iterdir():
        if f.name.startswith(prefix) and f.suffix == ".yaml":
            return f
    return None


def update_yaml_face_box(yaml_path: Path, face_box: dict, dry_run: bool) -> str:
    """Update or insert face_box field in YAML. Returns 'inserted', 'updated', or 'unchanged'."""
    text = yaml_path.read_text()

    face_box_yaml = (
        f"face_box:\n"
        f"  x: {face_box['x']}\n"
        f"  y: {face_box['y']}\n"
        f"  w: {face_box['w']}\n"
        f"  h: {face_box['h']}\n"
    )

    # Check if face_box already exists
    pattern = re.compile(r'^face_box:\n(?:  [a-z]: [\d.]+\n){1,4}', re.MULTILINE)
    m = pattern.search(text)

    if m:
        old = m.group(0)
        if old == face_box_yaml:
            return "unchanged"
        new_text = text[:m.start()] + face_box_yaml + text[m.end():]
        if not dry_run:
            yaml_path.write_text(new_text)
        return "updated"
    else:
        # Append at end of file
        if not text.endswith("\n"):
            text += "\n"
        new_text = text + "\n" + face_box_yaml
        if not dry_run:
            yaml_path.write_text(new_text)
        return "inserted"


def main():
    dry_run = "--dry-run" in sys.argv

    if dry_run:
        print("DRY RUN — no files will be modified\n")

    changes = 0
    skips = 0
    errors = 0

    for stack_name, disc_indices in STACKS.items():
        stack_dir = DISCIPLINES_DIR / stack_name
        images_dir = stack_dir / "images"

        if not images_dir.exists():
            print(f"SKIP {stack_name}: no images/ directory")
            skips += len(disc_indices)
            continue

        print(f"\n=== {stack_name} ===")

        for idx in disc_indices:
            img_path = find_latest_image(images_dir, idx)
            if img_path is None:
                print(f"  {idx:02d}: NO IMAGE FOUND")
                errors += 1
                continue

            yaml_path = find_yaml_for_discipline(stack_dir, idx)
            if yaml_path is None:
                print(f"  {idx:02d}: NO YAML FOUND")
                errors += 1
                continue

            face_box = detect_face_box(img_path)
            if face_box is None:
                print(f"  {idx:02d}: YOLO failed, using fallback")
                face_box = fallback_face_box()

            result = update_yaml_face_box(yaml_path, face_box, dry_run)

            box_str = f"x={face_box['x']:.3f} y={face_box['y']:.3f} w={face_box['w']:.3f} h={face_box['h']:.3f}"

            if result == "unchanged":
                print(f"  {idx:02d}: {box_str} (unchanged) ← {img_path.name}")
                skips += 1
            else:
                action = f"WOULD {result.upper()}" if dry_run else result.upper()
                print(f"  {idx:02d}: {box_str} ({action}) ← {img_path.name}")
                changes += 1

    print(f"\nDone: {changes} updated, {skips} unchanged, {errors} errors")
    if errors > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
