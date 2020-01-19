pub fn format_int(num: i32) -> String {
    format!("{}", num)
}

pub fn format_float(num: f32) -> String {
    format!("{}", num.to_bits())
}

pub fn format_string(s: &str) -> String {
    format!("\"{}\"", s)
}
