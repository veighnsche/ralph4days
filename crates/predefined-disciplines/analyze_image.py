#!/usr/bin/env python3
"""Analyze discipline images: detect pose, compute crop regions, write sidecar YAML.

Replaces detect_faces.py with a comprehensive analysis pipeline that persists
all detection data as a sidecar YAML per image and updates the discipline YAML
with pre-computed crop boxes.

Dependencies: ultralytics, numpy, Pillow, pyyaml

Usage:
    python analyze_image.py                    # all stacks
    python analyze_image.py 02                 # single stack
    python analyze_image.py 02 00              # single discipline
    python analyze_image.py --dry-run
"""

import re
import sys
from pathlib import Path

import numpy as np
import yaml
from PIL import Image
from ultralytics import YOLO

DISCIPLINES_DIR = Path(__file__).parent / "src" / "defaults" / "disciplines"

STACKS = {
    "01_generic": list(range(8)),
    "02_desktop": list(range(8)),
    "03_saas": list(range(8)),
    "04_mobile": list(range(8)),
}

IMAGE_RE = re.compile(r"^(\d{2})_(.+?)_(\d+)_(\d+)x(\d+)_([a-z0-9]+)\.png$")

KEYPOINT_NAMES = [
    "nose", "left_eye", "right_eye", "left_ear", "right_ear",
    "left_shoulder", "right_shoulder", "left_elbow", "right_elbow",
    "left_wrist", "right_wrist", "left_hip", "right_hip",
    "left_knee", "right_knee", "left_ankle", "right_ankle",
]

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
            candidates.append((m.group(6), f))
    if not candidates:
        return None
    candidates.sort(key=lambda x: x[0])
    return candidates[-1][1]


def find_yaml_for_discipline(stack_dir: Path, disc_index: int) -> Path | None:
    prefix = f"{disc_index:02d}_"
    for f in stack_dir.iterdir():
        if f.name.startswith(prefix) and f.suffix == ".yaml":
            return f
    return None


def analyze_image(img_path: Path) -> dict:
    """Run full analysis on one image. Returns sidecar data dict."""
    img = Image.open(img_path)
    w, h = img.size

    result = {
        "source": img_path.name,
        "dimensions": {"w": w, "h": h},
    }

    model = get_model()
    detections = model(str(img_path), verbose=False)

    if not detections or len(detections[0].keypoints) == 0:
        result["detection"] = {"method": "fallback", "confidence": 0.0}
        result["pose"] = {"eyeline_y": 0.15, "ankle_y": 0.90, "center_x": 0.50, "body_height": 0.75}
        result["boxes"] = _fallback_boxes()
        result["crops"] = _compute_crops(result["pose"], result["boxes"], w, h)
        return result

    kpts = detections[0].keypoints.xy[0].cpu().numpy()
    confs = detections[0].keypoints.conf[0].cpu().numpy() if detections[0].keypoints.conf is not None else np.ones(17)

    # Best detection confidence
    det_conf = float(detections[0].boxes.conf[0].cpu().numpy()) if len(detections[0].boxes) > 0 else 0.0

    # Extract valid keypoints
    keypoints = {}
    for i, name in enumerate(KEYPOINT_NAMES):
        if i < len(kpts) and confs[i] > 0.3 and kpts[i][1] > 0:
            keypoints[name] = {
                "x": round(float(kpts[i][0]) / w, 4),
                "y": round(float(kpts[i][1]) / h, 4),
                "conf": round(float(confs[i]), 3),
            }

    result["detection"] = {
        "model": "yolov8x-pose",
        "confidence": round(det_conf, 3),
        "method": "pose",
        "keypoints": keypoints,
    }

    # Compute pose metrics
    pose = _compute_pose(keypoints)
    result["pose"] = pose

    # Compute bounding boxes
    boxes = _compute_boxes(keypoints, pose)
    result["boxes"] = boxes

    # Compute crop regions
    result["crops"] = _compute_crops(pose, boxes, w, h)

    return result


