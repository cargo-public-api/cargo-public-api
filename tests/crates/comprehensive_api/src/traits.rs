pub trait Empty {}

pub trait Simple {
    fn act();
}

pub trait AssociatedConst {
    const CONST: bool;
}

pub trait AssociatedConstDefault {
    const CONST_WITH_DEFAULT: bool = true;
}

pub trait AssociatedType {
    type Type;
}
