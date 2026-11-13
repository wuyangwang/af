//! File type, glob, and ignore filtering.

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub hidden: bool,
    pub no_ignore: bool,
    pub file_type: Option<String>,
    pub glob: Option<String>,
    pub before: usize,
    pub after: usize,
    pub ignore_case: bool,
}
