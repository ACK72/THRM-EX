use serde::{Serialize, Deserialize};
use std::{
	env, thread, process, time::Duration
};
use windows::{
	core::*, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*
};

const TITLE: PCWSTR = w!("명일방주");

static mut HOOK: HHOOK = HHOOK(0);
static mut STATE: bool = true;

static mut SP: u32 = 0x20; // VK_SPACE
static mut KC: u32 = 0x43; // VK_KEY_C
static mut KQ: u32 = 0x51; // VK_KEY_Q
static mut KX: u32 = 0x58; // VK_KEY_X
static mut KZ: u32 = 0x5A; // VK_KEY_Z
static mut F3: u32 = 0x72; // VK_F3
static mut F4: u32 = 0x73; // VK_F4

#[derive(Serialize, Deserialize)]
struct Config {
	message: String,
	site_link: String,
	game_pause: u32,
	game_speed: u32,
	operator_retreat: u32,
	operator_skill: u32,
	operator_deselect: u32,
	macro_pause: u32,
	macro_quit: u32
}

impl ::std::default::Default for Config {
    fn default() -> Self {
		Self {
			message: String::from("Find the HEX value of your key in the site below and replace it with a \"decimal\" value."),
			site_link: String::from("https://learn.microsoft.com/ko-kr/windows/win32/inputdev/virtual-key-codes"),
			game_pause: 0x20,
			game_speed: 0x43,
			operator_retreat: 0x51,
			operator_skill: 0x58,
			operator_deselect: 0x5A,
			macro_pause: 0x72,
			macro_quit: 0x73
		}
	}
}

fn main() {
	let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
	path.push("thrm_ex.cfg");

	let config: Config = confy::load_path(path).unwrap();

	unsafe {
		SP = config.game_pause;
		KC = config.game_speed;
		KQ = config.operator_retreat;
		KX = config.operator_skill;
		KZ = config.operator_deselect;
		F3 = config.macro_pause;
		F4 = config.macro_quit;
	}

	thread::spawn(|| {
		loop {
			let hwnd = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
			if hwnd == HWND(0) {
				unsafe {
					if HOOK != HHOOK(0) {
						UnhookWindowsHookEx(HOOK).unwrap();
					}
				}
				process::exit(0);
			}

			thread::sleep(Duration::from_secs(3));
		}
	});

	unsafe {
		HOOK = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), HINSTANCE(0), 0).unwrap();
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

	unsafe { UnhookWindowsHookEx(HOOK).unwrap(); };
}

extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	let mut click = false;
	unsafe {
		if code >= 0 && w_param == WPARAM(WM_KEYDOWN as usize) {
			let key_struct :KBDLLHOOKSTRUCT = *(l_param.0 as *const KBDLLHOOKSTRUCT);

			click = match key_struct.vkCode {
				a if a == SP => STATE && relative_click(0.9375, 0.075),
				a if a == KC => STATE && relative_click(0.85, 0.075),
				a if a == KQ => STATE && relative_click(0.47, 0.27),
				a if a == KX => STATE && relative_click(0.675, 0.55),
				a if a == KZ => STATE && relative_click(0.95, 0.35),
				a if a == F3 => change_state(),
				a if a == F4 => terminate_hook(),
				_ => false
			}
		}

		if click {
			return LRESULT(1);
		} else {
			return CallNextHookEx(HOOK, code, w_param, l_param);
		}
	}
}

fn relative_click(ratio_x:f32, ratio_y:f32) -> bool {
	let handle = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
	let active = unsafe { GetForegroundWindow() };

	if handle == HWND(0) || handle != active {
		return false;
	}

	let mut client_rect = RECT::default();
	let _ = unsafe { GetClientRect(handle, &mut client_rect) };

	let width = client_rect.right - client_rect.left;
	let height = client_rect.bottom - client_rect.top;

	let x = (width as f32 * ratio_x) as isize;
	let y = (height as f32 * ratio_y) as isize;

	let pos = y << 16 | x;

	unsafe {
		let _ = PostMessageA(handle, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos));
		let _ = PostMessageA(handle, WM_LBUTTONUP, WPARAM(1), LPARAM(pos));
	}

	return true;
}

fn change_state() -> bool {
	let handle = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
	let active = unsafe { GetForegroundWindow() };

	if handle == HWND(0) || handle != active {
		return false;
	}

	unsafe {
		STATE = !STATE;
	}

	return true;
}

fn terminate_hook() -> bool {
	let handle = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
	let active = unsafe { GetForegroundWindow() };

	if handle == HWND(0) || handle != active {
		return false;
	}

	unsafe {
		PostQuitMessage(0);
	}

	return true;
}