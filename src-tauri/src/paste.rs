use std::thread;
use std::time::Duration;

/// 激活目标窗口并模拟粘贴（Cmd+V / Ctrl+V）
/// keywords: 窗口匹配关键词列表
pub fn activate_and_paste(keywords: &[String]) -> Result<(), String> {
    let owner = find_target_owner(keywords)?;
    activate_window(&owner)?;
    thread::sleep(Duration::from_millis(500));
    simulate_paste(&owner)?;
    Ok(())
}

// ──────────────────────────── 查找目标窗口所有者 ────────────────────────────

fn find_target_owner(keywords: &[String]) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        find_owner_macos(keywords)
    }
    #[cfg(target_os = "windows")]
    {
        find_owner_windows(keywords)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = keywords;
        Err("不支持的平台".to_string())
    }
}

// ──────────────────────────── 激活窗口 ────────────────────────────

fn activate_window(owner: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        activate_window_macos(owner)
    }
    #[cfg(target_os = "windows")]
    {
        activate_window_windows(owner)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = owner;
        Err("不支持的平台".to_string())
    }
}

// ──────────────────────────── 模拟粘贴 ────────────────────────────

fn simulate_paste(owner: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        simulate_paste_macos(owner)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = owner;
        simulate_paste_windows()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = owner;
        Err("不支持的平台".to_string())
    }
}

// ──────────────────────────── macOS 实现 ────────────────────────────

#[cfg(target_os = "macos")]
fn find_owner_macos(keywords: &[String]) -> Result<String, String> {
    use core_foundation::base::TCFType;
    use core_foundation::string::CFString;
    use std::ffi::c_void;

    extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, window_id: u32) -> *const c_void;
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, idx: isize) -> *const c_void;
        fn CFDictionaryGetValue(dict: *const c_void, key: *const c_void) -> *const c_void;
        fn CFRelease(cf: *const c_void);
    }

    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;

    unsafe {
        let list = CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, 0);
        if list.is_null() {
            return Err("无法获取窗口列表".to_string());
        }

        let count = CFArrayGetCount(list);
        let key_owner = CFString::new("kCGWindowOwnerName");
        let key_name = CFString::new("kCGWindowName");

        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list, i);
            if dict.is_null() {
                continue;
            }

            let owner_ptr =
                CFDictionaryGetValue(dict, key_owner.as_concrete_TypeRef() as *const c_void);
            let owner = if !owner_ptr.is_null() {
                let s = CFString::wrap_under_get_rule(owner_ptr as _);
                s.to_string()
            } else {
                continue;
            };

            let name_ptr =
                CFDictionaryGetValue(dict, key_name.as_concrete_TypeRef() as *const c_void);
            let name = if !name_ptr.is_null() {
                let s = CFString::wrap_under_get_rule(name_ptr as _);
                s.to_string()
            } else {
                String::new()
            };

            let matched = keywords.iter().any(|keyword| {
                (!name.is_empty() && name.contains(keyword)) || owner.contains(keyword)
            });

            if matched {
                CFRelease(list);
                return Ok(owner);
            }
        }

        CFRelease(list);
        Err("未找到目标窗口，请先打开闲鱼/闲管家".to_string())
    }
}

