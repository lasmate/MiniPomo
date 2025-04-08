// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

slint::include_modules!();

static mut WORK_TIME: u32 = 25;
static mut PLAY_TIME: u32 = 25;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    
    // Use separate handles for each callback
    let ui_handle_work = ui.as_weak();
    ui.on_update_workTime(move |work_time| {
        let ui = ui_handle_work.upgrade().unwrap();
        unsafe {
            WORK_TIME = work_time as u32;
        }
        println!("Work time updated: {}", work_time);
    });
    
    let ui_handle_play = ui.as_weak();
    ui.on_update_playTime(move |play_time| {
        
        unsafe {
            PLAY_TIME = play_time as u32;
        }println!("Play time updated: {}", play_time);
        // Additional logic for play time handling can be added here
    });

    ui.on_startTimer(move || {
        let ui_handle_timer = ui.as_weak();
        let work_duration_mins = unsafe { WORK_TIME };
        let play_duration_mins = unsafe { PLAY_TIME };
        let work_total_seconds = work_duration_mins * 60;
        let play_total_seconds = play_duration_mins * 60;
        let update_interval_ms = 600; // Update approximately every 0.6 seconds
        let mut elapsed_seconds = 0;

        // Create and start the timer
        let _timer = std::thread::spawn(move || {
            // Initialize progress to 0
            let ui = ui_handle_timer.upgrade().unwrap();
            ui.set_remaining_timeChanged(0);
            
            // Work timer phase
            while elapsed_seconds < work_total_seconds {
                std::thread::sleep(std::time::Duration::from_millis(update_interval_ms));
                elapsed_seconds += (update_interval_ms / 1000) as u32;
                
                // Calculate progress from 0.0 to 1.0
                let progress = elapsed_seconds as f32 / work_total_seconds as f32;
                if let Some(ui) = ui_handle_timer.upgrade() {
                    ui.set_remaining_time((progress * 100.0) as i32);
                    // Note: isWorkTime property is not defined in your UI file
                } else {
                    break; // UI was dropped
                }
            }
            
            // Work timer completed
            if let Some(ui) = ui_handle_timer.upgrade() {
                ui.set_remaining_time(100);
                println!("Work timer completed!");
                
                // Reset for play timer
                elapsed_seconds = 0;
                // Note: isWorkTime property is not defined in your UI file
                ui.set_remaining_time(0);
                
                // Play timer phase
                while elapsed_seconds < play_total_seconds {
                    std::thread::sleep(std::time::Duration::from_millis(update_interval_ms));
                    elapsed_seconds += update_interval_ms / 1000;
                    
                    // Calculate progress from 0.0 to 1.0
                    let progress = elapsed_seconds as f32 / play_total_seconds as f32;
                    if let Some(ui) = ui_handle_timer.upgrade() {
                        ui.set_remaining_time((progress * 100.0) as i32);
                    } else {
                        break; // UI was dropped
                    }
                }
                
                // Play timer completed
                if let Some(ui) = ui_handle_timer.upgrade() {
                    ui.set_remaining_time(0);
                    println!("Play timer completed!");
                }
            }
        });

        unsafe {
            println!("Timer started with Work Time: {}", WORK_TIME);

        }

    });
    

    ui.run().map_err(|e| {
        eprintln!("Failed to run the UI: {}", e);
        e
    })?;

    Ok(())
}
