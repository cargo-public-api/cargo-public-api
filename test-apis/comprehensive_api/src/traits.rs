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

// error[E0658]: associated type defaults are unstable
// skip for now
// pub trait AssociatedTypeDefault {
//     type Type = usize;
// }

pub unsafe trait UnsafeTrait {}
