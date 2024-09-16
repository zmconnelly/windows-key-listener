use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
};

mod key_chord_parser;
use key_chord_parser::KeyChordParser;

type Callback = Arc<dyn Fn() + Send + Sync + 'static>;

struct HookData {
    callback: Callback,
    vk_codes: Vec<i32>,
    interval: Duration,
    key_states: Arc<RwLock<HashMap<i32, bool>>>,
    should_block: bool,
    last_trigger: Arc<RwLock<Instant>>,
    hook_handle: Arc<RwLock<Option<isize>>>,
}

lazy_static! {
    static ref HOOKS: Arc<RwLock<Vec<HookData>>> = Arc::new(RwLock::new(Vec::new()));
}

unsafe extern "system" fn keyboard_hook(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code == HC_ACTION as i32 {
        let kb_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode as i32;

        let key_state = if w_param.0 as usize == WM_KEYDOWN as usize {
            true
        } else if w_param.0 as usize == WM_KEYUP as usize {
            false
        } else {
            return CallNextHookEx(None, n_code, w_param, l_param);
        };

        let mut should_block = false;

        let hooks = HOOKS.read().unwrap();
        for hook_data in hooks.iter() {
            if hook_data.vk_codes.contains(&vk_code) {
                let mut key_states = hook_data.key_states.write().unwrap();

                key_states.insert(vk_code, key_state);

                // Check if all keys in the chord are currently pressed
                let all_keys_pressed = hook_data
                    .vk_codes
                    .iter()
                    .all(|&code| *key_states.get(&code).unwrap_or(&false));

                if all_keys_pressed && key_state {
                    let mut last_trigger = hook_data.last_trigger.write().unwrap();
                    let now = Instant::now();
                    if now.duration_since(*last_trigger) >= hook_data.interval {
                        *last_trigger = now;
                        let callback_clone = Arc::clone(&hook_data.callback);
                        thread::spawn(move || {
                            (callback_clone)();
                        });
                    }
                }
                should_block = hook_data.should_block;
            }
        }

        if should_block {
            return LRESULT(1); // Non-zero return blocks event propagation
        }
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

pub struct KeyListener {
    parser: KeyChordParser,
}

impl KeyListener {
    pub fn new() -> Self {
        KeyListener {
            parser: KeyChordParser::new(),
        }
    }

    pub fn listen(
        &self,
        key_chord: &str,
        should_block: bool,
        interval: Duration,
        callback: Callback,
    ) {
        let key_chord = key_chord.to_string();

        if let Some(vk_codes) = self.parser.parse(&key_chord) {
            let key_states: Arc<RwLock<HashMap<i32, bool>>> = Arc::new(RwLock::new(
                vk_codes.iter().map(|&vk| (vk, false)).collect(),
            ));

            thread::spawn(move || {
                unsafe {
                    let h_instance = GetModuleHandleW(None).unwrap_or_default();
                    let hook =
                        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), h_instance, 0)
                            .unwrap();

                    if hook.is_invalid() {
                        println!("Failed to set hook for key chord: {}", key_chord);
                        return;
                    }

                    HOOKS.write().unwrap().push(HookData {
                        callback,
                        vk_codes,
                        interval,
                        key_states,
                        should_block,
                        last_trigger: Arc::new(RwLock::new(Instant::now())),
                        hook_handle: Arc::new(RwLock::new(Some(hook.0 as isize))),
                    });

                    // Keep the thread alive to process messages
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {}

                    // Unhook when done
                    UnhookWindowsHookEx(hook).unwrap();
                }
            });
        } else {
            println!("Failed to parse key chord: {}", key_chord);
        }
    }

    pub fn unlisten(&self) {
        let mut hooks = HOOKS.write().unwrap();
        hooks.iter_mut().for_each(|hook| {
            if let Some(hhk) = hook.hook_handle.write().unwrap().take() {
                unsafe { UnhookWindowsHookEx(HHOOK(hhk as *mut _)).unwrap() };
            }
        });
    }
}
