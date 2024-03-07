pub fn strip_quotes(mut st: String) -> String {
    // suffix
    st = st
        .strip_suffix("\"")
        .map(|val| val.to_string())
        .unwrap_or(st);
    // prefix
    return st
        .strip_prefix("\"")
        .map(|val| val.to_string())
        .unwrap_or(st);
}
