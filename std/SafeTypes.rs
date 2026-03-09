//#ARROW_IGNORE
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SafeVector {
    content: Vec<Value>,
}

impl SafeVector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    // Безопасный геттер (без паники)
    pub fn get_value(&self, index: u64) -> Value {
        self.content.get(index as usize).cloned().unwrap_or(Value::Bool(false))
    }

    // Безопасный сеттер (только замена в рамках лимита)
    pub fn set_value(&mut self, index: u64, val: Value) -> Result<(), &'static str> {
        let idx = index as usize;
        if idx >= self.content.len() {
            return Err("Index out of SafeVector limit");
        }
        self.content[idx] = val;
        Ok(())
    }

    pub fn push(&mut self, val: Value) {
        self.content.push(val);
    }
}

// --- РЕАЛИЗАЦИЯ СТАНДАРТНЫХ ТРЕЙТОВ ---

// 1. Вывод (Display)
impl fmt::Display for SafeVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, val) in self.content.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{:?}", val)?;
        }
        write!(f, "]")
    }
}

// 2. Чтение по индексу: let x = safe_vec[i];
impl Index<usize> for SafeVector {
    type Output = Value;
    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

// 3. Запись по индексу: safe_vec[i] = val;
impl IndexMut<usize> for SafeVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.content[index]
    }
}

// 4. Создание из обычного вектора: SafeVector::from(vec![...])
impl From<Vec<Value>> for SafeVector {
    fn from(v: Vec<Value>) -> Self {
        Self { content: v }
    }
}

// 5. Позволяет использовать .collect() для сборки в SafeVector
impl FromIterator<Value> for SafeVector {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Self { content: iter.into_iter().collect() }
    }
}

// 6. Позволяет использовать вектор в цикле for x in safe_vec (потребление)
impl IntoIterator for SafeVector {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

// 7. Позволяет итерироваться по ссылке: for x in &safe_vec
impl<'a> IntoIterator for &'a SafeVector {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.iter()
    }
}

// 8. Позволяет расширять вектор: safe_vec.extend(другой_итератор)
impl Extend<Value> for SafeVector {
    fn extend<I: IntoIterator<Item = Value>>(&mut self, iter: I) {
        self.content.extend(iter);
    }
}

// 9. Оператор сложения: v3 = v1 + v2;
impl Add<SafeVector> for SafeVector {
    type Output = Self;
    fn add(mut self, rhs: SafeVector) -> Self {
        self.content.extend(rhs.content);
        self
    }
}

// 10. Оператор добавления: v1 += Value::U8(1);
impl AddAssign<Value> for SafeVector {
    fn add_assign(&mut self, rhs: Value) {
        self.push(rhs);
    }
}

// 11. Представление в виде слайса (дает доступ к методам .sort(), .contains() и т.д.)
impl AsRef<[Value]> for SafeVector {
    fn as_ref(&self) -> &[Value] {
        &self.content
    }
}

#[derive(Debug, Clone, Default)]
pub struct SafeDict {
    keys: Vec<Value>,
    values: Vec<Value>,
}

impl SafeDict {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    pub fn add(&mut self, key: Value, value: Value) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn get(&self, key: &Value) -> ValueRes {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                return ValueRes { value: self.values[i].clone(), status: OpStatus::Success };
            }
        }
        ValueRes { value: Value::Bool(false), status: OpStatus::IndexError }
    }

    pub fn set(&mut self, key: &Value, value: Value) -> OpStatus {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                self.values[i] = value;
                return OpStatus::Success;
            }
        }
        OpStatus::IndexError
    }

    pub fn contains_key(&self, key: &Value) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        for i in (0..self.keys.len()).rev() {
            if &self.keys[i] == key {
                let value = self.values.remove(i);
                self.keys.remove(i);
                return Some(value);
            }
        }
        None
    }

    pub fn keys(&self) -> impl Iterator<Item = &Value> {
        self.keys.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.values.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.keys.iter().zip(self.values.iter())
    }

    // Безопасная замена содержимого с проверкой длины
    pub fn replace_contents(&mut self, new_keys: Vec<Value>, new_values: Vec<Value>) -> Result<(), &'static str> {
        if new_keys.len() != new_values.len() {
            return Err("keys and values length mismatch");
        }
        self.keys = new_keys;
        self.values = new_values;
        Ok(())
    }

    // Небезопасные методы (могут нарушить инвариант) — оставлены для особых случаев
    pub unsafe fn set_keys(&mut self, new_keys: Vec<Value>) {
        self.keys = new_keys;
    }

    pub unsafe fn set_values(&mut self, new_values: Vec<Value>) {
        self.values = new_values;
    }
}

// 1. Создание из вектора пар
impl From<Vec<(Value, Value)>> for SafeDict {
    fn from(pairs: Vec<(Value, Value)>) -> Self {
        let (keys, values) = pairs.into_iter().unzip();
        Self { keys, values }
    }
}

// 2. Сборка из итератора (для .collect())
impl FromIterator<(Value, Value)> for SafeDict {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(iter: I) -> Self {
        let (keys, values) = iter.into_iter().unzip();
        Self { keys, values }
    }
}

// 3. Расширение словаря другим итератором
impl Extend<(Value, Value)> for SafeDict {
    fn extend<I: IntoIterator<Item = (Value, Value)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.add(k, v);
        }
    }
}

// 4. Преобразование в итератор (потребление)
impl IntoIterator for SafeDict {
    type Item = (Value, Value);
    type IntoIter = std::iter::Zip<std::vec::IntoIter<Value>, std::vec::IntoIter<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().zip(self.values.into_iter())
    }
}

// 5. Итератор по ссылкам
impl<'a> IntoIterator for &'a SafeDict {
    type Item = (&'a Value, &'a Value);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, Value>, std::slice::Iter<'a, Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter().zip(self.values.iter())
    }
}

// 6. Отображение (Display)
impl std::fmt::Display for SafeDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, (k, v)) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}: {:?}", k, v)?;
        }
        write!(f, "}}")
    }
}
//#ARROW_NO_IGNORE