//! Read-only input decoding for the Maschine Mikro MK3 (USB product 0x1700).
//!
//! This is deliberately separate from the full MK3 driver: the Mikro has no displays,
//! a different button matrix, and a different control report layout.

use hidapi::{HidApi, HidDevice};

pub use crate::Event;

pub const PRODUCT: u16 = 0x1700;

pub const BUTTONS: &[(usize, &str)] = &[
    (0, "BUTTON_MASCHINE"),
    (1, "BUTTON_FAVORITES"),
    (2, "BUTTON_BROWSE"),
    (3, "BUTTON_VOLUME"),
    (4, "BUTTON_SWING"),
    (5, "BUTTON_TEMPO"),
    (6, "BUTTON_PLUGIN"),
    (7, "BUTTON_SAMPLE"),
    (8, "BUTTON_LEFT"),
    (9, "BUTTON_RIGHT"),
    (10, "BUTTON_PITCH"),
    (11, "BUTTON_MOD"),
    (12, "BUTTON_PERFORM"),
    (13, "BUTTON_NOTES"),
    (14, "BUTTON_GROUP"),
    (15, "BUTTON_AUTO"),
    (16, "BUTTON_LOCK"),
    (17, "BUTTON_NOTE_REPEAT"),
    (18, "BUTTON_TRANSPORT_LOOP"),
    (19, "BUTTON_TRANSPORT_ERASE"),
    (20, "BUTTON_TAP"),
    (21, "BUTTON_FOLLOW"),
    (22, "BUTTON_TRANSPORT_PLAY"),
    (23, "BUTTON_TRANSPORT_RECORD"),
    (24, "BUTTON_TRANSPORT_STOP"),
    (25, "BUTTON_SHIFT"),
    (26, "BUTTON_FIX_VELOCITY"),
    (27, "BUTTON_PAD_MODE"),
    (28, "BUTTON_KEYBOARD"),
    (29, "BUTTON_CHORDS"),
    (30, "BUTTON_STEP"),
    (31, "BUTTON_SCENE"),
    (32, "BUTTON_PATTERN"),
    (33, "BUTTON_EVENTS"),
    (34, "BUTTON_VARIATION"),
    (35, "BUTTON_DUPLICATE"),
    (36, "BUTTON_SELECT"),
    (37, "BUTTON_SOLO"),
    (38, "BUTTON_MUTE"),
    (39, "BUTTON_WHEEL"),
];

const LED_BUTTONS: [Option<&str>; 39] = [
    None,
    None,
    Some("LED_BROWSE"),
    Some("LED_VOLUME"),
    Some("LED_SWING"),
    Some("LED_TEMPO"),
    Some("LED_PLUGIN"),
    None,
    Some("LED_LEFT"),
    Some("LED_RIGHT"),
    Some("LED_PITCH"),
    Some("LED_MOD"),
    Some("LED_PERFORM"),
    Some("LED_NOTES"),
    None,
    Some("LED_AUTO_WRITE"),
    Some("LED_LOCK"),
    Some("LED_NOTE_REPEAT"),
    Some("LED_TRANSPORT_LOOP"),
    Some("LED_TRANSPORT_ERASE"),
    Some("LED_TAP"),
    Some("LED_FOLLOW"),
    Some("LED_TRANSPORT_PLAY"),
    Some("LED_TRANSPORT_RECORD"),
    Some("LED_TRANSPORT_STOP"),
    Some("LED_SHIFT"),
    Some("LED_FIX_VELOCITY"),
    Some("LED_PAD_MODE"),
    Some("LED_KEYBOARD"),
    Some("LED_CHORDS"),
    Some("LED_STEP"),
    Some("LED_SCENE"),
    Some("LED_PATTERN"),
    None,
    Some("LED_VARIATION"),
    Some("LED_DUPLICATE"),
    Some("LED_SELECT"),
    Some("LED_SOLO"),
    Some("LED_MUTE"),
];

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
            let old = self.slider.unwrap_or(0);
            let kind = match (old == 0, slider == 0) {
                (true, false) => 0,
                (false, true) => 2,
                _ => 1,
            };
            out.push(Event::Touch(0, kind, f64::from(slider) / 200.0));
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

    /// Send the existing DECKARD LED state in the Mikro's 0x80 layout.
    pub fn write_leds(&self, leds: &crate::leds::Leds) -> Result<(), String> {
        let mut report = [0u8; 81];
        report[0] = 0x80;
        for (index, name) in LED_BUTTONS.into_iter().enumerate() {
            report[1 + index] = name.and_then(|name| leds.get(name)).map(mikro_button_led).unwrap_or(0);
        }
        for index in 0..16 {
            let pad = pad_number(index as u8);
            report[1 + 39 + index] = leds.get(&format!("LED_PAD_{pad}")).unwrap_or(0);
        }
        for index in 0..25 {
            report[1 + 55 + index] = leds.get(&format!("LED_TS_{:02}", index + 1)).map(mikro_colour_led).unwrap_or(0);
        }
        self.hid.write(&report).map(|_| ()).map_err(|e| e.to_string())
    }
}

fn mikro_button_led(value: u8) -> u8 {
    match value {
        0 => 0,
        0x7f => 0x7e,
        _ => 0x7c,
    }
}

fn mikro_colour_led(value: u8) -> u8 {
    u8::from(value != 0) * 0x7e
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
        assert_eq!(events, vec![Event::Button("BUTTON_PITCH", true), Event::Wheel(1), Event::Touch(0, 0, 0.5)]);
    }

    #[test]
    fn maps_bottom_left_pad_to_one() {
        assert_eq!(pad_number(12), 1);
        assert_eq!(pad_number(3), 16);
    }

    #[test]
    fn maps_full_mk3_led_names_to_mikro_layout() {
        assert_eq!(LED_BUTTONS[22], Some("LED_TRANSPORT_PLAY"));
        assert_eq!(pad_number(0), 13);
        assert_eq!(pad_number(12), 1);
    }

    #[test]
    fn converts_full_mk3_button_levels_to_mikro_levels() {
        assert_eq!(mikro_button_led(0), 0);
        assert_eq!(mikro_button_led(crate::leds::BACKLIGHT), 0x7c);
        assert_eq!(mikro_button_led(crate::leds::ON), 0x7e);
        assert_eq!(mikro_colour_led(0), 0);
        assert_eq!(mikro_colour_led(1), 0x7e);
    }
}
