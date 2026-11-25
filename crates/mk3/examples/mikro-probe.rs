//! Read-only Mikro MK3 input probe.
//!
//! Run with `cargo run -p deckard-mk3 --example mikro-probe` while the Mikro is connected.

use deckard_mk3::mikro::{self, Event};
use std::time::Duration;

fn main() {
    let mut device = mikro::Device::open().expect("open Maschine Mikro MK3");
    let mut events = Vec::new();
    println!("Listening for Mikro MK3 input; press Ctrl-C to stop.");
    loop {
        device.poll(100, &mut events).expect("read Mikro MK3");
        for event in events.drain(..) {
            match event {
                Event::Button(name, pressed) => println!("button {name} {}", if pressed { "down" } else { "up" }),
                Event::Wheel(steps) => println!("wheel {steps:+}"),
                Event::Knob(index, amount) => println!("knob {} {amount:+.3}", index + 1),
                Event::Touch(_, kind, position) => println!("slider kind {kind} position {position:.3}"),
                Event::PadOn(pad, pressure) => println!("pad {pad} down pressure {pressure:.3}"),
                Event::PadPressure(pad, pressure) => println!("pad {pad} pressure {pressure:.3}"),
                Event::PadOff(pad) => println!("pad {pad} up"),
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
