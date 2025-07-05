use anyhow::Result;
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, InputEvent, EventType, KeyCode,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex as TokioMutex,
    task::JoinHandle,
    time::sleep,
};

#[tokio::main]
async fn main() -> Result<()> {
    let dev = Arc::new(Mutex::new(create_virtual_mouse()?));

    let device_path = find_touchpad_event().await.unwrap_or_else(|| {
        eprintln!("Touchpad device not found");
        std::process::exit(1);
    });
    let mut lines = spawn_libinput(&device_path).await?;

    let right_pressed = Arc::new(Mutex::new(false));
    let scrolling = Arc::new(Mutex::new(false));
    let last_scroll_time = Arc::new(Mutex::new(Instant::now()));
    let scroll_wait_task: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    println!("Start monitoring libinput events for right click hold...");

    while let Some(line) = lines.next_line().await? {
        if line.contains("GESTURE_HOLD_BEGIN") {
            handle_gesture_hold_begin(
                &line,
                &dev,
                &right_pressed,
                &scrolling,
                &scroll_wait_task,
            )
            .await?;
        } else if line.contains("GESTURE_HOLD_END") {
            handle_gesture_hold_end(
                &line,
                &dev,
                &right_pressed,
                &scrolling,
                &last_scroll_time,
                &scroll_wait_task,
            )
            .await?;
        } else if line.contains("POINTER_SCROLL_FINGER") {
            handle_pointer_scroll_finger(&scrolling, &last_scroll_time).await;
        } else{
            handle_scroll_terminated_by_other_gesture(
                &dev,
                &right_pressed,
                &scroll_wait_task,
            )?;


        }
    }

    Ok(())
}

fn create_virtual_mouse() -> Result<VirtualDevice> {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_RIGHT);

    let dev = VirtualDeviceBuilder::new()?
        .name("gesture_hold_rightclick")
        .with_keys(&keys)?
        .build()?;
    Ok(dev)
}

async fn spawn_libinput(
    device_path: &str,
) -> Result<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>> {
    let mut child = Command::new("libinput")
        .arg("debug-events")
        .arg("--device")
        .arg(device_path)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("No stdout");
    let reader = BufReader::new(stdout);
    Ok(reader.lines())
}

async fn handle_gesture_hold_begin(
    line: &str,
    dev: &Arc<Mutex<VirtualDevice>>,
    right_pressed: &Arc<Mutex<bool>>,
    scrolling: &Arc<Mutex<bool>>,
    scroll_wait_task: &Arc<Mutex<Option<JoinHandle<()>>>>,
) -> Result<()> {
    if let Some(finger_count) = extract_finger_count(line) {
        if finger_count == 2 {
            if let Some(handle) = scroll_wait_task.lock().unwrap().take() {
                handle.abort();
            }
            let mut pressed = right_pressed.lock().unwrap();
            if !*pressed {
                let mut dev_lock = dev.lock().unwrap();
                send_btn(&mut dev_lock, KeyCode::BTN_RIGHT, true)?;
                *pressed = true;
                println!("Right BTN down");
            }
            *scrolling.lock().unwrap() = false;
        }
    }
    Ok(())
}

