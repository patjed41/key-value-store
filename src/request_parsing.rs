// author - Patryk Jędrzejczak

use regex::Regex;

use super::{TaskError, Request, StoreRequest, LoadRequest};

fn match_regex(message: &str, pattern: &str) -> Result<bool, TaskError> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.is_match(message)),
        Err(_) => Err(TaskError)
    }
}

// Returns true if there exists a prefix of a message parameter
// that is a correct STORE request.
fn is_store_request(message: &str) -> Result<bool, TaskError> {
    match_regex(message, r"^STORE\$[a-z]*\$[a-z]*\$")
}

// Returns true if there exists a prefix of a message parameter
// that is a correct LOAD request.
fn is_load_request(message: &str) -> Result<bool, TaskError> {
    match_regex(message, r"^LOAD\$[a-z]*\$")
}

// Returns true if message could become a correct STORE request.
fn could_become_store_request(message: &str) -> Result<bool, TaskError> {
    static STORE: &str = "STORE$";
    if message.len() <= STORE.len() {
        return Ok(message == &STORE[..message.len()]);
    }

    Ok(match_regex(message, r"^STORE\$[a-z]*$")?
        || match_regex(message, r"^STORE\$[a-z]*\$[a-z]*$")?)
}

// Returns true if message could become a correct LOAD request.
fn could_become_load_request(message: &str) -> Result<bool, TaskError> {
    static LOAD: &str = "LOAD$";
    if message.len() <= LOAD.len() {
        return Ok(message == &LOAD[..message.len()]);
    }

    match_regex(message,  r"^LOAD\$[a-z]*$")
}

// Splits a message with a prefix that is a correct STORE request
// from STORE$key$value$rest to (key, value, rest).
fn split_store_request(message: &str) -> (String, String, String) {
    let dollars: Vec<usize> = message.match_indices('$').map(|(pos, _)| pos).collect();
    let key = message[dollars[0] + 1..dollars[1]].to_string();
    let value = message[dollars[1] + 1..dollars[2]].to_string();
    let rest = message[dollars[2] + 1..].to_string();
    (key, value, rest)
}

// Splits a message with a prefix that is a correct LOAD request
// from LOAD$key$rest to (key, rest).
fn split_load_request(message: &str) -> (String, String) {
    let dollars: Vec<usize> = message.match_indices('$').map(|(pos, _)| pos).collect();
    let key = message[dollars[0] + 1..dollars[1]].to_string();
    let rest = message[dollars[1] + 1..].to_string();
    (key, rest)
}

