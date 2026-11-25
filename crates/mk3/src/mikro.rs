//! Read-only input decoding for the Maschine Mikro MK3 (USB product 0x1700).
//!
//! This is deliberately separate from the full MK3 driver: the Mikro has no displays,
//! a different button matrix, and a different control report layout.

use hidapi::{HidApi, HidDevice};

pub const PRODUCT: u16 = 0x1700;

pub const BUTTONS: &[(usize, &str)] = &[
    (0, "MASCHINE"),
    (1, "FAVORITES"),
    (2, "BROWSER"),
    (3, "VOLUME"),
    (4, "SWING"),
    (5, "TEMPO"),
    (6, "PLUGIN"),
    (7, "SAMPLING"),
    (8, "LEFT"),
    (9, "RIGHT"),
    (10, "PITCH"),
    (11, "MOD"),
    (12, "PERFORM"),
    (13, "NOTES"),
    (14, "GROUP"),
    (15, "AUTO"),
    (16, "LOCK"),
    (17, "NOTE_REPEAT"),
    (18, "RESTART"),
    (19, "ERASE"),
    (20, "TAP"),
    (21, "FOLLOW"),
    (22, "PLAY"),
    (23, "REC"),
    (24, "STOP"),
    (25, "SHIFT"),
    (26, "FIXED_VEL"),
    (27, "PAD_MODE"),
    (28, "KEYBOARD"),
    (29, "CHORDS"),
    (30, "STEP"),
    (31, "SCENE"),
    (32, "PATTERN"),
    (33, "EVENTS"),
    (34, "VARIATION"),
    (35, "DUPLICATE"),
    (36, "SELECT"),
    (37, "SOLO"),
    (38, "MUTE"),
    (39, "WHEEL"),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Button(&'static str, bool),
    Wheel(i32),
    Slider(f64),
    PadOn(usize, f64),
    PadPressure(usize, f64),
    PadOff(usize),
}

#[derive(Default)]
pub struct Decoder {
    buttons: [u8; 5],
    wheel: Option<u8>,
    slider: Option<u8>,
    held: [bool; 16],
}

impl Decoder {
    pub fn decode(&mut self, report: &[u8], out: &mut Vec<Event>) {
        match report.first() {
            Some(1) if report.len() >= 11 => self.controls(report, out),
            Some(2) => self.pads(&report[1..], out),
            _ => {}
        }
    }

    fn controls(&mut self, r: &[u8], out: &mut Vec<Event>) {
        let bits: [u8; 5] = r[1..6].try_into().unwrap();
        let old = self.buttons;
        for &(bit, name) in BUTTONS {
            if bit >= 40 {
                continue;
            }
            let (was, is) = (old[bit / 8] >> (bit % 8) & 1, bits[bit / 8] >> (bit % 8) & 1);
            if was != is {
                out.push(Event::Button(name, is != 0));
            }
        }
        self.buttons = bits;

        let wheel = r[7] & 0x0f;
        if let Some(old) = self.wheel {
            let delta = ((wheel as i32 - old as i32 + 8).rem_euclid(16)) - 8;
            if delta != 0 {
                out.push(Event::Wheel(delta));
            }
        }
        self.wheel = Some(wheel);

        let slider = r[10].min(200);
        if self.slider.is_some_and(|old| old != slider) {
            out.push(Event::Slider(f64::from(slider) / 200.0));
        }
        self.slider = Some(slider);
    }

    fn pads(&mut self, bytes: &[u8], out: &mut Vec<Event>) {
        for entry in bytes.chunks_exact(3) {
            let index = entry[0];
            let word = u16::from_be_bytes([entry[1], entry[2]]);
            if index == 0 && word == 0 {
                break;
            }
            let Some(slot) = (index < 16).then_some(index as usize) else { continue };
            let pad = pad_number(index);
            let pressure = f64::from(word & 0x0fff) / 4095.0;
            match word >> 12 {
                0 | 1 => {
                    out.push(Event::PadOn(pad, pressure));
                    self.held[slot] = true;
                }
                4 if self.held[slot] => out.push(Event::PadPressure(pad, pressure)),
                2 | 3 if std::mem::take(&mut self.held[slot]) => out.push(Event::PadOff(pad)),
                _ => {}
            }
        }
    }
}

pub fn pad_number(index: u8) -> usize {
    (3 - usize::from(index / 4)) * 4 + usize::from(index % 4) + 1
}

pub struct Device {
    hid: HidDevice,
    decoder: Decoder,
}

impl Device {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| e.to_string())?;
        let hid = api.open(crate::VENDOR, PRODUCT).map_err(|e| format!("couldn't open Maschine Mikro MK3: {e}"))?;
        Ok(Self { hid, decoder: Decoder::default() })
    }

    pub fn poll(&mut self, timeout_ms: i32, out: &mut Vec<Event>) -> Result<(), String> {
        let mut report = [0u8; 64];
        let n = self.hid.read_timeout(&mut report, timeout_ms).map_err(|e| e.to_string())?;
        if n != 0 {
            self.decoder.decode(&report[..n], out);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls(bytes: [u8; 5], wheel: u8, slider: u8) -> Vec<u8> {
        [vec![1], bytes.to_vec(), vec![0, wheel, 0, 0, slider]].concat()
    }

    #[test]
    fn decodes_named_buttons_and_rotary() {
        let mut decoder = Decoder::default();
        let mut events = Vec::new();
        decoder.decode(&controls([0, 0, 0, 0, 0], 0, 0), &mut events);
        decoder.decode(&controls([0, 4, 0, 0, 0], 1, 100), &mut events);
        assert_eq!(events, vec![Event::Button("PITCH", true), Event::Wheel(1), Event::Slider(0.5)]);
    }

    #[test]
    fn maps_bottom_left_pad_to_one() {
        assert_eq!(pad_number(12), 1);
        assert_eq!(pad_number(3), 16);
    }
}