async fn handle_gesture_hold_end(
    line: &str,
    dev: &Arc<Mutex<VirtualDevice>>,
    right_pressed: &Arc<Mutex<bool>>,
    scrolling: &Arc<Mutex<bool>>,
    last_scroll_time: &Arc<Mutex<Instant>>,
    scroll_wait_task: &Arc<Mutex<Option<JoinHandle<()>>>>,
) -> Result<()> {
    let cancelled = line.contains("cancelled");
    if let Some(finger_count) = extract_finger_count(line) {
        if finger_count == 2 {
            if cancelled {
                start_scroll_wait_task(
                    Arc::clone(dev),
                    Arc::clone(right_pressed),
                    Arc::clone(scrolling),
                    Arc::clone(last_scroll_time),
                    Arc::clone(scroll_wait_task),
                )
                .await;
            } else {
                if let Some(handle) = scroll_wait_task.lock().unwrap().take() {
                    handle.abort();
                }
                let mut pressed = right_pressed.lock().unwrap();
                if *pressed {
                    let mut dev_lock = dev.lock().unwrap();
                    send_btn(&mut dev_lock, KeyCode::BTN_RIGHT, false)?;
                    *pressed = false;
                    println!("Right BTN up");
                }
            }
        }
    }
    Ok(())
}
fn handle_scroll_terminated_by_other_gesture(
    dev: &Arc<Mutex<VirtualDevice>>,
    right_pressed: &Arc<Mutex<bool>>,
    scroll_wait_task: &Arc<Mutex<Option<JoinHandle<()>>>>,
) -> Result<()> {
    if let Some(handle) = scroll_wait_task.lock().unwrap().take() {
        handle.abort();
    }
    let mut pressed = right_pressed.lock().unwrap();
    if *pressed {
        let mut dev_lock = dev.lock().unwrap();
        send_btn(&mut dev_lock, KeyCode::BTN_RIGHT, false)?;
        *pressed = false;
        println!("Right BTN up due to other gesture");
    }
    Ok(())
}

async fn start_scroll_wait_task(
    dev: Arc<Mutex<VirtualDevice>>,
    right_pressed: Arc<Mutex<bool>>,
    scrolling: Arc<Mutex<bool>>,
    last_scroll_time: Arc<Mutex<Instant>>,
    scroll_wait_task: Arc<Mutex<Option<JoinHandle<()>>>>,
) {
    if let Some(handle) = scroll_wait_task.lock().unwrap().take() {
        handle.abort();
    }

    let dev_clone = Arc::clone(&dev);
    let right_pressed_clone = Arc::clone(&right_pressed);
    let scrolling_clone = Arc::clone(&scrolling);
    let last_scroll_time_clone = Arc::clone(&last_scroll_time);
    let scroll_wait_task_clone = Arc::clone(&scroll_wait_task);

    let handle = tokio::spawn(async move {
        loop {
            {
                let scrolling_now = *scrolling_clone.lock().unwrap();
                let last_time = *last_scroll_time_clone.lock().unwrap();
                if !scrolling_now && last_time.elapsed() > Duration::from_millis(300) {
                    let mut pressed = right_pressed_clone.lock().unwrap();
                    if *pressed {
                        let mut dev_lock = dev_clone.lock().unwrap();
                        if let Err(e) = send_btn(&mut dev_lock, KeyCode::BTN_RIGHT, false) {
                            eprintln!("Failed to send btn up: {e}");
                        } else {
                            println!("Right BTN up after scroll wait");
                        }
                        *pressed = false;
                    }
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        *scroll_wait_task_clone.lock().unwrap() = None;
    });

    *scroll_wait_task.lock().unwrap() = Some(handle);
}


async fn handle_pointer_scroll_finger(
    scrolling: &Arc<Mutex<bool>>,
    last_scroll_time: &Arc<Mutex<Instant>>,
) {
    *scrolling.lock().unwrap() = true;
    *last_scroll_time.lock().unwrap() = Instant::now();
}

fn extract_finger_count(line: &str) -> Option<u32> {
    line.trim().chars().last()?.to_digit(10)
}

fn send_btn(dev: &mut VirtualDevice, key: KeyCode, pressed: bool) -> Result<()> {
    let event = InputEvent::new(EventType::KEY.0, key.0, if pressed { 1 } else { 0 });
    dev.emit(&[event])?;
    Ok(())
}

async fn find_touchpad_event() -> Option<String> {
    let output = Command::new("libinput")
        .arg("list-devices")
        .output()
        .await
        .ok()?; 

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("Device:") && line.contains("Touchpad") {
            println!("{}",line);
            if let Some(next_line) = stdout.lines().skip_while(|l| *l != line).nth(1) {
                println!("{}",next_line);
                if let Some(path) = next_line.trim().strip_prefix("Kernel:") {
                    println!("{}",path);
                    return Some(path.trim().to_string());
                }
            }

        }
    }
    None
}