def _compute_pose(kpts: dict) -> dict:
    """Derive pose metrics from keypoints."""
    # Eyeline from eyes, fallback to nose
    eye_ys = [kpts[k]["y"] for k in ("left_eye", "right_eye") if k in kpts]
    if eye_ys:
        eyeline_y = sum(eye_ys) / len(eye_ys)
    elif "nose" in kpts:
        eyeline_y = kpts["nose"]["y"]
    else:
        eyeline_y = 0.15

    # Ankle line
    ankle_ys = [kpts[k]["y"] for k in ("left_ankle", "right_ankle") if k in kpts]
    ankle_y = max(ankle_ys) if ankle_ys else 0.90

    # Center X from shoulders, hips, or face
    center_xs = []
    for pair in [("left_shoulder", "right_shoulder"), ("left_hip", "right_hip")]:
        if pair[0] in kpts and pair[1] in kpts:
            center_xs.append((kpts[pair[0]]["x"] + kpts[pair[1]]["x"]) / 2)
    if not center_xs:
        face_xs = [kpts[k]["x"] for k in ("nose", "left_eye", "right_eye") if k in kpts]
        if face_xs:
            center_xs.append(sum(face_xs) / len(face_xs))
    center_x = sum(center_xs) / len(center_xs) if center_xs else 0.50

    body_height = round(ankle_y - eyeline_y, 4)

    return {
        "eyeline_y": round(eyeline_y, 4),
        "ankle_y": round(ankle_y, 4),
        "center_x": round(center_x, 4),
        "body_height": max(body_height, 0.3),
    }


def _compute_boxes(kpts: dict, pose: dict) -> dict:
    """Compute face and body bounding boxes."""
    # Face box from ears/eyes
    face_pts_x = [kpts[k]["x"] for k in ("left_ear", "right_ear", "left_eye", "right_eye", "nose") if k in kpts]
    face_pts_y = [kpts[k]["y"] for k in ("left_ear", "right_ear", "left_eye", "right_eye", "nose") if k in kpts]

    if face_pts_x and face_pts_y:
        face_cx = sum(face_pts_x) / len(face_pts_x)
        face_cy = sum(face_pts_y) / len(face_pts_y)
        face_spread_x = max(face_pts_x) - min(face_pts_x)
        face_spread_y = max(face_pts_y) - min(face_pts_y)
        face_size = max(face_spread_x, face_spread_y) * 1.8
        face_size = max(face_size, 0.10)
        face = {
            "x": round(max(0, face_cx - face_size / 2), 4),
            "y": round(max(0, face_cy - face_size * 0.6), 4),
            "w": round(min(face_size, 1.0), 4),
            "h": round(min(face_size, 1.0), 4),
        }
    else:
        face = {"x": 0.25, "y": 0.0, "w": 0.50, "h": 0.25}

    # Body box
    all_xs = [kpts[k]["x"] for k in kpts]
    all_ys = [kpts[k]["y"] for k in kpts]
    if all_xs:
        body_x = max(0, min(all_xs) - 0.05)
        body_w = min(1.0, max(all_xs) - min(all_xs) + 0.10)
        body_y = max(0, min(all_ys) - 0.04)
        body_h = min(1.0, max(all_ys) - min(all_ys) + 0.08)
        body = {"x": round(body_x, 4), "y": round(body_y, 4), "w": round(body_w, 4), "h": round(body_h, 4)}
    else:
        body = {"x": 0.15, "y": 0.04, "w": 0.70, "h": 0.86}

    return {"face": face, "body": body}


def _fallback_boxes() -> dict:
    return {
        "face": {"x": 0.25, "y": 0.0, "w": 0.50, "h": 0.25},
        "body": {"x": 0.15, "y": 0.04, "w": 0.70, "h": 0.86},
    }


def _compute_crops(pose: dict, boxes: dict, img_w: int, img_h: int) -> dict:
    """Pre-compute crop regions for UI aspect ratios."""
    face = boxes["face"]
    cx = pose["center_x"]

    # face (1:1 square centered on face)
    face_center_x = face["x"] + face["w"] / 2
    face_center_y = face["y"] + face["h"] / 2
    face_size = max(face["w"], face["h"]) * 1.2
    face_crop = _clamp_box(
        face_center_x - face_size / 2,
        face_center_y - face_size * 0.45,
        face_size,
        face_size,
    )

    # card (~5:4, head+shoulders)
    card_h = 0.40
    card_w = 0.80
    card_crop = _clamp_box(cx - card_w / 2, max(0, face["y"] - 0.02), card_w, card_h)

    # portrait (2:3, head to waist)
    portrait_h = 0.53
    portrait_w = 0.80
    portrait_crop = _clamp_box(cx - portrait_w / 2, 0.0, portrait_w, portrait_h)

    # landscape (3:1 banner)
    landscape_h = 0.35
    landscape_crop = _clamp_box(0.0, max(0, pose["eyeline_y"] - 0.10), 1.0, landscape_h)

    # strip (1:4, center third)
    strip_crop = _clamp_box(0.25, 0.0, 0.50, 1.0)

    return {
        "face": face_crop,
        "card": card_crop,
        "portrait": portrait_crop,
        "landscape": landscape_crop,
        "strip": strip_crop,
    }


