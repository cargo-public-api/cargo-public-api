#[non_exhaustive]
pub struct AStruct {
    #[cfg(feature = "feature_a")]
    pub feature_a: (),
    #[cfg(feature = "feature_b")]
    pub feature_b: (),
    #[cfg(feature = "feature_c")]
    pub feature_c: (),
}
