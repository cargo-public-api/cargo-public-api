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

pub trait TraitReferencingOwnAssociatedType {
    type OwnAssociatedType;

    fn own_associated_type_output(&self) -> Self::OwnAssociatedType;
    fn own_associated_type_output_explicit_as(
        &self,
    ) -> <Self as TraitReferencingOwnAssociatedType>::OwnAssociatedType;
}

// error[E0658]: associated type defaults are unstable
// skip for now
// pub trait AssociatedTypeDefault {
//     type Type = usize;
// }

pub unsafe trait UnsafeTrait {}
