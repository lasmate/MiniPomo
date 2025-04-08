#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::{Arc, Mutex}, time::{Duration, Instant}};
use slint::Timer;

slint::include_modules!();

#[derive(Clone, Copy, PartialEq)]
enum TimerState {
    Idle,
    Working,
    Playing,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    
    // Initialize timer state
    let timer_state = Arc::new(Mutex::new(TimerState::Idle));
    let start_time = Arc::new(Mutex::new(None::<Instant>));
    
    // Initialize work and play times
    let work_time = Arc::new(Mutex::new(25));
    let play_time = Arc::new(Mutex::new(5));
    
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

    // Initialize timer
    let ui_handle = ui.as_weak();
    let timer = Timer::default();
    
    let timer_state_clone = timer_state.clone();
    let start_time_clone = start_time.clone();
    let work_time_clone = work_time.clone();
    let play_time_clone = play_time.clone();
    
    // Update timer every 100ms
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100), 
        move || {
            let state = *timer_state_clone.lock().unwrap();
            
            // If timer is not running, do nothing
            if state == TimerState::Idle {
                return;
            }
            
            let Some(start) = *start_time_clone.lock().unwrap() else {
                return;
            };
            
            let elapsed = start.elapsed();
            
            match state {
                TimerState::Working => {
                    let work_mins = *work_time_clone.lock().unwrap();
                    let total_seconds = work_mins * 60;
                    let elapsed_seconds = elapsed.as_secs() as u32;
                    
                    if elapsed_seconds >= total_seconds {
                        // Work time completed, switch to play time
                        println!("Work timer completed! Switching to play timer.");
                        *timer_state_clone.lock().unwrap() = TimerState::Playing;
                        *start_time_clone.lock().unwrap() = Some(Instant::now());
                        
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_remainingTime(100); // Reset progress for play time
                        }
                        return;
                    }
                    
                    // Update progress bar
                    if let Some(ui) = ui_handle.upgrade() {
                        let remaining = total_seconds - elapsed_seconds;
                        let progress = (remaining as f32 / total_seconds as f32 * 100.0) as i32;
                        ui.set_remainingTime(progress);
                        
                        // Log progress every second
                        if elapsed_seconds % 5 == 0 || remaining <= 5 {
                            println!("Work: {}% ({}s remaining of {}s total)", 
                                progress, remaining, total_seconds);
                        }
                    }
                },
                TimerState::Playing => {
                    let play_mins = *play_time_clone.lock().unwrap();
                    let total_seconds = play_mins * 60;
                    let elapsed_seconds = elapsed.as_secs() as u32;
                    
                    if elapsed_seconds >= total_seconds {
                        // Play time completed, reset timer
                        println!("Play timer completed! Timer reset.");
                        *timer_state_clone.lock().unwrap() = TimerState::Idle;
                        *start_time_clone.lock().unwrap() = None;
                        
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_remainingTime(0); // Reset progress bar
                        }
                        return;
                    }
                    
                    // Update progress bar
                    if let Some(ui) = ui_handle.upgrade() {
                        let remaining = total_seconds - elapsed_seconds;
                        let progress = (remaining as f32 / total_seconds as f32 * 100.0) as i32;
                        ui.set_remainingTime(progress);
                        
                        // Log progress every second
                        if elapsed_seconds % 5 == 0 || remaining <= 5 {
                            println!("Play: {}% ({}s remaining of {}s total)", 
                                progress, remaining, total_seconds);
                        }
                    }
                },
                _ => {}
            }
        }
    );

    // Start timer callback
    let timer_state_clone = timer_state.clone();
    let start_time_clone = start_time.clone();
    ui.on_startTimer(move || {
        let mut state = timer_state_clone.lock().unwrap();
        
        if *state == TimerState::Idle {
            // Start work timer
            *state = TimerState::Working;
            *start_time_clone.lock().unwrap() = Some(Instant::now());
            println!("Timer started: Work phase");
        }
    });
    
    ui.run()?;
    Ok(())
}
