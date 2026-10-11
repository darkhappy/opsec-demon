"""Render original animated SIM illustrations; Pillow is required only to rebuild art."""
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

OUTPUT = Path(__file__).resolve().parents[1] / 'docs/assets/discord'
COLORS = {
    'nightfall': ('NIGHTFALL ARCHON', (247, 48, 100)),
    'black_choir': ('BLACK CHOIR // ABYSS-999', (181, 118, 255)),
    'yoggies': ('YOGGIE QUARTERMASTER', (242, 151, 228)),
    'porcelain': ('PORCELAIN ARCHDEMON', (95, 230, 255)),
    'ghost_kernel': ('GHOST KERNEL // SOUL CACHE', (115, 255, 171)),
}


def font(size):
    for name in ['C:/Windows/Fonts/consolab.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf']:
        if Path(name).exists():
            return ImageFont.truetype(name, size)
    return ImageFont.load_default(size=size)


def frame(key, title, color, index):
    image = Image.new('RGB', (512, 512), (9, 7, 22))
    draw = ImageDraw.Draw(image)
    phase = index * math.tau / 24
    for line in range(16, 512, 24):
        draw.line((0, line, 512, line), fill=(24, 17, 40))
        draw.line((line, 0, line, 512), fill=(24, 17, 40))
    draw.rounded_rectangle((14, 14, 498, 498), radius=20, outline=color, width=3)
    draw.text((32, 34), 'OPSEC DEMON', fill=color, font=font(32))
    draw.text((32, 76), 'SIM // FICTIONAL DIRECTORATE', fill=(187, 178, 213), font=font(19))
    for radius in [115, 138, 155]:
        draw.ellipse((256-radius, 272-radius, 256+radius, 272+radius), outline=color, width=2)
    for spoke in range(12):
        angle = phase + spoke * math.tau / 12
        x, y = 256 + 155 * math.cos(angle), 272 + 155 * math.sin(angle)
        draw.ellipse((x-4, y-4, x+4, y+4), fill=color)
    if key == 'yoggies':
        for slot in range(5):
            angle = phase / 6 + slot * math.tau / 5 - math.pi/2
            x, y = 256 + 74 * math.cos(angle), 272 + 74 * math.sin(angle)
            draw.ellipse((x-24, y-24, x+24, y+24), fill=(243, 222, 245), outline=color, width=3)
            draw.ellipse((x-10, y-8, x+10, y+12), fill=(140, 42, 163))
            draw.text((x-5, y-18), str(slot+1), fill=(40, 11, 55), font=font(15))
    elif key == 'porcelain':
        draw.rounded_rectangle((206, 218, 306, 305), 15, fill=(232, 241, 252), outline=color, width=4)
        draw.ellipse((191, 272, 321, 335), fill=color)
        draw.ellipse((213, 284, 299, 321), fill=(9, 7, 22))
        draw.rectangle((242, 329, 270, 357), fill=(232, 241, 252))
    elif key == 'ghost_kernel':
        draw.rectangle((193, 210, 319, 337), outline=color, width=5)
        for offset in range(0, 5):
            y = 223 + offset * 22
            draw.text((207, y), '101010' if (index+offset) % 2 else '010101', fill=color, font=font(20))
    else:
        draw.polygon([(256, 175), (340, 320), (172, 320)], outline=color, width=6)
        draw.ellipse((221, 237, 291, 277), outline=color, width=4)
        draw.ellipse((247, 246, 265, 268), fill=color)
        draw.line((225, 185, 205, 150), fill=color, width=5)
        draw.line((287, 185, 307, 150), fill=color, width=5)
    draw.text((256, 439), title, anchor='mm', fill=color, font=font(20))
    draw.text((256, 473), 'ILLUSTRATION / NO ACTUAL CLEARANCE', anchor='mm', fill=(187, 178, 213), font=font(16))
    return image


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for key, (title, color) in COLORS.items():
        frames = [frame(key, title, color, index) for index in range(24)]
        frames[0].save(OUTPUT / f'{key}.png')
        frames[0].save(OUTPUT / f'{key}.gif', save_all=True, append_images=frames[1:], duration=120, loop=0, disposal=2)
    print(f'Rendered {len(COLORS)} original SIM illustration pairs to {OUTPUT}')


if __name__ == '__main__':
    main()
