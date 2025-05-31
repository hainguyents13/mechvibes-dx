// Simple mouse button test - paste this code into src/main.rs temporarily to test
use rdev::{listen, Event, EventType};

fn main() {
    println!("🖱️ Starting simple mouse button test...");
    println!("🖱️ Try clicking your mouse buttons to see if they are detected.");
    println!("🖱️ Press Ctrl+C to exit.");

    if let Err(error) = listen(move |event: Event| {
        match event.event_type {
            EventType::ButtonPress(button) => {
                println!("🖱️ BUTTON PRESS: {:?}", button);
            }
            EventType::ButtonRelease(button) => {
                println!("🖱️ BUTTON RELEASE: {:?}", button);
            }
            EventType::MouseMove { x, y } => {
                // Comment out mouse move to reduce spam
                // println!("🖱️ MOUSE MOVE: ({}, {})", x, y);
            }
            EventType::Wheel { delta_x, delta_y } => {
                println!("🖱️ WHEEL: ({}, {})", delta_x, delta_y);
            }
            _ => {
                println!("🖱️ OTHER: {:?}", event.event_type);
            }
        }
    }) {
        eprintln!("❌ Error: {:?}", error);
    }
}
