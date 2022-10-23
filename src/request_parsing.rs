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