// If message contains a prefix that is a correct request, returns
// Some(request). If message is incorrect, returns TaskError.
// Otherwise, returns None. Removes request from message.
pub fn try_parse_request(message: &mut String) -> Result<Option<Request>, TaskError> {
    if is_store_request(message)? {
        let (key, value, rest) = split_store_request(message);
        *message = rest;
        Ok(Some(Request::Store(StoreRequest::new(key, value))))
    } else if is_load_request(message)? {
        let (key, rest) = split_load_request(message);
        *message = rest;
        Ok(Some(Request::Load(LoadRequest::new(key))))
    } else if could_become_store_request(message)?
        || could_become_load_request(message)? {
        Ok(None)
    } else {
        Err(TaskError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_store_request_returns_true_when_given_exact_store_request() {
        let correct_store_requests = vec![
            "STORE$$$", "STORE$k$$", "STORE$key$$", "STORE$$v$", "STORE$$value$",
            "STORE$k$v$", "STORE$key$value$", "STORE$qwertyuiopasdfghjklzxcvbnm$value$",
            "STORE$key$qwertyuiopasdfghjklzxcvbnm$"
        ];

        for request in correct_store_requests {
            assert!(is_store_request(request).unwrap());
        }
    }

    #[test]
    fn is_store_request_returns_true_when_given_prefix_of_store_request() {
        let correct_prefixes_of_store_requests = vec![
            "STORE$$$a", "STORE$k$$$$$$$$$$$$$", "STORE$key$$qqqqqqqqqqqqq", "STORE$$v$123123", "STORE$$value$*"
        ];

        for request in correct_prefixes_of_store_requests {
            assert!(is_store_request(request).unwrap());
        }
    }

    #[test]
    fn is_store_request_returns_false_when_given_incorrect_store_request() {
        let incorrect_store_requests = vec![
            "", "S", "STORE", "STORE$$", "STORE$k$", "STORE$k$v", "STORE$$v",
            "STORE$1$v$", "STORE$k$1$", "STORE$K$v$", "STORE$k$V$", "STORE$*$*$",
            "STOR$k$v$", "LOAD$k$", "STORE$A$v$abc", "aSTORE$k$v$"
        ];

        for request in incorrect_store_requests {
            assert!(!is_store_request(request).unwrap());
        }
    }

    #[test]
    fn is_load_request_returns_true_when_given_exact_load_request() {
        let correct_load_requests = vec![
            "LOAD$$", "LOAD$k$", "LOAD$key$", "LOAD$qwertyuiopasdfghjklzxcvbnm$"
        ];

        for request in correct_load_requests {
            assert!(is_load_request(request).unwrap());
        }
    }

    #[test]
    fn is_load_request_returns_true_when_given_prefix_of_load_request() {
        let correct_prefixes_of_load_requests = vec![
            "LOAD$$x", "LOAD$k$1*2*3*", "LOAD$key$$$$$", "LOAD$qwertyuiopasdfghjklzxcvbnm$abcd&4321"
        ];

        for request in correct_prefixes_of_load_requests {
            assert!(is_load_request(request).unwrap());
        }
    }

    #[test]
    fn is_load_request_returns_false_when_given_incorrect_load_request() {
        let incorrect_load_requests = vec![
            "", "L", "LOAD", "LOAD$", "LOAD$k", "LOAD$1$", "LOAD$K$",
            "LOAD$*$", "LOA$k$", "STORE$k$v$", "LOAD$K$a", "aLOAD$k$"
        ];

        for request in incorrect_load_requests {
            assert!(!is_load_request(request).unwrap());
        }
    }

    #[test]
    fn could_become_store_request_returns_true_when_should() {
        let correct_store_prefixes = vec![
            "", "S", "ST", "STO", "STOR", "STORE", "STORE$",
            "STORE$key", "STORE$qwertyuiopasdfghjklzxcvbnm",
            "STORE$key$", "STORE$key$value", "STORE$$",
            "STORE$key$qwertyuiopasdfghjklzxcvbnm"
        ];

        for request in correct_store_prefixes {
            assert!(could_become_store_request(request).unwrap());
        }
    }

    #[test]
    fn could_become_store_request_returns_false_when_should() {
        let incorrect_store_prefixes = vec![
            "T", "a", "STOE$", "STOREa", "STORE$1", "STORE$*", "STORE$a$1",
            "STORE$$$", "STORE$$$a", "STORE$key$value$", "STRE$key$value"
        ];

        for request in incorrect_store_prefixes {
            assert!(!could_become_store_request(request).unwrap());
        }
    }

    #[test]
    fn could_become_load_request_returns_true_when_should() {
        let correct_load_prefixes = vec![
            "", "L", "LO", "LOA", "LOAD", "LOAD$", "LOAD$a",
            "LOAD$key", "LOAD$qwertyuiopasdfghjklzxcvbnm"
        ];

        for request in correct_load_prefixes {
            assert!(could_become_load_request(request).unwrap());
        }
    }

    #[test]
    fn could_become_load_request_returns_false_when_should() {
        let incorrect_load_prefixes = vec![
            "O", "a", "LOD$", "LOADa", "LOAD$1", "LOAD$*", "LOAD$$",
            "LOAD$$a", "LAD$key"
        ];

        for request in incorrect_load_prefixes {
            assert!(!could_become_load_request(request).unwrap());
        }
    }

    #[test]
    fn split_store_request_splits_correctly() {
        let test_cases = vec![
            ("STORE$$$", ("", "", "")),
            ("STORE$k$$", ("k", "", "")),
            ("STORE$$v$", ("", "v", "")),
            ("STORE$k$v$", ("k", "v", "")),
            ("STORE$k$v$r", ("k", "v", "r")),
            ("STORE$$$r", ("", "", "r")),
            ("STORE$key$value$rest", ("key", "value", "rest")),
            ("STORE$k$v$STORE$k$v$", ("k", "v", "STORE$k$v$"))
        ];

        for (input, (k, v, r)) in test_cases {
            assert_eq!((k.to_string(), v.to_string(), r.to_string()), split_store_request(input));
        }
    }

    #[test]
    fn split_load_request_splits_correctly() {
        let test_cases = vec![
            ("LOAD$$", ("", "")),
            ("LOAD$k$", ("k", "")),
            ("LOAD$$r", ("", "r")),
            ("LOAD$k$r", ("k", "r")),
            ("LOAD$key$rest", ("key", "rest")),
            ("LOAD$k$LOAD$k$", ("k", "LOAD$k$"))
        ];

        for (input, (k, r)) in test_cases {
            assert_eq!((k.to_string(), r.to_string()), split_load_request(input));
        }
    }
}
