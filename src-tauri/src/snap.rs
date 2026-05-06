use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

// ──────────────────────────── 数据结构 ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapPosition {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapConfig {
    pub position: SnapPosition,
    pub target_keyword: String,
    pub target_keywords: Option<Vec<String>>,
    pub gap: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapStatus {
    pub enabled: bool,
    pub target_found: bool,
    pub target_title: Option<String>,
    pub position: String,
}

#[derive(Debug, Clone)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowRect {
    fn to_physical(&self, scale_factor: f64) -> Self {
        Self {
            x: (self.x as f64 * scale_factor).round() as i32,
            y: (self.y as f64 * scale_factor).round() as i32,
            width: (self.width as f64 * scale_factor).round() as i32,
            height: (self.height as f64 * scale_factor).round() as i32,
        }
    }
}

// ──────────────────────────── SnapManager ────────────────────────────

pub struct SnapManager {
    running: Arc<AtomicBool>,
    target_found: Arc<AtomicBool>,
    target_title: Arc<Mutex<Option<String>>>,
    config: Arc<Mutex<Option<SnapConfig>>>,
}

impl SnapManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            target_found: Arc::new(AtomicBool::new(false)),
            target_title: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, app: AppHandle, config: SnapConfig) {
        self.stop();

        let position_str = match &config.position {
            SnapPosition::Left => "Left",
            SnapPosition::Right => "Right",
            SnapPosition::Top => "Top",
            SnapPosition::Bottom => "Bottom",
        };
        log::info!(
            "开始吸附: 目标={:?}, 位置={}, 间距={}",
            config.keywords(), position_str, config.gap
        );

        *self.config.lock().unwrap() = Some(config.clone());
        self.running.store(true, Ordering::SeqCst);
        self.target_found.store(false, Ordering::SeqCst);

        let running = self.running.clone();
        let target_found = self.target_found.clone();
        let target_title = self.target_title.clone();

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                if let Some(window) = app.get_webview_window("main") {
                    match find_target_window(&config.keywords()) {
                        Some((rect, title)) => {
                            target_found.store(true, Ordering::SeqCst);
                            *target_title.lock().unwrap() = Some(title);

                            let my_size = match window.outer_size() {
                                Ok(size) => size,
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(50));
                                    continue;
                                }
                            };
                            let mw = my_size.width as i32;
                            let mh = my_size.height as i32;
                            let gap = config.gap;
                            let monitor_rect = match window.current_monitor() {
                                Ok(Some(monitor)) => {
                                    let pos = monitor.position();
                                    let size = monitor.size();
                                    WindowRect {
                                        x: pos.x,
                                        y: pos.y,
                                        width: size.width as i32,
                                        height: size.height as i32,
                                    }
                                }
                                _ => WindowRect {
                                    x: 0,
                                    y: 0,
                                    width: 1920,
                                    height: 1080,
                                },
                            };
                            let screen_left = monitor_rect.x;
                            let screen_top = monitor_rect.y;
                            let screen_right = monitor_rect.x + monitor_rect.width;
                            let screen_bottom = monitor_rect.y + monitor_rect.height;
                            let scale_factor = window.scale_factor().unwrap_or(1.0);
                            let target_rect = rect.to_physical(scale_factor);

                            let (mut nx, mut ny) = match config.position {
                                SnapPosition::Right => (target_rect.x + target_rect.width + gap, target_rect.y),
                                SnapPosition::Left => (target_rect.x - mw - gap, target_rect.y),
                                SnapPosition::Top => (target_rect.x, target_rect.y - mh - gap),
                                SnapPosition::Bottom => (target_rect.x, target_rect.y + target_rect.height + gap),
                            };
                            if matches!(config.position, SnapPosition::Right) && nx + mw > screen_right {
                                nx = target_rect.x - mw - gap;
                            }
                            if matches!(config.position, SnapPosition::Left) && nx < screen_left {
                                nx = target_rect.x + target_rect.width + gap;
                            }
                            if matches!(config.position, SnapPosition::Bottom) && ny + mh > screen_bottom {
                                ny = target_rect.y - mh - gap;
                            }
                            if matches!(config.position, SnapPosition::Top) && ny < screen_top {
                                ny = target_rect.y + target_rect.height + gap;
                            }
                            nx = nx.clamp(screen_left, screen_right - mw);
                            ny = ny.clamp(screen_top, screen_bottom - mh);

                            let _ = window.set_position(tauri::PhysicalPosition::new(nx, ny));
                        }
                        None => {
                            target_found.store(false, Ordering::SeqCst);
                            *target_title.lock().unwrap() = None;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.target_found.store(false, Ordering::SeqCst);
        *self.target_title.lock().unwrap() = None;
        *self.config.lock().unwrap() = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> SnapStatus {
        let position = self
            .config
            .lock()
            .unwrap()
            .as_ref()
            .map(|c| match c.position {
                SnapPosition::Left => "Left".to_string(),
                SnapPosition::Right => "Right".to_string(),
                SnapPosition::Top => "Top".to_string(),
                SnapPosition::Bottom => "Bottom".to_string(),
            })
            .unwrap_or_else(|| "Right".to_string());

        SnapStatus {
            enabled: self.is_running(),
            target_found: self.target_found.load(Ordering::SeqCst),
            target_title: self.target_title.lock().unwrap().clone(),
            position,
        }
    }
}

impl SnapConfig {
    fn keywords(&self) -> Vec<String> {
        match &self.target_keywords {
            Some(keywords) if !keywords.is_empty() => keywords
                .iter()
                .map(|keyword| keyword.trim())
                .filter(|keyword| !keyword.is_empty())
                .map(ToString::to_string)
                .collect(),
            _ => vec![self.target_keyword.clone()],
        }
    }
}

// ──────────────────────────── 跨平台窗口查找 ────────────────────────────

/// 查找包含指定关键词的目标窗口，返回其矩形和标题
fn find_target_window(keywords: &[String]) -> Option<(WindowRect, String)> {
    #[cfg(target_os = "windows")]
    {
        find_target_window_windows(keywords)
    }
    #[cfg(target_os = "macos")]
    {
        find_target_window_macos(keywords)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = keywords;
        None
    }
}

// ──────────────────────────── Windows 实现 ────────────────────────────

#[cfg(target_os = "windows")]
fn find_target_window_windows(keywords: &[String]) -> Option<(WindowRect, String)> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    struct SearchContext {
        keywords: Vec<String>,
        result: Option<(WindowRect, String)>,
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

        let title = String::from_utf16_lossy(&buf[..read as usize]);
        if ctx.keywords.iter().any(|keyword| title.contains(keyword)) {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                ctx.result = Some((
                    WindowRect {
                        x: rect.left,
                        y: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    },
                    title,
                ));
                return BOOL(0); // 停止枚举
            }
        }
        TRUE
    }

    let mut ctx = SearchContext {
        keywords: keywords.to_vec(),
        result: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut ctx as *mut SearchContext as isize),
        );
    }

    ctx.result
}

