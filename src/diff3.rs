pub enum ConflictResolution {
    Ours,
    Theirs,
}

pub fn three_way_merge(
    ours: &str,
    common: &str,
    theirs: &str,
    strategy: Option<ConflictResolution>,
) -> String {
}
