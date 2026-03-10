//#ARROW_IGNORE
/*
    ArrowRust Terminal Module
    Provides RGB colors, cursor manipulation, and screen control.
    Writing this at 4 AM is the only way to ensure it works.
*/

use std::io::{Write, stdout};

pub struct Terminal;

impl Terminal {
	// --- Управление экраном ---

	/// Очистить весь экран и вернуть курсор в 1,1
	pub fn clear() {
		print!("\x1B[2J\x1B[1;1H");
		let _ = stdout().flush();
	}

	/// Очистить строку, в которой стоит курсор
	pub fn clear_line() {
		print!("\x1B[2K");
		let _ = stdout().flush();
	}

	// --- Управление курсором ---

	/// Переместить курсор в X, Y (отсчет с 1)
	pub fn move_to(x: u32, y: u32) {
		print!("\x1B[{};{}H", y, x);
		let _ = stdout().flush();
	}

	/// Скрыть курсор (полезно для анимаций)
	pub fn hide_cursor() {
		print!("\x1B[?25l");
		let _ = stdout().flush();
	}

	/// Показать курсор
	pub fn show_cursor() {
		print!("\x1B[?25h");
		let _ = stdout().flush();
	}

	// --- RGB Цвета (TrueColor 24-bit) ---

	/// Установить цвет текста через RGB
	pub fn set_fg_rgb(r: u8, g: u8, b: u8) {
		print!("\x1B[38;2;{};{};{}m", r, g, b);
	}

	/// Установить цвет фона через RGB
	pub fn set_bg_rgb(r: u8, g: u8, b: u8) {
		print!("\x1B[48;2;{};{};{}m", r, g, b);
	}

	/// Сбросить все стили и цвета
	pub fn reset() {
		print!("\x1B[0m");
		let _ = stdout().flush();
	}

	// --- Вспомогательные функции (сахар) ---

	/// Напечатать цветной текст и сбросить цвет
	pub fn print_rgb(text: &str, r: u8, g: u8, b: u8) {
		Self::set_fg_rgb(r, g, b);
		print!("{}", text);
		Self::reset();
	}
}

// Трейт для "Персикового" конвейера
pub trait Colorable {
	fn color_rgb<T: fmt::Display>(self, r: u8, g: u8, b: u8) -> String;
}

impl Colorable for String {
	fn color_rgb(self, r: u8, g: u8, b: u8) -> String {
		format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, self)
	}
}

impl Colorable for &str {
	fn color_rgb(self, r: u8, g: u8, b: u8) -> String {
		format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, self)
	}
}
impl Colorable for SafeString {
	fn color_rgb(self, r: u8, g: u8, b: u8) -> String {
		format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, self)
	}
}
pub fn draw_box(x: u32, y: u32, w: u32, h: u32) {
	Terminal::move_to(x, y);
	print!("┌{:─^width$}┐", "", width = (w - 2) as usize);

	for i in 0..(h - 2) {
		Terminal::move_to(x, y + i + 1);
		print!("│{:width$}│", "", width = (w - 2) as usize);
	}

	Terminal::move_to(x, y + h - 1);
	print!("└{:─^width$}┘", "", width = (w - 2) as usize);
	let _ = std::io::stdout().flush();
}

//#ARROW_NO_IGNORE
