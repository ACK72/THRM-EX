use serde::{Serialize, Deserialize};
use std::{
	env, thread, process, time::Duration
};
use windows::{
	core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*
};

const TITLE: PCWSTR = w!("명일방주");

static mut KBDHOOK: HHOOK = HHOOK(0);
static mut MSEHOOK: HHOOK = HHOOK(0);

static mut POS: (i32, i32) = (0, 0);
static mut LAST: DIRECTION = DIRECTION::NONE;
static mut STATE: bool = true;

static mut GP: u32 = 0;
static mut GS: u32 = 0;

static mut OQ: u32 = 0;
static mut OS: u32 = 0;
static mut OD: u32 = 0;

static mut PU: u32 = 0;
static mut PD: u32 = 0;
static mut PL: u32 = 0;
static mut PR: u32 = 0;

static mut MP: u32 = 0;
static mut MQ: u32 = 0;

#[derive(PartialEq, Eq)]
enum DIRECTION {
	UP,
	DOWN,
	LEFT,
	RIGHT,
	NONE
}

#[derive(Serialize, Deserialize)]
struct Config {
	message: String,
	site_link: String,

	game_pause: u32,
	game_speed: u32,

	operator_retreat: u32,
	operator_skill: u32,
	operator_deselect: u32,

	position_up: u32,
	position_down: u32,
	position_left: u32,
	position_right: u32,

	macro_pause: u32,
	macro_quit: u32
}

impl ::std::default::Default for Config {
    fn default() -> Self {
		Self {
			message: String::from("Find the HEX value of your key in the site below and replace it with a \"decimal\" value."),
			site_link: String::from("https://learn.microsoft.com/ko-kr/windows/win32/inputdev/virtual-key-codes"),

			game_pause: 0x20, // VK_SPACE
			game_speed: 0x43, // VK_KEY_C

			operator_retreat: 0x51, // VK_KEY_Q
			operator_skill: 0x45, // VK_KEY_E
			operator_deselect: 0x46, // VK_KEY_F

			position_up: 0x57, // VK_KEY_W
			position_down: 0x53, // VK_KEY_S
			position_left: 0x41, // VK_KEY_A
			position_right: 0x44, // VK_KEY_D

			macro_pause: 0x72, // VK_F3
			macro_quit: 0x73 // VK_F4
		}
	}
}

fn main() {
	let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
	path.push("thrm_ex.cfg");

	let config: Config = confy::load_path(path).unwrap();

	unsafe {
		GP = config.game_pause;
		GS = config.game_speed;
		
		OQ = config.operator_retreat;
		OS = config.operator_skill;
		OD = config.operator_deselect;

		PU = config.position_up;
		PD = config.position_down;
		PL = config.position_left;
		PR = config.position_right;

		MP = config.macro_pause;
		MQ = config.macro_quit;
	}

	thread::spawn(|| {
		loop {
			let hwnd = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
			if hwnd == HWND(0) {
				unsafe {
					if KBDHOOK != HHOOK(0) {
						UnhookWindowsHookEx(KBDHOOK).unwrap();
					}
					if MSEHOOK != HHOOK(0) {
						UnhookWindowsHookEx(MSEHOOK).unwrap();
					}
				}
				process::exit(0);
			}

			thread::sleep(Duration::from_secs(3));
		}
	});

	unsafe {
		MSEHOOK = SetWindowsHookExA(WH_MOUSE_LL, Some(mouse_callback), HINSTANCE(0), 0).unwrap();
		KBDHOOK = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_callback), HINSTANCE(0), 0).unwrap();
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

	unsafe {
		UnhookWindowsHookEx(KBDHOOK).unwrap();
		UnhookWindowsHookEx(MSEHOOK).unwrap();
	};
}

extern "system" fn keyboard_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	let mut click = false;
	unsafe {
		if code >= 0 {
			if w_param == WPARAM(WM_KEYDOWN as usize) {
				let key_struct :KBDLLHOOKSTRUCT = *(l_param.0 as *const KBDLLHOOKSTRUCT);

				click = match key_struct.vkCode {
					a if a == GP => STATE && input_tap(0.9375, 0.075),
					a if a == GS => STATE && input_tap(0.85, 0.075),

					a if a == OQ => STATE && input_tap(0.47, 0.27),
					a if a == OS => STATE && input_tap(0.675, 0.55),
					a if a == OD => STATE && clear_tap() && input_tap(0.95, 0.35),

					a if a == PU => STATE && input_swipe(POS, DIRECTION::UP, false),
					a if a == PD => STATE && input_swipe(POS, DIRECTION::DOWN, false),
					a if a == PL => STATE && input_swipe(POS, DIRECTION::LEFT, false),
					a if a == PR => STATE && input_swipe(POS, DIRECTION::RIGHT, false),

					a if a == MP => change_state(),
					a if a == MQ => terminate_hook(),
					_ => false
				};
			} else if w_param == WPARAM(WM_KEYUP as usize) {
				let key_struct :KBDLLHOOKSTRUCT = *(l_param.0 as *const KBDLLHOOKSTRUCT);

				click = match key_struct.vkCode {
					a if a == PU => STATE && input_swipe(POS, DIRECTION::UP, true),
					a if a == PD => STATE && input_swipe(POS, DIRECTION::DOWN, true),
					a if a == PL => STATE && input_swipe(POS, DIRECTION::LEFT, true),
					a if a == PR => STATE && input_swipe(POS, DIRECTION::RIGHT, true),
					_ => false
				};
			}
		}

		if click {
			LRESULT(1)
		} else {
			CallNextHookEx(KBDHOOK, code, w_param, l_param)
		}
	}
}