def _clamp_box(x: float, y: float, w: float, h: float) -> dict:
    x = max(0.0, min(x, 1.0 - w))
    y = max(0.0, min(y, 1.0 - h))
    w = min(w, 1.0 - x)
    h = min(h, 1.0 - y)
    return {"x": round(x, 4), "y": round(y, 4), "w": round(w, 4), "h": round(h, 4)}


BOX_COLORS = {
    "face box":   (255,  50,  50),   # red
    "body box":   ( 50,  50, 255),   # blue
    "crop:face":  (255, 120, 120),   # light red
    "crop:card":  (255, 200,   0),   # yellow
    "crop:portrait": ( 50, 220,  50), # green
    "crop:landscape": (  0, 200, 255), # cyan
    "crop:strip": (200,  50, 255),   # purple
}

KEYPOINT_COLOR = (255, 255, 255)


def write_debug_image(img_path: Path, data: dict, dry_run: bool):
    """Write a _dev.png with all boxes, crops, and keypoints drawn on the image."""
    from PIL import ImageDraw, ImageFont

    dev_path = img_path.with_name(img_path.stem + "_dev.png")
    if dry_run:
        print(f"    WOULD WRITE debug: {dev_path.name}")
        return

    img = Image.open(img_path).convert("RGB")
    w, h = img.size
    draw = ImageDraw.Draw(img, "RGBA")

    def norm_rect(box: dict) -> tuple:
        return (
            int(box["x"] * w),
            int(box["y"] * h),
            int((box["x"] + box["w"]) * w),
            int((box["y"] + box["h"]) * h),
        )

    # Draw crops (filled semi-transparent + outline)
    crop_map = {
        "crop:strip": data["crops"].get("strip"),
        "crop:landscape": data["crops"].get("landscape"),
        "crop:portrait": data["crops"].get("portrait"),
        "crop:card": data["crops"].get("card"),
        "crop:face": data["crops"].get("face"),
    }
    for label, box in crop_map.items():
        if box is None:
            continue
        r, g, b = BOX_COLORS[label]
        rect = norm_rect(box)
        draw.rectangle(rect, outline=(r, g, b, 200), width=2, fill=(r, g, b, 25))

    # Draw detection boxes (thicker, no fill)
    box_map = {
        "body box": data["boxes"].get("body"),
        "face box": data["boxes"].get("face"),
    }
    for label, box in box_map.items():
        if box is None:
            continue
        r, g, b = BOX_COLORS[label]
        rect = norm_rect(box)
        draw.rectangle(rect, outline=(r, g, b, 255), width=3)

    # Draw keypoints
    kpts = data.get("detection", {}).get("keypoints", {})
    for name, kp in kpts.items():
        px = int(kp["x"] * w)
        py = int(kp["y"] * h)
        r = 5
        draw.ellipse((px - r, py - r, px + r, py + r), fill=KEYPOINT_COLOR + (220,), outline=(0, 0, 0, 180))

    # Draw eyeline + ankle horizontal guides
    pose = data.get("pose", {})
    if "eyeline_y" in pose:
        ey = int(pose["eyeline_y"] * h)
        draw.line([(0, ey), (w, ey)], fill=(255, 255, 0, 100), width=1)
    if "ankle_y" in pose:
        ay = int(pose["ankle_y"] * h)
        draw.line([(0, ay), (w, ay)], fill=(255, 255, 0, 100), width=1)

    # Legend bar at bottom
    legend_h = 28
    legend_img = Image.new("RGB", (w, legend_h), (20, 20, 20))
    legend_draw = ImageDraw.Draw(legend_img)

    try:
        font = ImageFont.truetype("/usr/share/fonts/google-noto/NotoSans-Regular.ttf", 13)
    except OSError:
        font = ImageFont.load_default()

    x_cursor = 8
    for label, color in BOX_COLORS.items():
        legend_draw.rectangle((x_cursor, 7, x_cursor + 14, 21), fill=color, outline=(180, 180, 180))
        x_cursor += 18
        legend_draw.text((x_cursor, 6), label, fill=(220, 220, 220), font=font)
        bbox = font.getbbox(label)
        x_cursor += (bbox[2] - bbox[0]) + 14

    # Compose final image
    final = Image.new("RGB", (w, h + legend_h))
    final.paste(img, (0, 0))
    final.paste(legend_img, (0, h))
    final.save(dev_path)


