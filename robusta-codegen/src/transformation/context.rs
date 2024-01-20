use crate::transformation::JavaPath;
use syn::{LifetimeParam, Path};

#[derive(Clone)]
pub(crate) struct StructContext {
    pub(crate) struct_type: Path,
    pub(crate) struct_name: String,
    pub(crate) struct_lifetimes: Vec<LifetimeParam>,
    pub(crate) package: Option<JavaPath>,
}
