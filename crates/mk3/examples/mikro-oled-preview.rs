use deckard_mk3::mikro::{OLED_HEIGHT, OLED_WIDTH, oled_frame_from_transfers, oled_transfers};
use png::{BitDepth, ColorType, Encoder};
use std::fs::File;

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'G' => [0x0f, 0x10, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        ' ' => [0; 7],
        _ => [0; 7],
    }
}

fn text_frame(text: &str) -> Vec<u8> {
    let scale = 2;
    let advance = 13;
    let width = text.chars().count() * advance;
    let left = (OLED_WIDTH.saturating_sub(width)) / 2;
    let mut frame = vec![0; OLED_WIDTH * OLED_HEIGHT / 8];
    for (i, ch) in text.chars().enumerate() {
        for (row, bits) in glyph(ch).into_iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = left + i * advance + col * scale + dx;
                        let y = 1 + row * scale + dy;
                        if x < OLED_WIDTH && y < OLED_HEIGHT {
                            frame[y * (OLED_WIDTH / 8) + x / 8] |= 0x80 >> (x % 8);
                        }
                    }
                }
            }
        }
    }
    frame
}

fn write_png(path: &str, frame: &[u8]) {
    let scale = 8;
    let mut pixels = vec![0u8; OLED_WIDTH * scale * OLED_HEIGHT * scale];
    for y in 0..OLED_HEIGHT {
        for x in 0..OLED_WIDTH {
            let value = if frame[y * (OLED_WIDTH / 8) + x / 8] & (0x80 >> (x % 8)) != 0 { 255 } else { 0 };
            for dy in 0..scale {
                for dx in 0..scale {
                    pixels[(y * scale + dy) * OLED_WIDTH * scale + x * scale + dx] = value;
                }
            }
        }
    }
    let file = File::create(path).unwrap();
    let mut encoder = Encoder::new(file, (OLED_WIDTH * scale) as u32, (OLED_HEIGHT * scale) as u32);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&pixels).unwrap();
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "mikro-oled-preview.png".into());
    let frame = oled_frame_from_transfers(&oled_transfers(&text_frame("ROOT C4"))).unwrap();
    write_png(&path, &frame);
    println!("wrote {path}");
}