def write_sidecar(img_path: Path, data: dict, dry_run: bool):
    """Write sidecar YAML next to the PNG."""
    sidecar_path = img_path.with_suffix(".yaml")
    if dry_run:
        print(f"    WOULD WRITE sidecar: {sidecar_path.name}")
        print(yaml.dump(data, default_flow_style=False, sort_keys=False)[:500])
        return
    sidecar_path.write_text(yaml.dump(data, default_flow_style=False, sort_keys=False))


def update_discipline_yaml_crops(yaml_path: Path, crops: dict, dry_run: bool) -> str:
    """Update or insert crops field in discipline YAML. Returns 'inserted', 'updated', or 'unchanged'."""
    text = yaml_path.read_text()

    face = crops["face"]
    card = crops["card"]
    crops_yaml = (
        f"crops:\n"
        f"  face:\n"
        f"    x: {face['x']}\n"
        f"    y: {face['y']}\n"
        f"    w: {face['w']}\n"
        f"    h: {face['h']}\n"
        f"  card:\n"
        f"    x: {card['x']}\n"
        f"    y: {card['y']}\n"
        f"    w: {card['w']}\n"
        f"    h: {card['h']}\n"
    )

    # Check if crops already exists
    pattern = re.compile(r'^crops:\n(?:  \w+:\n(?:    [a-z]: [\d.]+\n){1,4}){1,2}', re.MULTILINE)
    m = pattern.search(text)

    if m:
        old = m.group(0)
        if old == crops_yaml:
            return "unchanged"
        new_text = text[:m.start()] + crops_yaml + text[m.end():]
        if not dry_run:
            yaml_path.write_text(new_text)
        return "updated"
    else:
        # Also remove old face_box if present
        fb_pattern = re.compile(r'\nface_box:\n(?:  [a-z]: [\d.]+\n){1,4}', re.MULTILINE)
        text = fb_pattern.sub('', text)

        if not text.endswith("\n"):
            text += "\n"
        new_text = text + "\n" + crops_yaml
        if not dry_run:
            yaml_path.write_text(new_text)
        return "inserted"


def parse_args():
    args = sys.argv[1:]
    dry_run = "--dry-run" in args
    args = [a for a in args if a != "--dry-run"]

    stack_filter = None
    disc_filter = None

    if len(args) >= 1:
        stack_filter = args[0]
    if len(args) >= 2:
        disc_filter = int(args[1])

    return stack_filter, disc_filter, dry_run


def main():
    stack_filter, disc_filter, dry_run = parse_args()

    if dry_run:
        print("DRY RUN — no files will be modified\n")

    changes = 0
    skips = 0
    errors = 0

    for stack_name, disc_indices in STACKS.items():
        if stack_filter and not stack_name.startswith(stack_filter):
            continue

        stack_dir = DISCIPLINES_DIR / stack_name
        images_dir = stack_dir / "images"

        if not images_dir.exists():
            print(f"SKIP {stack_name}: no images/ directory")
            skips += len(disc_indices)
            continue

        print(f"\n=== {stack_name} ===")

        indices = [disc_filter] if disc_filter is not None else disc_indices

        for idx in indices:
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

            print(f"  {idx:02d}: analyzing {img_path.name}...")
            data = analyze_image(img_path)

            # Write sidecar + debug overlay
            write_sidecar(img_path, data, dry_run)
            write_debug_image(img_path, data, dry_run)

            # Update discipline YAML with face + card crops only
            crops_for_yaml = {"face": data["crops"]["face"], "card": data["crops"]["card"]}
            result = update_discipline_yaml_crops(yaml_path, crops_for_yaml, dry_run)

            face = data["crops"]["face"]
            card = data["crops"]["card"]
            method = data["detection"]["method"]
            print(
                f"       face=({face['x']:.2f},{face['y']:.2f},{face['w']:.2f},{face['h']:.2f}) "
                f"card=({card['x']:.2f},{card['y']:.2f},{card['w']:.2f},{card['h']:.2f}) "
                f"[{method}] yaml:{result}"
            )

            if result != "unchanged":
                changes += 1
            else:
                skips += 1

    print(f"\nDone: {changes} updated, {skips} unchanged, {errors} errors")
    if errors > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
