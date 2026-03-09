#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct u1024(pub [u64; 16]);

impl u1024 {
    pub const ZERO: Self = u1024([0; 16]);
    pub const ONE: Self = { let mut d = [0; 16]; d[0] = 1; u1024(d) };

    pub fn is_zero(&self) -> bool { self.0.iter().all(|&x| x == 0) }

    // Сложение с переносом
    pub fn carrying_add(self, rhs: Self) -> (Self, bool) {
        let mut res = [0u64; 16];
        let mut carry = false;
        for i in 0..16 {
            let (s1, c1) = self.0[i].overflowing_add(rhs.0[i]);
            let (s2, c2) = s1.overflowing_add(carry as u64);
            res[i] = s2;
            carry = c1 || c2;
        }
        (u1024(res), carry)
    }

    // Вычитание с заемом
    pub fn borrowing_sub(self, rhs: Self) -> (Self, bool) {
        let mut res = [0u64; 16];
        let mut borrow = false;
        for i in 0..16 {
            let (s1, b1) = self.0[i].overflowing_sub(rhs.0[i]);
            let (s2, b2) = s1.overflowing_sub(borrow as u64);
            res[i] = s2;
            borrow = b1 || b2;
        }
        (u1024(res), borrow)
    }

    // Умножение "в столбик" (O(n^2))
    pub fn full_mul(self, rhs: Self) -> Self {
        let mut res = [0u64; 16];
        for i in 0..16 {
            let mut carry = 0u128;
            for j in 0..(16 - i) {
                let prod = (self.0[i] as u128) * (rhs.0[j] as u128) + (res[i + j] as u128) + carry;
                res[i + j] = prod as u64;
                carry = prod >> 64;
            }
        }
        u1024(res)
    }

    // Быстрое деление и остаток (Binary Long Division)
    pub fn div_rem(self, rhs: Self) -> (Self, Self) {
        if rhs.is_zero() { panic!("Division by zero"); }
        let mut quotient = Self::ZERO;
        let mut remainder = Self::ZERO;
        for i in (0..1024).rev() {
            remainder = (remainder << 1) | u1024::from_bit(self.get_bit(i));
            if remainder >= rhs {
                remainder = remainder.borrowing_sub(rhs).0;
                quotient = quotient | (u1024::ONE << i);
            }
        }
        (quotient, remainder)
    }
    pub fn trailing_zeros(&self) -> u32 {
        let mut count = 0;
        for &word in &self.0 {
            if word == 0 {
                count += 64;
            } else {
                count += word.trailing_zeros();
                break;
            }
        }
        count
    }

    fn get_bit(&self, i: u32) -> bool { (self.0[(i / 64) as usize] >> (i % 64)) & 1 == 1 }
    fn from_bit(bit: bool) -> Self { if bit { Self::ONE } else { Self::ZERO } }
}

// Реализация сдвигов для u1024 (аналогично предыдущему ответу)
impl std::ops::Shl<u32> for u1024 {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self {
        let mut res = [0u64; 16];
        let word = (rhs / 64) as usize;
        let bit = rhs % 64;
        for i in word..16 {
            res[i] = self.0[i - word] << bit;
            if bit > 0 && i > word { res[i] |= self.0[i - word - 1] >> (64 - bit); }
        }
        u1024(res)
    }
}
impl std::ops::BitOr for u1024 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        let mut r = [0u64; 16];
        for i in 0..16 { r[i] = self.0[i] | rhs.0[i]; }
        u1024(r)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct i1024(pub u1024);

impl i1024 {
    pub fn is_negative(&self) -> bool { (self.0 .0[15] >> 63) == 1 }

    pub fn abs(self) -> (u1024, bool) {
        if self.is_negative() { ((-self).0, true) } 
        else { (self.0, false) }
    }
}
impl std::ops::Shr<u32> for u1024 {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self {
        let mut res = [0u64; 16];
        let word_shift = (rhs / 64) as usize;
        let bit_shift = rhs % 64;
        for i in 0..(16 - word_shift) {
            res[i] = self.0[i + word_shift] >> bit_shift;
            if bit_shift > 0 && i + word_shift + 1 < 16 {
                res[i] |= self.0[i + word_shift + 1] << (64 - bit_shift);
            }
        }
        u1024(res)
    }
}
impl std::ops::Neg for i1024 {
    type Output = Self;
    fn neg(self) -> Self {
        // Инверсия бит + 1
        let mut res = [0u64; 16];
        for i in 0..16 { res[i] = !self.0.0[i]; }
        i1024(u1024(res).carrying_add(u1024::ONE).0)
    }
}

impl std::ops::Add for i1024 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { i1024(self.0.carrying_add(rhs.0).0) }
}

impl std::ops::Sub for i1024 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { self + (-rhs) }
}

impl std::ops::Mul for i1024 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let (a, s1) = self.abs();
        let (b, s2) = rhs.abs();
        let res = i1024(a.full_mul(b));
        if s1 ^ s2 { -res } else { res }
    }
}

