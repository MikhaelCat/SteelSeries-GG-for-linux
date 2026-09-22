#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! hidapi = "2.6"
//! anyhow = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = "0.4"
//! ```

use anyhow::Result;
use hidapi::{HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct DeviceInfo {
    vendor_id: u16,
    product_id: u16,
    path: String,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
    release_number: u16,
}

fn main() -> Result<()> {
    println!("🔧 SteelSeries Hardware Test Runner");
    println!("====================================\n");

    // Initialize HIDAPI
    let api = HidApi::new()?;
    println!("✅ HIDAPI initialized\n");

    // Detect SteelSeries devices
    let mut detected_devices = Vec::new();

    for device in api.list_devices() {
        const SS_VID: u16 = 0x1246;

        if device.vendor_id() == SS_VID {
            let info = DeviceInfo {
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
                path: device.path().to_string_lossy().into_owned(),
                manufacturer: device.manufacturer_string(),
                product: device.product_string(),
                serial_number: device.serial_number_string(),
                release_number: device.release_number(),
            };

            detected_devices.push(info);
            println!("📱 Found SteelSeries Device:");
            println!("   Path: {}", info.path);
            println!("   Product: {:?}", info.product);
            println!("   VID: 0x{:04x} PID: 0x{:04x}", info.vendor_id, info.product_id);
            println!("   Release: v{}.{}", info.release_number >> 8, info.release_number & 0xFF);
            println!();
        }
    }

    if detected_devices.is_empty() {
        eprintln!("❌ No SteelSeries devices found!");
        eprintln!("💡 Connect your SteelSeries keyboard/mouse and try again.");
        std::process::exit(1);
    }

    println!("📊 Total Devices Found: {}\n", detected_devices.len());

    // Test RGB lighting on keyboards
    test_rgb_lighting(&api, &detected_devices)?;

    // Performance measurement
    measure_hardware_latency(&api, &detected_devices)?;

    println!("\n✅ All hardware tests completed successfully!");
    Ok(())
}

fn test_rgb_lighting(api: &HidApi, devices: &[DeviceInfo]) -> Result<()> {
    println!("🎨 RGB Lighting Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for device in devices.iter() {
        // Open device
        if let Ok(mut dev) = api.open_path(device.path.clone()) {
            println!("🔍 Testing device: {:?}\n", device.product);

            // Get device capabilities
            let report_size = dev.report_descriptor().map(|d| d.len()).unwrap_or(0);
            println!("   Report Size: {} bytes", report_size);

            // Try to send basic RGB command (varies by device)
            // This is a generic test - actual commands depend on device model
            let rgb_commands = vec![
                vec![0x01, 0xFF, 0x00, 0x00],     // Red
                vec![0x01, 0x00, 0xFF, 0x00],     // Green
                vec![0x01, 0x00, 0x00, 0xFF],     // Blue
                vec![0x01, 0xFF, 0xFF, 0x00],     // Yellow
            ];

            for (i, cmd) in rgb_commands.iter().enumerate() {
                match dev.send_output_report(&cmd) {
                    Ok(_) => {
                        println!("   ✅ Command {} sent successfully: RGB({},{},{})", 
                                i + 1, cmd[1], cmd[2], cmd[3]);
                        
                        // Small delay to allow LED response
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        println!("   ⚠️  Command {} failed: {}", i + 1, e);
                    }
                }
            }

            println!();
            dev.close_device()?;
        } else {
            println!("⚠️  Could not open device for testing: {:?}\n", device.product);
        }
    }

    Ok(())
}

fn measure_hardware_latency(api: &HidApi, devices: &[DeviceInfo]) -> Result<()> {
    println!("⚡ Latency Measurements");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    const ITERATIONS: usize = 100;

    for device in devices.iter() {
        if let Ok(mut dev) = api.open_path(device.path.clone()) {
            println!("🔬 Measuring latency for: {:?}\n", device.product);

            let mut latencies_ns = Vec::new();

            for i in 0..ITERATIONS {
                // Create a test command
                let cmd = vec![0x02, i as u8, 0x00, 0x00];

                let start = Instant::now();
                
                match dev.send_output_report(&cmd) {
                    Ok(_) => {
                        let elapsed = start.elapsed();
                        latencies_ns.push(elapsed.as_nanos());
                    }
                    Err(_) => {
                        latencies_ns.push(u128::MAX); // Mark as failure
                    }
                }
            }

            // Calculate statistics
            if !latencies_ns.is_empty() {
                let valid_latencies: Vec<u128> = latencies_ns
                    .iter()
                    .filter(|&&l| l != u128::MAX)
                    .cloned()
                    .collect();

                if !valid_latencies.is_empty() {
                    let avg: u128 = valid_latencies.iter().sum() / valid_latencies.len() as u128;
                    let max: u128 = valid_latencies.iter().max().unwrap();
                    let min: u128 = valid_latencies.iter().min().unwrap();

                    println!("   Results ({} iterations):", ITERATIONS);
                    println!("   ├─ Average: {:.2} μs", avg as f64 / 1000.0);
                    println!("   ├─ Min:     {:.2} μs", min as f64 / 1000.0);
                    println!("   └─ Max:     {:.2} μs", max as f64 / 1000.0);
                    println!();

                    // Check performance target
                    if avg as f64 / 1000.0 < 1000.0 {
                        println!("   ✅ Performance target met (< 1ms)");
                    } else {
                        println!("   ⚠️  Target exceeded (> 1ms), may need optimization");
                    }
                }
            }
        }
    }

    Ok(())
}
