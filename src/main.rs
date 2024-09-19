use std::collections::HashMap;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde_json::{json, Value};
use log::{info, warn, error};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use chrono::Utc;
use clap::{Command, Arg};

#[derive(Clone)]
struct VmInfo {
    socket_path: String,
    target_memory_mb: u64,
    current_memory_mb: u64,
    last_balanced: Instant,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let vm_infos = Arc::new(Mutex::new(parse_args()?));

    // Set up signal handling
    let running = Arc::new(Mutex::new(true));
    let r = running.clone();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
        for sig in signals.forever() {
            info!("Received signal {:?}", sig);
            *r.lock().unwrap() = false;
            break;
        }
    });

    // Spawn a thread for each VM
    let mut handles = vec![];
    for (vm_name, vm_info) in vm_infos.lock().unwrap().iter() {
        let vm_name = vm_name.clone();
        let vm_info = vm_info.clone();
        let vm_infos = Arc::clone(&vm_infos);
        let running = Arc::clone(&running);

        let handle = thread::spawn(move || {
            while *running.lock().unwrap() {
                match balance_memory(&vm_name, &vm_info, &vm_infos) {
                    Ok(_) => info!("Memory balanced successfully for VM: {}", vm_name),
                    Err(e) => error!("Error balancing memory for VM {}: {}", vm_name, e),
                }
                thread::sleep(Duration::from_secs(60));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    info!("Shutting down gracefully");
    Ok(())
}

fn parse_args() -> Result<HashMap<String, VmInfo>, Box<dyn std::error::Error>> {
    let matches = Command::new("Memory Balancer")
        .version("1.0")
        .author("Your Name")
        .about("Balances memory across VMs")
        .arg(Arg::new("vm_config")
            .help("VM configurations in the format: <vm_name> <qmp_socket_path> <target_memory_mb>")
            .num_args(3..)
            .required(true))
        .get_matches();

    let mut vm_infos = HashMap::new();
    let vm_configs: Vec<_> = matches.get_many::<String>("vm_config").unwrap().collect();
    
    for chunk in vm_configs.chunks(3) {
        let vm_name = chunk[0].to_string();
        let socket_path = chunk[1].to_string();
        let target_memory_mb = chunk[2].parse()?;
        vm_infos.insert(vm_name, VmInfo {
            socket_path,
            target_memory_mb,
            current_memory_mb: 0,
            last_balanced: Instant::now(),
        });
    }
    Ok(vm_infos)
}

fn balance_memory(vm_name: &str, vm_info: &VmInfo, vm_infos: &Arc<Mutex<HashMap<String, VmInfo>>>) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(&vm_info.socket_path)?;
    
    // QMP Handshake
    send_command(&mut stream, json!({"execute": "qmp_capabilities"}))?;
    
    // Get current memory info
    let memory_info = send_command(&mut stream, json!({"execute": "query-memory-size-summary"}))?;
    
    if let Some(actual_memory) = memory_info["return"]["base-memory"].as_u64() {
        let actual_memory_mb = actual_memory / (1024 * 1024);
        info!("VM: {}, Current memory: {} MB", vm_name, actual_memory_mb);
        
        // Calculate totals before locking vm_infos
        let (total_memory, total_target) = {
            let vm_infos = vm_infos.lock().unwrap();
            (
                vm_infos.values().map(|v| v.current_memory_mb).sum::<u64>(),
                vm_infos.values().map(|v| v.target_memory_mb).sum::<u64>()
            )
        };
        
        let mut vm_infos = vm_infos.lock().unwrap();
        let vm_info = vm_infos.get_mut(vm_name).unwrap();
        vm_info.current_memory_mb = actual_memory_mb;

        let (new_target, should_adjust) = if total_memory > total_target {
            let excess = total_memory - total_target;
            let share = excess * actual_memory_mb / total_memory;
            let new_target = actual_memory_mb - share;
            (new_target, new_target < vm_info.target_memory_mb)
        } else if actual_memory_mb < vm_info.target_memory_mb && 
                  vm_info.last_balanced.elapsed() > Duration::from_secs(300) {
            (vm_info.target_memory_mb, true)
        } else {
            (actual_memory_mb, false)
        };

        if should_adjust {
            info!("VM: {}, Adjusting memory to {} MB", vm_name, new_target);
            send_command(&mut stream, json!({
                "execute": "balloon",
                "arguments": {"value": new_target * 1024 * 1024}
            }))?;
            vm_info.last_balanced = Instant::now();
        }

        // Log metrics
        let timestamp = Utc::now().to_rfc3339();
        info!("METRIC,timestamp={},vm={},current_memory={},target_memory={}", 
              timestamp, vm_name, vm_info.current_memory_mb, vm_info.target_memory_mb);
    } else {
        warn!("VM: {}, Failed to get current memory info", vm_name);
    }

    Ok(())
}

fn send_command(stream: &mut UnixStream, command: Value) -> std::io::Result<Value> {
    let command_str = format!("{}\n", serde_json::to_string(&command)?);
    stream.write_all(command_str.as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    
    Ok(serde_json::from_str(&response)?)
}
