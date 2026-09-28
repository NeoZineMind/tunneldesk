from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "src-tauri" / "icons"

NAVY = (15, 59, 130, 255)
WHITE = (248, 250, 252, 255)
BLUE = (37, 99, 235, 255)
SKY = (32, 183, 229, 255)

APP_SIZE = 1024
TRAY_SIZE = 256
SCALE = 4


def box(values: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
    return tuple(value * SCALE for value in values)


def rounded(
    draw: ImageDraw.ImageDraw,
    values: tuple[int, int, int, int],
    radius: int,
    fill: tuple[int, int, int, int],
) -> None:
    draw.rounded_rectangle(box(values), radius=radius * SCALE, fill=fill)


def circle(
    draw: ImageDraw.ImageDraw,
    center: tuple[int, int],
    radius: int,
    fill: tuple[int, int, int, int],
) -> None:
    x, y = center
    draw.ellipse(box((x - radius, y - radius, x + radius, y + radius)), fill=fill)


def draw_app_mark(draw: ImageDraw.ImageDraw) -> None:
    rounded(draw, (184, 668, 816, 708), 20, NAVY)
    draw.ellipse(box((142, 644, 214, 716)), fill=WHITE, outline=NAVY, width=22 * SCALE)
    draw.ellipse(box((786, 644, 858, 716)), fill=WHITE, outline=NAVY, width=22 * SCALE)

    rounded(draw, (230, 190, 730, 292), 12, NAVY)
    rounded(draw, (426, 238, 542, 682), 12, NAVY)

    curve = [(470, 665), (500, 650), (520, 590), (545, 540), (575, 525), (610, 550), (648, 598)]
    draw.line([(x * SCALE, y * SCALE) for x, y in curve], fill=SKY, width=30 * SCALE, joint="curve")
    draw.polygon(
        [(620 * SCALE, 568 * SCALE), (653 * SCALE, 603 * SCALE), (608 * SCALE, 605 * SCALE)],
        fill=SKY,
    )


def draw_tray_mark(draw: ImageDraw.ImageDraw) -> None:
    rounded(draw, (78, 92, 178, 116), 8, WHITE)
    rounded(draw, (116, 112, 140, 190), 8, WHITE)
    rounded(draw, (82, 168, 174, 188), 8, WHITE)
    circle(draw, (78, 178), 12, WHITE)
    circle(draw, (178, 178), 12, WHITE)
    curve = [(126, 169), (136, 163), (139, 146), (148, 141), (159, 151), (170, 164)]
    draw.line([(x * SCALE, y * SCALE) for x, y in curve], fill=SKY, width=8 * SCALE, joint="curve")


def build_app_icon(size: int = APP_SIZE) -> Image.Image:
    canvas = APP_SIZE * SCALE
    image = Image.new("RGBA", (canvas, canvas), (0, 0, 0, 0))
    draw_app_mark(ImageDraw.Draw(image))
    return image.resize((size, size), Image.Resampling.LANCZOS)


def build_tray_icon(size: int = TRAY_SIZE) -> Image.Image:
    canvas = TRAY_SIZE * SCALE
    image = Image.new("RGBA", (canvas, canvas), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    rounded(draw, (34, 30, 222, 218), 46, NAVY)
    draw_tray_mark(ImageDraw.Draw(image))
    return image.resize((size, size), Image.Resampling.LANCZOS)


def save_windows_store_logos(app: Image.Image) -> None:
    mapping = {
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    for name, size in mapping.items():
        app.resize((size, size), Image.Resampling.LANCZOS).save(OUT / name)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    app = build_app_icon()
    tray = build_tray_icon()

    app.save(OUT / "icon-source.png")
    app.save(OUT / "icon.png")
    app.resize((32, 32), Image.Resampling.LANCZOS).save(OUT / "32x32.png")
    app.resize((64, 64), Image.Resampling.LANCZOS).save(OUT / "64x64.png")
    app.resize((128, 128), Image.Resampling.LANCZOS).save(OUT / "128x128.png")
    app.resize((256, 256), Image.Resampling.LANCZOS).save(OUT / "128x128@2x.png")
    app.save(
        OUT / "icon.ico",
        sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )
    app.save(OUT / "icon.icns")
    save_windows_store_logos(app)

    tray.save(OUT / "tray-icon.png")
    tray.save(
        OUT / "tray-icon.ico",
        sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    print(OUT)


if __name__ == "__main__":
    main()
