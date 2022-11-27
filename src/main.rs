use std::{
    thread, process, ffi::{c_void}, mem::size_of, time::Duration
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Dwm::*, Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*
};

static mut _TARGET: PCWSTR = w!("명일방주");
static mut _HOOK: HHOOK = HHOOK(0);
static mut _STATE: bool = true;

fn main() {
    thread::spawn(|| {
        loop {
            let hwnd = unsafe { FindWindowW(PCWSTR::null(), _TARGET) };
            if hwnd == HWND(0) {
                unsafe {
                    if _HOOK != HHOOK(0) {
                        UnhookWindowsHookEx(_HOOK);
                    }
                }
                process::exit(0);
            }

            thread::sleep(Duration::from_secs(3));
        }
    });

    unsafe {
        _HOOK = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), HINSTANCE(0), 0).unwrap();
    }

    let mut msg: MSG = MSG {
        hwnd : HWND(0),
        message : 0 as u32,
        wParam : WPARAM(0),
        lParam : LPARAM(0),
        time : 0 as u32,
        pt : POINT { x: 0, y: 0 },
    };

    loop {
        unsafe {
            let pm = GetMessageW(&mut msg, HWND(0), 0 as u32, 0 as u32);
            if pm == BOOL(0) {
                break;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe { UnhookWindowsHookEx(_HOOK) };
}

extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut click = false;
    unsafe {
        if code >= 0 && w_param == WPARAM(WM_KEYDOWN as usize) {
            let key_struct :KBDLLHOOKSTRUCT = *(l_param.0 as *const KBDLLHOOKSTRUCT);

            click = match key_struct.vkCode {
                0x20 => _STATE && relative_click(_TARGET, 0.9375, 0.075), // VK_SPACE
                0x43 => _STATE && relative_click(_TARGET, 0.85, 0.075), // VK_KEY_C
                0x51 => _STATE && relative_click(_TARGET, 0.47, 0.27), // VK_KEY_Q
                0x53 => _STATE && relative_click(_TARGET, 0.675, 0.55), // VK_KEY_S
                0x72 => change_state(_TARGET), //VK_F3
                0x73 => terminate_hook(_TARGET), //VK_F4
                _ => false
            }
        }

        if click {
            return LRESULT(1);
        } else {
            return CallNextHookEx(_HOOK, code, w_param, l_param);
        }
    }
}

fn relative_click(title:PCWSTR, ratio_x:f32, ratio_y:f32) -> bool {
    let frame = Box::new(RECT::default());
    let frame_ptr = Box::into_raw(frame);

    let handle = unsafe { FindWindowW(PCWSTR::null(), title) };
    let active = unsafe { GetForegroundWindow() };

    if handle == HWND(0) || handle != active {
        return false;
    }

    let _res = unsafe { DwmGetWindowAttribute(handle, DWMWA_EXTENDED_FRAME_BOUNDS, frame_ptr as *mut c_void, size_of::<RECT>() as u32) };
    let frame = unsafe { Box::from_raw(frame_ptr) };

    let mut minf = MONITORINFO::default();
    minf.cbSize = std::mem::size_of::<MONITORINFO>() as _;

    unsafe {
        let hmnt = MonitorFromWindow(handle, MONITOR_DEFAULTTOPRIMARY);
        let _res = GetMonitorInfoA(hmnt, &mut minf as *mut MONITORINFO);
    }

    let x = if minf.rcMonitor.left == frame.left && minf.rcMonitor.top == frame.top && minf.rcMonitor.right == frame.right && minf.rcMonitor.bottom == frame.bottom {
        ((ratio_x * (frame.right - frame.left) as f32).floor()) as isize
    } else {
        ((ratio_x * (frame.right - frame.left - 32) as f32).floor() + 31.0) as isize
    };
    let y = (ratio_y * (frame.bottom - frame.top) as f32).floor() as isize;

    let pos = y << 16 | x;

    unsafe {
        PostMessageA(handle, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos));
        PostMessageA(handle, WM_LBUTTONUP, WPARAM(0), LPARAM(pos));
    }

    return true;
}

fn change_state(title:PCWSTR) -> bool {
    let handle = unsafe { FindWindowW(PCWSTR::null(), title) };
    let active = unsafe { GetForegroundWindow() };

    if handle == HWND(0) || handle != active {
        return false;
    }

    unsafe {
        _STATE = !_STATE;
    }

    return true;
}

fn terminate_hook(title:PCWSTR) -> bool {
    let handle = unsafe { FindWindowW(PCWSTR::null(), title) };
    let active = unsafe { GetForegroundWindow() };

    if handle == HWND(0) || handle != active {
        return false;
    }

    unsafe {
        PostQuitMessage(0);
    }

    return true;
}