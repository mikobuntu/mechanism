use deckard_mk3::mikro::{OLED_HEIGHT, OLED_WIDTH, oled_frame_from_transfers, oled_transfers};
use png::{BitDepth, ColorType, Encoder};
use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
};

const SCALE: usize = 8;
const ADVANCE: usize = 13;

struct Fixture {
    name: &'static str,
    text: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture { name: "standby", text: "MASCHINE MIKRO" },
    Fixture { name: "group-select", text: "GROUP SELECT" },
    Fixture { name: "root-c4", text: "ROOT C4" },
    Fixture { name: "root-c-sharp-4", text: "ROOT C#4" },
    Fixture { name: "browser", text: "BROWSER" },
    Fixture { name: "projects", text: "PROJECTS" },
    Fixture { name: "favorites", text: "FAVORITES" },
    Fixture { name: "pattern", text: "PATTERN" },
    Fixture { name: "scene", text: "SCENE" },
    Fixture { name: "step", text: "STEP" },
    Fixture { name: "keyboard", text: "KEYBOARD" },
    Fixture { name: "chords", text: "CHORDS" },
    Fixture { name: "pad-mode", text: "PAD MODE" },
    Fixture { name: "song", text: "SONG" },
    Fixture { name: "events", text: "EVENTS" },
    Fixture { name: "control", text: "CONTROL" },
    Fixture { name: "mixer", text: "MIXER" },
    Fixture { name: "module", text: "MODULE" },
    Fixture { name: "grid", text: "GRID" },
    Fixture { name: "mod-matrix", text: "MOD MATRIX" },
    Fixture { name: "sampling", text: "SAMPLING" },
    Fixture { name: "tuner", text: "TUNER" },
    Fixture { name: "audio", text: "AUDIO" },
    Fixture { name: "export", text: "EXPORTING SOUND" },
    Fixture { name: "settings", text: "SETTINGS" },
    Fixture { name: "loading", text: "LOADING" },
    Fixture { name: "playing", text: "PLAYING" },
    Fixture { name: "saved", text: "SAVED" },
    Fixture { name: "long-name", text: "MY VERY LONG PROJECT NAME" },
];

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0f, 0x10, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0e],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1b, 0x11],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x0e, 0x10, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x01, 0x0e],
        '-' => [0, 0, 0, 0x1f, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 0x06, 0x06],
        ':' => [0, 0x06, 0x06, 0, 0x06, 0x06, 0],
        '/' => [1, 2, 2, 4, 8, 8, 0x10],
        '#' => [0x0a, 0x1f, 0x0a, 0x0a, 0x1f, 0x0a, 0],
        ' ' => [0; 7],
        _ => [0; 7],
    }
}

fn text_frame(text: &str, offset: usize) -> Vec<u8> {
    let width = text.chars().count() * ADVANCE;
    let left = if width < OLED_WIDTH { (OLED_WIDTH - width) / 2 } else { 0 };
    let stride = OLED_WIDTH / 8;
    let mut frame = vec![0; stride * OLED_HEIGHT];
    for (index, ch) in text.chars().enumerate() {
        for (row, bits) in glyph(ch).into_iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }
                for dy in 0..2 {
                    for dx in 0..2 {
                        let x = left + index * ADVANCE + col * 2 + dx;
                        let y = 1 + row * 2 + dy;
                        let x = x.saturating_sub(offset);
                        if x < OLED_WIDTH && y < OLED_HEIGHT {
                            frame[y * stride + x / 8] |= 0x80 >> (x % 8);
                        }
                    }
                }
            }
        }
    }
    frame
}

fn encoded_frame(text: &str, offset: usize) -> Vec<u8> {
    let source = text_frame(text, offset);
    let transfers = oled_transfers(&source);
    oled_frame_from_transfers(&transfers).expect("generated OLED transfer must decode")
}

fn write_png(path: &Path, pixels: &[u8], width: usize, height: usize) {
    let file = File::create(path).expect("create preview PNG");
    let mut encoder = Encoder::new(file, width as u32, height as u32);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    encoder.write_header().expect("write PNG header").write_image_data(pixels).expect("write PNG pixels");
}

fn enlarged(frame: &[u8]) -> Vec<u8> {
    let stride = OLED_WIDTH / 8;
    let mut pixels = vec![0u8; OLED_WIDTH * SCALE * OLED_HEIGHT * SCALE];
    for y in 0..OLED_HEIGHT {
        for x in 0..OLED_WIDTH {
            let value = if frame[y * stride + x / 8] & (0x80 >> (x % 8)) != 0 { 255 } else { 0 };
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    pixels[(y * SCALE + dy) * OLED_WIDTH * SCALE + x * SCALE + dx] = value;
                }
            }
        }
    }
    pixels
}

fn main() {
    let output = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "target/mikro-oled-previews".into()));
    create_dir_all(&output).expect("create output directory");
    let mut files = Vec::new();
    for fixture in FIXTURES {
        let width = fixture.text.chars().count() * ADVANCE;
        let offsets = if width > OLED_WIDTH { vec![0, 13, width - OLED_WIDTH] } else { vec![0] };
        for offset in offsets {
            let suffix = if offset == 0 { "".to_string() } else { format!("-offset-{offset}") };
            let path = output.join(format!("{}{}.png", fixture.name, suffix));
            write_png(&path, &enlarged(&encoded_frame(fixture.text, offset)), OLED_WIDTH * SCALE, OLED_HEIGHT * SCALE);
            files.push(path);
        }
    }
    let sheet_width = OLED_WIDTH * SCALE;
    let sheet_height = OLED_HEIGHT * SCALE * files.len();
    let mut sheet = vec![0u8; sheet_width * sheet_height];
    for (index, path) in files.iter().enumerate() {
        let frame = encoded_frame(
            FIXTURES.iter().find(|fixture| path.file_name().unwrap().to_string_lossy().starts_with(fixture.name)).unwrap().text,
            path.file_stem().unwrap().to_string_lossy().split("offset-").nth(1).and_then(|v| v.parse().ok()).unwrap_or(0),
        );
        let stride = OLED_WIDTH / 8;
        for y in 0..OLED_HEIGHT {
            for x in 0..OLED_WIDTH {
                let value = if frame[y * stride + x / 8] & (0x80 >> (x % 8)) != 0 { 255 } else { 0 };
                for dy in 0..SCALE {
                    for dx in 0..SCALE {
                        sheet[(index * OLED_HEIGHT * SCALE + y * SCALE + dy) * sheet_width + x * SCALE + dx] = value;
                    }
                }
            }
        }
    }
    write_png(&output.join("contact-sheet.png"), &sheet, sheet_width, sheet_height);
    println!("wrote {} previews and {}", files.len(), output.join("contact-sheet.png").display());
}
