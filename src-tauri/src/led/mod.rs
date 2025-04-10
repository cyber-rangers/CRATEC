use serialport::SerialPort;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use once_cell::sync::Lazy;

// Globální instance LED kontroléru dostupná z celé aplikace
pub static LED_CONTROLLER: Lazy<LedController> = Lazy::new(|| {
    LedController::new("/dev/ttyS0", 9600)
});

pub struct LedController {
    port_name: String,
    baud_rate: u32,
    active_processes: Arc<Mutex<u32>>,
    thread_running: Arc<Mutex<bool>>,
}

impl LedController {
    pub fn new(port_name: &str, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.to_string(),
            baud_rate,
            active_processes: Arc::new(Mutex::new(0)),
            thread_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Notifikuje o započetí nového procesu
    /// Tato metoda by měla být volána pokaždé, když začíná nový proces
    pub fn notify_process_start(&self) {
        let mut active_processes = self.active_processes.lock().unwrap();
        *active_processes += 1;
        println!("LED: Aktivní procesy: {}", *active_processes);

        // Spustíme blikání, pouze pokud ještě neběží
        if *active_processes == 1 {
            self.start_blinking();
        }
    }

    /// Notifikuje o ukončení procesu
    /// Tato metoda by měla být volána pokaždé, když končí proces
    pub fn notify_process_end(&self) {
        let mut active_processes = self.active_processes.lock().unwrap();
        if *active_processes > 0 {
            *active_processes -= 1;
            println!("LED: Aktivní procesy: {}", *active_processes);
            
            // Zastavíme blikání, pokud není žádný aktivní proces
            if *active_processes == 0 {
                self.stop_blinking();
            }
        }
    }

    // Private metoda pro spuštění blikání v samostatném vlákně
    fn start_blinking(&self) {
        let mut thread_running = self.thread_running.lock().unwrap();
        if *thread_running {
            return; // Vlákno již běží
        }
        *thread_running = true;
        
        let port_name = self.port_name.clone();
        let baud_rate = self.baud_rate;
        let active_processes = Arc::clone(&self.active_processes);
        let thread_running = Arc::clone(&self.thread_running);

        thread::spawn(move || {
            println!("LED: Spuštění blikání");
            match serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_millis(10))
                .open() 
            {
                Ok(mut port) => {
                    let steps = 100;
                    let period_ms = 20;

                    while *active_processes.lock().unwrap() > 0 {
                        // Fade in
                        for i in 0..steps {
                            let duty = i as f32 / steps as f32;
                            pwm_led(&mut port, duty, period_ms);
                            
                            // Kontrola, zda máme pokračovat
                            if *active_processes.lock().unwrap() == 0 {
                                break;
                            }
                        }
                        
                        // Kontrola po fade in
                        if *active_processes.lock().unwrap() == 0 {
                            break;
                        }

                        // Fade out
                        for i in (0..steps).rev() {
                            let duty = i as f32 / steps as f32;
                            pwm_led(&mut port, duty, period_ms);
                            
                            // Kontrola, zda máme pokračovat
                            if *active_processes.lock().unwrap() == 0 {
                                break;
                            }
                        }
                    }
                    
                    // Zajistíme vypnutí LED na konci
                    set_rts(&mut port, true);
                }
                Err(e) => {
                    eprintln!("LED: Nepodařilo se otevřít port: {}", e);
                }
            }
            
            println!("LED: Ukončení blikání");
            *thread_running.lock().unwrap() = false;
        });
    }

    fn stop_blinking(&self) {
        // Nic nemusíme dělat, blikání se zastaví samo 
        // když počet procesů klesne na 0
        println!("LED: Zastavuji blikání");
    }
}

fn pwm_led(port: &mut Box<dyn SerialPort>, duty: f32, period_ms: u64) {
    let on_time = (duty * period_ms as f32) as u64;
    let off_time = period_ms - on_time;

    set_rts(port, false); // RTS = false -> 3.3V -> LED ON
    thread::sleep(Duration::from_millis(on_time));
    set_rts(port, true); // RTS = true -> 0V -> LED OFF
    thread::sleep(Duration::from_millis(off_time));
}

fn set_rts(port: &mut Box<dyn SerialPort>, state: bool) {
    port.write_request_to_send(state).unwrap();
}