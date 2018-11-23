
pub fn max_str_len(v: &Vec<String>) -> usize {
    v.iter().max_by(|text1, text2| {
        text1.len().cmp(&text2.len())
    }).unwrap_or(&String::new()).len()
}