extern "system" fn mouse_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	unsafe {
		if code >= 0 {
			let mouse_struct :MOUSEHOOKSTRUCT = *(l_param.0 as *const MOUSEHOOKSTRUCT);
			let (x, y) = get_mouse_info(mouse_struct.pt.x, mouse_struct.pt.y);

			if x >= 0 && y >= 0 {
				if w_param == WPARAM(WM_LBUTTONDOWN as usize) {
					POS = (-1, -2);
				} else if w_param == WPARAM(WM_MOUSEMOVE as usize) {
					if POS == (-1, -2) || POS == (-1, -1) {
						POS = (-1, -1);
					}
				} else if w_param == WPARAM(WM_LBUTTONUP as usize) {
					POS = if POS == (-1, -1) {
						(x, y)
					} else {
						(-1, -3)
					};
				}
			}
		}

		CallNextHookEx(MSEHOOK, code, w_param, l_param)
	}
}

fn get_gpg_info() -> (HWND, i32, i32, i32, i32) {
	let hwnd = unsafe { FindWindowW(PCWSTR::null(), TITLE) };
	let fwnd = unsafe { GetForegroundWindow() };

	if hwnd == HWND(0) || hwnd != fwnd {
		return (HWND(0), 0, 0, 0, 0);
	}

	let mut window_rect = RECT::default();
	let _ = unsafe { GetWindowRect(hwnd, &mut window_rect) };

	let mut client_rect = RECT::default();
	let _ = unsafe { GetClientRect(hwnd, &mut client_rect) };

	let mut point = POINT { x: window_rect.left, y: window_rect.top };
	let _ = unsafe { ScreenToClient(hwnd, &mut point) };

	let width = client_rect.right - client_rect.left;
	let height = client_rect.bottom - client_rect.top;

	(hwnd, width, height, point.x, point.y)
}

fn get_mouse_info(x: i32, y: i32) -> (i32, i32) {
	let (hwnd, width, height, _, _) = get_gpg_info();

	if hwnd == HWND(0) {
		return (-1, -1);
	}
	
	let mut point = POINT { x, y };
	let _ = unsafe { ScreenToClient(hwnd, &mut point) };

	if point.x > width || point.y > height {
		return (-1, -1);
	}

	(point.x, point.y)
}

fn get_relative_point(rx: f32, ry: f32, w: i32, h: i32, ax: i32, ay: i32) -> isize {
	let nx = ((rx * w as f32) as i32 - ax) as isize;
	let ny = ((ry * h as f32) as i32 - ay) as isize;

	ny << 16 | nx
}

fn input_tap(rx:f32, ry:f32) -> bool {
	let (hwnd, width, height, ax, ay) = get_gpg_info();

	if hwnd == HWND(0) {
		return false;
	}

	let pos = get_relative_point(rx, ry, width, height, ax, ay);

	unsafe {
		let _ = PostMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos));
		let _ = PostMessageA(hwnd, WM_LBUTTONUP, WPARAM(1), LPARAM(pos));
	}

	true
}

fn clear_tap() -> bool {
	let (hwnd, width, _, ax, ay) = get_gpg_info();

	if hwnd == HWND(0) {
		return false;
	}

	if unsafe { LAST == DIRECTION::NONE } {
		return true;
	}
	
	let (x, y) = unsafe { POS };
	let pos = (y - ay) << 16 | x - ax + (width as f32 * 0.01) as i32; // (width * 0.01) is dummy value, same position is not working

	let _ = unsafe { PostMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos as isize)) };
	spin_sleep::sleep(Duration::new(0, 50 * 1000000));

	let _ = unsafe { PostMessageA(hwnd, WM_LBUTTONUP, WPARAM(1), LPARAM(pos as isize)) };
	unsafe { LAST = DIRECTION::NONE };

	true
}

fn input_swipe(position: (i32, i32), direction: DIRECTION, is_up: bool) -> bool {
	let (hwnd, width, height, ax, ay) = get_gpg_info();

	if hwnd == HWND(0) {
		return false;
	}

	let (x, y) = position;
	if x < 0 || y < 0 {
		return true;
	}

	let x = x - ax;
	let y = y - ay;

	let (dx, dy) = match direction {
		DIRECTION::UP => (0, (-0.2 * height as f32) as i32),
		DIRECTION::DOWN => (0, (0.2 * height as f32) as i32),
		DIRECTION::LEFT => ((-0.2 * width as f32) as i32, 0),
		DIRECTION::RIGHT => ((0.2 * width as f32) as i32, 0),
		_ => return true
	};

	if is_up {
		if unsafe { LAST == direction } {
			unsafe { LAST = DIRECTION::NONE };
		} else {
			return true;
		}

		let pos = (y + dy) << 16 | (x + dx);
		let _ = unsafe { PostMessageA(hwnd, WM_LBUTTONUP, WPARAM(1), LPARAM(pos as isize)) };
	} else {
		if unsafe { LAST == direction } {
			return true;
		} else {
			unsafe { LAST = direction };
		}
		
		let pos = y << 16 | x;
		let _ = unsafe { PostMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos as isize)) };
		spin_sleep::sleep(Duration::new(0, 10 * 1000000));

		let pos = (y + dy) << 16 | (x + dx);
		let _ = unsafe { PostMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos as isize)) };
	}

	true
}

fn change_state() -> bool {
	let (hwnd, _, _, _, _) = get_gpg_info();

	if hwnd == HWND(0) {
		return false;
	}

	unsafe { STATE = !STATE };

	true
}

fn terminate_hook() -> bool {
	let (hwnd, _, _, _, _) = get_gpg_info();

	if hwnd == HWND(0) {
		return false;
	}

	unsafe { PostQuitMessage(0) };

	true
}