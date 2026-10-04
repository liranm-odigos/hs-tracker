//! Key output. On Windows this is one `SendInput` of scan codes. Everywhere
//! else the sender only records presses so the rotation can be tested.

use crate::keys::KeyCombo;

pub trait KeySender {
    fn key_down(&mut self, combo: &KeyCombo) -> Result<(), String>;
    fn key_up(&mut self, combo: &KeyCombo) -> Result<(), String>;
}

#[derive(Default)]
pub struct RecordingSender {
    pub events: Vec<(String, bool)>,
}

impl KeySender for RecordingSender {
    fn key_down(&mut self, combo: &KeyCombo) -> Result<(), String> {
        self.events.push((combo.label.clone(), true));
        Ok(())
    }

    fn key_up(&mut self, combo: &KeyCombo) -> Result<(), String> {
        self.events.push((combo.label.clone(), false));
        Ok(())
    }
}

#[cfg(windows)]
mod win {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};
    use std::time::Instant;

    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_ABOVE_NORMAL,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, RegisterHotKey, SendInput, UnregisterHotKey, HOT_KEY_MODIFIERS, INPUT,
        INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
        VK_CONTROL, VK_MENU, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetMessageW, GetWindowTextW, PostThreadMessageW, WM_HOTKEY, WM_QUIT,
    };

    use super::KeySender;
    use crate::keys::KeyCombo;
    use crate::rotation::{Engine, Step};
    use crate::timing::wait_until;

    static STOP: AtomicBool = AtomicBool::new(false);

    unsafe extern "system" fn console_handler(_ctrl: u32) -> i32 {
        STOP.store(true, Ordering::Release);
        1
    }

    struct ConsoleHandler;

    impl ConsoleHandler {
        fn install() -> Self {
            unsafe {
                SetConsoleCtrlHandler(Some(console_handler), 1);
            }
            Self
        }
    }

    impl Drop for ConsoleHandler {
        fn drop(&mut self) {
            unsafe {
                SetConsoleCtrlHandler(Some(console_handler), 0);
            }
        }
    }

    struct TimerResolution;

    impl TimerResolution {
        fn acquire() -> Self {
            unsafe {
                timeBeginPeriod(1);
            }
            Self
        }
    }

    impl Drop for TimerResolution {
        fn drop(&mut self) {
            unsafe {
                timeEndPeriod(1);
            }
        }
    }

    struct WindowsSender;

    impl KeySender for WindowsSender {
        fn key_down(&mut self, combo: &KeyCombo) -> Result<(), String> {
            send_combo(combo, true)
        }

        fn key_up(&mut self, combo: &KeyCombo) -> Result<(), String> {
            send_combo(combo, false)
        }
    }

    pub fn run_live(
        toggle: &KeyCombo,
        cancel: &KeyCombo,
        build: &crate::rotation::Build,
        hold: std::time::Duration,
        focus: Option<&str>,
        verbose: bool,
    ) -> Result<(), String> {
        STOP.store(false, Ordering::Release);
        let _timer = TimerResolution::acquire();
        let _console = ConsoleHandler::install();
        unsafe {
            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);
        }

        let running = Arc::new(AtomicBool::new(false));
        let hotkeys = Hotkeys::install(toggle, cancel, Arc::clone(&running))?;
        let mut sender = WindowsSender;
        let mut engine = Engine::new(build.clone(), Instant::now());
        let mut active = false;

        let stop = || STOP.load(Ordering::Acquire);
        while !stop() {
            let want = running.load(Ordering::Acquire);
            if want != active {
                active = want;
                if active {
                    println!("rotation on — {}", engine.build().name);
                } else {
                    println!("rotation off");
                }
            }
            if !active {
                if wait_until(Instant::now() + std::time::Duration::from_millis(5), stop) {
                    break;
                }
                continue;
            }
            if !foreground_matches(focus) {
                if wait_until(Instant::now() + std::time::Duration::from_millis(20), stop) {
                    break;
                }
                continue;
            }

            let now = Instant::now();
            let index = match engine.poll(now) {
                Step::Press { index } => index,
                Step::Wait { until } => {
                    let slice = if until <= now {
                        now + std::time::Duration::from_millis(1)
                    } else {
                        until.min(now + std::time::Duration::from_millis(5))
                    };
                    wait_until(slice, || stop() || !running.load(Ordering::Acquire));
                    continue;
                }
            };

            let combo = engine.build().skills[index].key.clone();
            let pressed_at = Instant::now();
            if let Err(err) = sender.key_down(&combo) {
                eprintln!("aion-assist: {err}");
                let _ = sender.key_up(&combo);
                running.store(false, Ordering::Release);
                continue;
            }
            let mut held = Release {
                sender: &mut sender,
                combo: &combo,
                armed: true,
            };
            wait_until(pressed_at + hold, || {
                stop() || !running.load(Ordering::Acquire)
            });
            // Release before the cooldown is recorded so the next press cannot
            // overlap this one, and before any logging which can block.
            held.release();
            engine.commit(index, pressed_at);
            if verbose {
                let skill = &engine.build().skills[index];
                println!("{}  [{}]", skill.name, skill.key);
            }
        }

        hotkeys.shutdown();
        println!("stopped");
        Ok(())
    }

    /// Releases the key if the press is abandoned early.
    struct Release<'a> {
        sender: &'a mut WindowsSender,
        combo: &'a KeyCombo,
        armed: bool,
    }

    impl Release<'_> {
        fn release(&mut self) {
            if self.armed {
                self.armed = false;
                if let Err(err) = self.sender.key_up(self.combo) {
                    eprintln!("aion-assist: {err}");
                }
            }
        }
    }

    impl Drop for Release<'_> {
        fn drop(&mut self) {
            self.release();
        }
    }

    struct Hotkeys {
        thread_id: u32,
        handle: Option<JoinHandle<()>>,
    }

    impl Hotkeys {
        fn install(
            toggle: &KeyCombo,
            cancel: &KeyCombo,
            running: Arc<AtomicBool>,
        ) -> Result<Self, String> {
            let toggle = toggle.clone();
            let cancel = cancel.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            let handle = thread::spawn(move || {
                let thread_id = unsafe { GetCurrentThreadId() };
                let registered = unsafe { register(&toggle, &cancel) };
                let ready = registered.is_ok();
                if tx.send((thread_id, registered)).is_err() {
                    if ready {
                        unsafe { release_hotkeys() }
                    }
                    return;
                }
                if ready {
                    message_loop(running);
                    unsafe { release_hotkeys() }
                }
            });

            let (thread_id, registered) = match rx.recv() {
                Ok(pair) => pair,
                Err(_) => {
                    let _ = handle.join();
                    return Err("hotkey thread stopped before the keys were registered".into());
                }
            };
            if let Err(err) = registered {
                let _ = handle.join();
                return Err(err);
            }
            Ok(Self {
                thread_id,
                handle: Some(handle),
            })
        }

        fn shutdown(mut self) {
            if let Some(handle) = self.handle.take() {
                unsafe {
                    PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
                }
                let _ = handle.join();
            }
        }
    }

    unsafe fn register(toggle: &KeyCombo, cancel: &KeyCombo) -> Result<(), String> {
        if RegisterHotKey(std::ptr::null_mut(), 1, modifiers(toggle), toggle.vk as u32) == 0 {
            return Err(hotkey_error("toggle", toggle));
        }
        if RegisterHotKey(std::ptr::null_mut(), 2, modifiers(cancel), cancel.vk as u32) == 0 {
            let err = hotkey_error("cancel", cancel);
            UnregisterHotKey(std::ptr::null_mut(), 1);
            return Err(err);
        }
        Ok(())
    }

    unsafe fn release_hotkeys() {
        UnregisterHotKey(std::ptr::null_mut(), 1);
        UnregisterHotKey(std::ptr::null_mut(), 2);
    }

    fn modifiers(combo: &KeyCombo) -> HOT_KEY_MODIFIERS {
        let mut flags = MOD_NOREPEAT;
        if combo.shift {
            flags |= MOD_SHIFT;
        }
        if combo.ctrl {
            flags |= MOD_CONTROL;
        }
        if combo.alt {
            flags |= MOD_ALT;
        }
        flags
    }

    fn hotkey_error(which: &str, combo: &KeyCombo) -> String {
        let code = unsafe { GetLastError() };
        format!(
            "could not register the {which} key {} (Windows error {code}). Choose a key the system is not already using.",
            combo.label
        )
    }

    fn message_loop(running: Arc<AtomicBool>) {
        unsafe {
            loop {
                let mut msg = std::mem::zeroed();
                let status = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if status == 0 || status == -1 {
                    break;
                }
                if msg.message == WM_HOTKEY {
                    match msg.wParam {
                        1 => {
                            running.fetch_xor(true, Ordering::AcqRel);
                        }
                        2 => {
                            STOP.store(true, Ordering::Release);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn foreground_matches(focus: Option<&str>) -> bool {
        let Some(needle) = focus else {
            return true;
        };
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_null() {
            return false;
        }
        let mut buffer = [0u16; 256];
        let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if len <= 0 {
            return false;
        }
        let title = String::from_utf16_lossy(&buffer[..len as usize]);
        title
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
    }

    fn send_combo(combo: &KeyCombo, down: bool) -> Result<(), String> {
        let mut keys = Vec::with_capacity(4);
        if down {
            push_modifiers(&mut keys, combo);
            keys.push((combo.vk, combo.extended));
        } else {
            keys.push((combo.vk, combo.extended));
            let start = keys.len();
            push_modifiers(&mut keys, combo);
            keys[start..].reverse();
        }
        let inputs: Vec<INPUT> = keys
            .into_iter()
            .map(|(vk, extended)| key_event(vk, down, extended))
            .collect();
        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            )
        };
        if sent == inputs.len() as u32 {
            Ok(())
        } else {
            let code = unsafe { GetLastError() };
            Err(format!(
                "SendInput delivered {sent} of {} events (Windows error {code})",
                inputs.len()
            ))
        }
    }

    fn push_modifiers(keys: &mut Vec<(u16, bool)>, combo: &KeyCombo) {
        if combo.ctrl {
            keys.push((VK_CONTROL, false));
        }
        if combo.alt {
            keys.push((VK_MENU, false));
        }
        if combo.shift {
            keys.push((VK_SHIFT, false));
        }
    }

    fn key_event(vk: u16, down: bool, extended: bool) -> INPUT {
        let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
        let (w_vk, w_scan, mut flags) = if scan == 0 {
            let flags = if down { 0 } else { KEYEVENTF_KEYUP };
            (vk, 0, flags)
        } else {
            (0, scan, KEYEVENTF_SCANCODE)
        };
        if scan != 0 && !down {
            flags |= KEYEVENTF_KEYUP;
        }
        if scan != 0 && extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: w_vk,
                    wScan: w_scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

#[cfg(windows)]
pub use win::run_live;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::parse;

    #[test]
    fn records_a_press_as_down_then_up() {
        let combo = parse("Shift+1").unwrap();
        let mut sender = RecordingSender::default();
        sender.key_down(&combo).unwrap();
        sender.key_up(&combo).unwrap();
        assert_eq!(
            sender.events,
            vec![("Shift+1".into(), true), ("Shift+1".into(), false)]
        );
    }
}