impl std::ops::Div for i1024 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let (a, s1) = self.abs();
        let (b, s2) = rhs.abs();
        let res = i1024(a.div_rem(b).0);
        if s1 ^ s2 { -res } else { res }
    }
}

impl std::ops::Rem for i1024 {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        let (a, s1) = self.abs();
        let (b, _) = rhs.abs();
        let res = i1024(a.div_rem(b).1);
        if s1 { -res } else { res }
    }
}

pub fn gcd(a: i64, b: i64) -> u64 {
    // Используем unsigned_abs(), чтобы безопасно обработать i64::MIN
    let mut u = a.unsigned_abs();
    let mut v = b.unsigned_abs();

    // Базовые случаи
    if u == 0 { return v; }
    if v == 0 { return u; }

    // Находим количество общих двоек
    let common_twos = (u | v).trailing_zeros();

    // Делаем u нечетным
    u >>= u.trailing_zeros();

    while v != 0 {
        // Убираем двойки из v
        v >>= v.trailing_zeros();

        // Упорядочиваем, чтобы u всегда было меньше или равно v
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }

        // Вычитаем (v гарантированно >= u, так что u64 не уйдет в минус)
        v -= u;
    }

    // Сдвигаем результат обратно на накопленные двойки
    u << common_twos
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct fraction{
    pub num: i1024,
    pub den: u1024,
}

impl fraction {
    pub fn reduce(&mut self) {
        if self.num.0.is_zero() {
            self.den = u1024::ONE;
            return;
        }
        
        let mut u = self.num.abs().0;
        let mut v = self.den;

        let common_twos = (u | v).trailing_zeros();
        u = u >> u.trailing_zeros();

        while !v.is_zero() {
            v = v >> v.trailing_zeros();
            if u > v { std::mem::swap(&mut u, &mut v); }
            v = v.borrowing_sub(u).0;
        }

        let gcd = u << common_twos;

        // Сокращаем числитель и знаменатель
        // Используем ваши div_rem (берем только частное .0)
        self.num = i1024(self.num.abs().0.div_rem(gcd).0);
        if (self.num.0.0[15] >> 63) == 1 { /* обработка знака если нужно */ }
        self.den = self.den.div_rem(gcd).0;
    }
}
// --- Реализация вывода для u1024 (нужна для Display дроби) ---
impl fmt::Display for u1024 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() { return write!(f, "0"); }
        let mut res = String::new();
        let mut temp = *self;
        let ten = u1024([10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        while !temp.is_zero() {
            let (q, r) = temp.div_rem(ten);
            res.push(std::char::from_digit(r.0[0] as u32, 10).unwrap());
            temp = q;
        }
        write!(f, "{}", res.chars().rev().collect::<String>())
    }
}

impl fmt::Display for i1024 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (abs_val, is_neg) = self.abs();
        if is_neg { write!(f, "-")?; }
        write!(f, "{}", abs_val)
    }
}

// --- Трейты для fraction ---

impl fmt::Display for fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut copy = *self;
        copy.reduce();

        let (abs_num, is_neg) = copy.num.abs();
        let (whole, rem) = abs_num.div_rem(copy.den);

        if is_neg { write!(f, "-")?; }

        if rem.is_zero() {
            write!(f, "{}", whole)
        } else if whole.is_zero() {
            write!(f, "{}/{}", rem, copy.den)
        } else {
            // Смешанное число: Целое(Остаток/Знаменатель)
            write!(f, "{}({}/{})", whole, rem, copy.den)
        }
    }
}

impl std::ops::Add for fraction {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        // a/b + c/d = (ad + bc) / bd
        let ad = self.num * i1024(rhs.den);
        let bc = rhs.num * i1024(self.den);
        let mut res = fraction {
            num: ad + bc,
            den: self.den.full_mul(rhs.den),
        };
        res.reduce();
        res
    }
}

impl std::ops::Sub for fraction {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + fraction { num: -rhs.num, den: rhs.den }
    }
}

impl std::ops::Mul for fraction {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut res = fraction {
            num: self.num * rhs.num,
            den: self.den.full_mul(rhs.den),
        };
        res.reduce();
        res
    }
}

impl std::ops::Div for fraction {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.num.0.is_zero() { panic!("Division by zero in fraction"); }
        let (abs_num, is_neg) = rhs.num.abs();
        let mut res = fraction {
            num: self.num * i1024(rhs.den),
            den: self.den.full_mul(abs_num),
        };
        if is_neg { res.num = -res.num; }
        res.reduce();
        res
    }
}

// Позволяет делать: let f = fraction::new(5, 10); // станет 1/2
impl fraction {
    pub fn new(num: i1024, den: u1024) -> Self {
        if den.is_zero() { panic!("Denominator is zero"); }
        let mut f = fraction { num, den };
        f.reduce();
        f
    }
}

