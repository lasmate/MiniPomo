#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::{Arc, Mutex}};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    
    // Use Arc<Mutex> instead of unsafe globals
    let work_time = Arc::new(Mutex::new(25));
    let play_time = Arc::new(Mutex::new(25));
    
    // Work time callback
    let work_time_clone = work_time.clone();
    ui.on_update_workTime(move |time| {
        *work_time_clone.lock().unwrap() = time as u32;
        println!("Work time updated: {}", time);
    });
    
    // Play time callback
    let play_time_clone = play_time.clone();
    ui.on_update_playTime(move |time| {
        *play_time_clone.lock().unwrap() = time as u32;
        println!("Play time updated: {}", time);
    });

    // Timer callback
    let ui_handle = ui.as_weak();
    let work_time_clone = work_time.clone();
    let play_time_clone = play_time.clone();
    ui.on_startTimer(move || {
        let ui_handle = ui_handle.clone();
        let work_mins = *work_time_clone.lock().unwrap();
        let play_mins = *play_time_clone.lock().unwrap();
        
        std::thread::spawn(move || {
            run_timer(&ui_handle, work_mins, play_mins);
        });
        
        println!("Timer started with Work Time: {}", work_mins);
    });
    
    ui.run()?;
    Ok(())
}

fn run_timer(ui_handle: &slint::Weak<AppWindow>, work_mins: u32, play_mins: u32) {
    let work_seconds = work_mins * 60;
    let play_seconds = play_mins * 60;
    let update_ms = 1000; // Update every second for smoother progress
    
    // Helper function to update progress
    let update_progress = |seconds: u32, total: u32| {
        if let Some(ui) = ui_handle.upgrade() {
            // Calculate remaining time percentage (0-100)
            let remaining = total - seconds;
            let progress = (remaining as f32 / total as f32 * 100.0) as i32;
            ui.set_remainingTime(progress);
            println!("Progress: {}% ({}s remaining of {}s total)", progress, remaining, total);
        } else {
            return false; // UI dropped
        }
        true
    };
    
    // Run work timer
    if !run_phase(ui_handle, work_seconds, update_ms, update_progress) {
        return;
    }
    println!("Work timer completed!");
    
    // Reset for play timer
    if let Some(ui) = ui_handle.upgrade() {
        ui.set_remainingTime(100); // Start full for play timer
    } else {
        return;
    }
    
    // Run play timer
    if !run_phase(ui_handle, play_seconds, update_ms, update_progress) {
        return;
    }
    println!("Play timer completed!");
    
    // Reset at end
    if let Some(ui) = ui_handle.upgrade() {
        ui.set_remainingTime(0);
    }
}

fn run_phase<F>(ui_handle: &slint::Weak<AppWindow>, total_seconds: u32, update_ms: u64, mut update_fn: F) -> bool 
where F: FnMut(u32, u32) -> bool {
    let mut elapsed = 0;
    
    // Initial update to show full progress bar
    if !update_fn(0, total_seconds) {
        return false;
    }
    
    while elapsed < total_seconds {
        std::thread::sleep(std::time::Duration::from_millis(update_ms));
        elapsed += update_ms as u32 / 1000;
        
        if !update_fn(elapsed, total_seconds) {
            return false; // UI was dropped
        }
    }
    true
}