#[cfg(target_os = "macos")]
fn activate_window_macos(owner: &str) -> Result<(), String> {
    use std::process::Command;

    // 使用 AppleScript 激活指定应用
    let script = format!(
        r#"tell application "System Events"
    set targetProc to first process whose name is "{}"
    set frontmost of targetProc to true
end tell"#,
        owner.replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("执行 AppleScript 失败: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("激活窗口失败: {}", err));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn simulate_paste_macos(_owner: &str) -> Result<(), String> {
    use std::ffi::c_void;

    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
        fn CGEventCreateKeyboardEvent(
            source: *const c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
        fn CFRelease(cf: *const c_void);
    }

    // 先检查辅助功能权限
    unsafe {
        if !AXIsProcessTrusted() {
            return Err("请在「系统设置 → 隐私与安全性 → 辅助功能」中授权本程序（开发模式需添加 target/debug/catering-calculator）".to_string());
        }
    }

    const SOURCE_STATE_COMBINED: i32 = 0;
    const HID_EVENT_TAP: u32 = 0;
    const FLAG_MASK_COMMAND: u64 = 0x100000;
    const VK_ANSI_V: u16 = 9;

    unsafe {
        let source = CGEventSourceCreate(SOURCE_STATE_COMBINED);
        if source.is_null() {
            return Err("无法创建事件源".to_string());
        }

        let key_down = CGEventCreateKeyboardEvent(source, VK_ANSI_V, true);
        if key_down.is_null() {
            CFRelease(source);
            return Err("无法创建按键事件".to_string());
        }
        CGEventSetFlags(key_down, FLAG_MASK_COMMAND);
        CGEventPost(HID_EVENT_TAP, key_down);

        // 短暂延迟确保按键事件被处理
        thread::sleep(Duration::from_millis(50));

        let key_up = CGEventCreateKeyboardEvent(source, VK_ANSI_V, false);
        if !key_up.is_null() {
            CGEventSetFlags(key_up, FLAG_MASK_COMMAND);
            CGEventPost(HID_EVENT_TAP, key_up);
            CFRelease(key_up);
        }

        CFRelease(key_down);
        CFRelease(source);
    }

    Ok(())
}

// ──────────────────────────── Windows 实现 ────────────────────────────

#[cfg(target_os = "windows")]
fn find_owner_windows(keywords: &[String]) -> Result<String, String> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    struct SearchContext {
        keywords: Vec<String>,
        result: Option<String>,
        hwnd: Option<HWND>,
    }

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut SearchContext);
        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return TRUE;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, &mut buf);
        if read == 0 {
            return TRUE;
        }
        let title = String::from_utf16_lossy(&buf[..read as usize]).to_lowercase();
        let matched = ctx
            .keywords
            .iter()
            .any(|kw| title.contains(&kw.to_lowercase()));
        if matched {
            ctx.result = Some(title);
            ctx.hwnd = Some(hwnd);
            return BOOL(0);
        }
        TRUE
    }

    let mut ctx = SearchContext {
        keywords: keywords.to_vec(),
        result: None,
        hwnd: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut ctx as *mut SearchContext as isize),
        );
    }

    match ctx.result {
        Some(title) => Ok(title),
        None => Err("未找到目标窗口，请先打开闲鱼/闲管家".to_string()),
    }
}

#[cfg(target_os = "windows")]
static mut FOUND_HWND: Option<windows::Win32::Foundation::HWND> = None;

#[cfg(target_os = "windows")]
fn activate_window_windows(owner: &str) -> Result<(), String> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
        SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    unsafe extern "system" fn find_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let keyword = &*(lparam.0 as *const String);
        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return TRUE;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, &mut buf);
        if read == 0 {
            return TRUE;
        }
        let title = String::from_utf16_lossy(&buf[..read as usize]).to_lowercase();
        if title.contains(&keyword.to_lowercase()) {
            FOUND_HWND = Some(hwnd);
            return BOOL(0);
        }
        TRUE
    }

    let keyword = owner.to_string();
    unsafe {
        FOUND_HWND = None;
        let _ = EnumWindows(
            Some(find_callback),
            LPARAM(&keyword as *const String as isize),
        );
        if let Some(hwnd) = FOUND_HWND {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
            Ok(())
        } else {
            Err("无法激活目标窗口".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
fn simulate_paste_windows() -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
    };

    unsafe {
        let inputs = [
            // Ctrl down
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // V down
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_V,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // V up
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_V,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Ctrl up
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != 4 {
            return Err("模拟粘贴按键失败".to_string());
        }
    }
    Ok(())
}
