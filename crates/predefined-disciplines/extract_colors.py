#!/usr/bin/env python3
"""Extract dominant color from discipline images and write to YAML files.

For each stack's discipline, finds the latest generated image (last timestamp
in filename wins), extracts the most prominent non-gray/non-black color via
K-means clustering, and updates the `color:` field in the corresponding YAML.

Dependencies: Pillow, numpy (no sklearn needed)

Usage:
    python crates/predefined-disciplines/extract_colors.py [--dry-run]
"""

import re
import sys
from pathlib import Path

import numpy as np
from PIL import Image

DISCIPLINES_DIR = Path(__file__).parent / "src" / "defaults" / "disciplines"

STACKS = {
    "01_generic": list(range(8)),
    "02_desktop": list(range(8)),
    "03_saas": list(range(8)),
    "04_mobile": list(range(8)),
}

IMAGE_RE = re.compile(r"^(\d{2})_(.+?)_\d+_\d+x\d+_([a-z0-9]+)\.png$")


def kmeans(pixels: np.ndarray, k: int = 8, max_iter: int = 20) -> tuple[np.ndarray, np.ndarray]:
    rng = np.random.default_rng(42)
    indices = rng.choice(len(pixels), size=k, replace=False)
    centers = pixels[indices].astype(float)

    for _ in range(max_iter):
        dists = np.linalg.norm(pixels[:, None] - centers[None, :], axis=2)
        labels = np.argmin(dists, axis=1)
        new_centers = np.zeros_like(centers)
        for i in range(k):
            mask = labels == i
            if mask.any():
                new_centers[i] = pixels[mask].mean(axis=0)
            else:
                new_centers[i] = centers[i]
        if np.allclose(centers, new_centers, atol=1.0):
            break
        centers = new_centers

    return centers, labels


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


def extract_dominant_color(image_path: Path) -> str:
    img = Image.open(image_path).convert("RGB")
    img = img.resize((150, 300), Image.LANCZOS)
    pixels = np.array(img).reshape(-1, 3).astype(float)

    centers, labels = kmeans(pixels, k=8)
    counts = np.bincount(labels, minlength=len(centers))

    best_score = -1.0
    best_color = centers[0]

    for center, count in zip(centers, counts):
        r, g, b = center
        brightness = 0.299 * r + 0.587 * g + 0.114 * b
        max_c = max(r, g, b)
        min_c = min(r, g, b)
        saturation = (max_c - min_c) / max_c if max_c > 0 else 0

        # Score = saturation^2 * brightness_bell * count^0.3
        # We want the most vivid color, not the most frequent pixel
        sat_score = saturation ** 2

        # Bell curve peaking at brightness 140 (good for card borders on dark UI)
        bright_score = np.exp(-((brightness - 140) ** 2) / (2 * 80 ** 2))

        # Cube root of count so frequency matters but doesn't dominate
        freq_score = count ** 0.3

        score = sat_score * bright_score * freq_score

        if score > best_score:
            best_score = score
            best_color = center

    r, g, b = best_color

    # Boost dim colors to a minimum brightness usable as card borders on dark UI.
    # Preserves hue and relative saturation, just scales RGB up.
    brightness = 0.299 * r + 0.587 * g + 0.114 * b
    min_brightness = 100
    if brightness > 0 and brightness < min_brightness:
        scale = min_brightness / brightness
        r = min(r * scale, 255)
        g = min(g * scale, 255)
        b = min(b * scale, 255)

    r, g, b = int(round(r)), int(round(g)), int(round(b))
    return f"#{r:02x}{g:02x}{b:02x}"


def find_yaml_for_discipline(stack_dir: Path, disc_index: int) -> Path | None:
    prefix = f"{disc_index:02d}_"
    for f in stack_dir.iterdir():
        if f.name.startswith(prefix) and f.suffix == ".yaml":
            return f
    return None


def update_yaml_color(yaml_path: Path, new_color: str, dry_run: bool) -> str | None:
    text = yaml_path.read_text()
    pattern = re.compile(r'^(color:\s*")[^"]*(")', re.MULTILINE)
    m = pattern.search(text)
    if not m:
        return None
    old_color = text[m.start(1) + len(m.group(1)) : m.start(2)]
    if old_color == new_color:
        return old_color
    new_text = text[: m.start(1)] + m.group(1) + new_color + m.group(2) + text[m.end(2) :]
    if not dry_run:
        yaml_path.write_text(new_text)
    return old_color


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

            color = extract_dominant_color(img_path)
            old_color = update_yaml_color(yaml_path, color, dry_run)

            if old_color is None:
                print(f"  {idx:02d}: YAML HAS NO color: FIELD — {yaml_path.name}")
                errors += 1
            elif old_color == color:
                print(f"  {idx:02d}: {color} (unchanged) ← {img_path.name}")
                skips += 1
            else:
                action = "WOULD UPDATE" if dry_run else "UPDATED"
                print(f"  {idx:02d}: {old_color} → {color} ({action}) ← {img_path.name}")
                changes += 1

    print(f"\nDone: {changes} updated, {skips} unchanged, {errors} errors")
    if errors > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