// ──────────────────────────── macOS 实现 ────────────────────────────

#[cfg(target_os = "macos")]
fn find_target_window_macos(keywords: &[String]) -> Option<(WindowRect, String)> {
    use core_foundation::base::TCFType;
    use core_foundation::string::CFString;
    use std::ffi::c_void;

    // Raw FFI declarations to avoid Rust wrapper trait bound issues
    extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, window_id: u32) -> *const c_void;
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, idx: isize) -> *const c_void;
        fn CFDictionaryGetValue(dict: *const c_void, key: *const c_void) -> *const c_void;
        fn CGRectMakeWithDictionaryRepresentation(
            dict: *const c_void,
            rect: *mut MacCGRect,
        ) -> bool;
        fn CFRelease(cf: *const c_void);
    }

    #[repr(C)]
    #[derive(Default)]
    struct MacCGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct MacCGSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct MacCGRect {
        origin: MacCGPoint,
        size: MacCGSize,
    }

    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;
    const K_CG_NULL_WINDOW_ID: u32 = 0;

    unsafe {
        let list = CGWindowListCopyWindowInfo(
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
            K_CG_NULL_WINDOW_ID,
        );
        if list.is_null() {
            return None;
        }

        let count = CFArrayGetCount(list);
        let key_owner = CFString::new("kCGWindowOwnerName");
        let key_name = CFString::new("kCGWindowName");
        let key_bounds = CFString::new("kCGWindowBounds");

        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list, i);
            if dict.is_null() {
                continue;
            }

            // 获取窗口所有者名称
            let owner_ptr =
                CFDictionaryGetValue(dict, key_owner.as_concrete_TypeRef() as *const c_void);
            let owner = if !owner_ptr.is_null() {
                let s = CFString::wrap_under_get_rule(owner_ptr as _);
                s.to_string()
            } else {
                String::new()
            };

            // 获取窗口名称
            let name_ptr =
                CFDictionaryGetValue(dict, key_name.as_concrete_TypeRef() as *const c_void);
            let name = if !name_ptr.is_null() {
                let s = CFString::wrap_under_get_rule(name_ptr as _);
                s.to_string()
            } else {
                String::new()
            };

            // 匹配关键词：窗口名称或所有者名称包含关键词
            let matched = keywords
                .iter()
                .any(|keyword| (!name.is_empty() && name.contains(keyword)) || owner.contains(keyword));
            if !matched {
                continue;
            }

            // 获取窗口边界
            let bounds_ptr =
                CFDictionaryGetValue(dict, key_bounds.as_concrete_TypeRef() as *const c_void);
            if bounds_ptr.is_null() {
                continue;
            }

            let mut rect = MacCGRect::default();
            if CGRectMakeWithDictionaryRepresentation(bounds_ptr, &mut rect) {
                let w = rect.size.width;
                let h = rect.size.height;

                // 过滤掉太小的窗口（菜单栏、状态栏图标等）
                if w < 100.0 || h < 100.0 {
                    continue;
                }

                let display_title = if !name.is_empty() {
                    format!("{} - {}", owner, name)
                } else {
                    owner.clone()
                };

                CFRelease(list);
                return Some((
                    WindowRect {
                        x: rect.origin.x as i32,
                        y: rect.origin.y as i32,
                        width: w as i32,
                        height: h as i32,
                    },
                    display_title,
                ));
            }
        }

        CFRelease(list);
        None
    }
}
