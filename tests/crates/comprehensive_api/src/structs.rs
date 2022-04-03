pub struct Unit;

pub struct Plain {
    pub x: usize,
}

pub struct PrivateField {
    pub(crate) x: usize,
}

pub struct Tuple(pub usize);

pub struct WithLifetime<'a> {
    pub z: &'a Unit,
}

pub struct ConstArg<T, const N: usize> {
    pub items: [T; N],
}
