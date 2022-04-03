pub trait Empty {}

pub trait Simple {
    fn act();
}

pub trait AssociatedConst {
    const Flag: bool;
}

pub trait AssociatedType {
    type Type;
}
