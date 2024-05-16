use std::fmt;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use itertools::{Itertools, Position};
//use with_position::{WithPosition, Position};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JWTError {
    Internal(String),
    External(jsonwebtoken::errors::Error),
}

pub type JWTResult<T> = Result<T, JWTError>;

impl From<jsonwebtoken::errors::Error> for JWTError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        JWTError::External(value)
    }
}

impl fmt::Display for JWTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JWTError::Internal(err) => write!(f, "{err}"),
            JWTError::External(err) => write!(f, "{err}"),
        }
    }
}

pub fn slurp_file(file_name: &str) -> Vec<u8> {
    fs::read(file_name).unwrap_or_else(|_| panic!("Unable to read file {file_name}"))
}

pub fn slurp_file_string(file_name: &str) -> String {
    fs::read_to_string(file_name).unwrap_or_else(|_| panic!("Unable to read file {file_name}"))
}

pub fn slurp_file_certificate(file_name: &str) -> String {
    
    let path = Path::new(file_name);
    let display = path.display();
    
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let reader = BufReader::new(file);    
    let mut cBuffer = String::new();    

    for line in reader.lines().enumerate().with_position() {
        match line {
            Position::Last((_idx, _)) => (),
            Position::First((_idx, _)) => (),
            Position::Middle((_idx, text)) => {
                match text {
                    Ok(text) => {                        
                            cBuffer = cBuffer.to_owned() + &text ;                         
                    }
                    Err(e) => println!("ERROR: {}", e),
                }
            },
            Position::Only((_idx, _)) => (),
        }
    }
    
    cBuffer.to_string()
    
}


pub fn write_file(path: &Path, content: &[u8]) {
    fs::write(path, content).unwrap_or_else(|_| panic!("Unable to write file {}", path.display()))
}

pub fn parse_duration_string(val: &str) -> Result<i64, String> {
    let mut base_val = val.replace(" ago", "");

    if val.starts_with('-') {
        base_val = base_val.replacen('-', "", 1);
    }

    match parse_duration::parse(&base_val) {
        Ok(parsed_duration) => {
            let is_past = val.starts_with('-') || val.contains("ago");
            let seconds = parsed_duration.as_secs() as i64;

            if is_past {
                Ok(-seconds)
            } else {
                Ok(seconds)
            }
        }
        Err(_) => Err(String::from(
            "must be a UNIX timestamp or systemd.time string",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_systemd_time_string() {
        assert_eq!(parse_duration_string("5s").unwrap(), 5);
        assert_eq!(parse_duration_string("2 days").unwrap(), 60 * 60 * 24 * 2);
        assert_eq!(parse_duration_string("-5s").unwrap(), -5);
        assert_eq!(
            parse_duration_string("2 days ago").unwrap(),
            60 * 60 * 24 * -2
        );
    }
}
