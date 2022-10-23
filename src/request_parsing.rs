// author - Patryk Jędrzejczak

use regex::Regex;

use super::TaskError;

// Returns true if there exists a prefix of a request parameter
// that is a correct STORE request.
pub fn is_store_request(request: &str) -> Result<bool, TaskError> {
    match Regex::new(r"^STORE\$[a-z]*\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(TaskError)
    }
}

// Returns true if there exists a prefix of a request parameter
// that is a correct LOAD request.
pub fn is_load_request(request: &str) -> Result<bool, TaskError> {
    match Regex::new(r"^LOAD\$[a-z]*\$") {
        Ok(store_regex) => Ok(store_regex.is_match(request)),
        Err(_) => Err(TaskError)
    }
}

// Splits a string with a prefix that is a correct STORE request
// from STORE$key$value$rest to (key, value, rest).
pub fn split_store_request(request: &str) -> (String, String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let value = request[dollars[1] + 1..dollars[2]].to_string();
    let rest = request[dollars[2] + 1..].to_string();
    (key, value, rest)
}

// Splits a string with a prefix that is a correct LOAD request
// from LOAD$key$rest to (key, rest).
pub fn split_load_request(request: &str) -> (String, String) {
    let dollars: Vec<usize> = request.match_indices('$').map(|(pos, _)| pos).collect();
    let key = request[dollars[0] + 1..dollars[1]].to_string();
    let rest = request[dollars[1] + 1..].to_string();
    (key, rest)
}