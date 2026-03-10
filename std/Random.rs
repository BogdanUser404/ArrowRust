//#ARROW_IGNORE
use std::time::{SystemTime, UNIX_EPOCH};
use std::mem::MaybeUninit;

pub fn random_int(min: u64, max: u64) -> u64 {
	if min >= max {
		return min;
	}

	// 1. Получаем UnixTime (наносекунды)
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_nanos() as u64;

	// 2. Извлекаем 2 числа из "грязного" стека (UB!)
	let [seed, salt]: [u64; 2] = unsafe {
		let mut data = MaybeUninit::<[u64; 2]>::uninit();
		data.assume_init()
	};

	// 3. Смешиваем энтропию (алгоритм на базе SplitMix64)
	let mut x = nanos.wrapping_add(seed);
	x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
	let raw_random = x ^ (x >> 31) ^ salt;

	// 4. Приводим к диапазону
	let range = max - min + 1;
	min + (raw_random % range)
}

fn random_char() -> char {
	// 1. Получаем энтропию (время + мусор + адрес) (UB!)
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_nanos() as u64;
	let [seed, salt]: [u64; 2] = unsafe { MaybeUninit::uninit().assume_init() };
	let stack_ptr = &nanos as *const u64 as u64;

	// 2. Смешиваем биты
	let mut x = nanos.wrapping_add(seed).wrapping_add(stack_ptr);
	x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
	let raw_val = x ^ (x >> 31) ^ salt;

	// 3. Превращаем в валидный UTF-32 (Unicode Scalar Value)
	let unicode_range = 0x110000 - 0x800; // Весь диапазон минус суррогаты
	let mut code_point = (raw_val % unicode_range) as u32;

	// Пропускаем зону суррогатов (D800-DFFF)
	if code_point >= 0xD800 {
		code_point += 0x800;
	}

	// Безопасно превращаем u32 в char (паника при невалидном)
	char::from_u32(code_point).unwrap_or('�')
}
//#ARROW_NO_IGNORE