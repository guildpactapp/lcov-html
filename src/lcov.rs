use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct LcovParser;

impl LcovParser {
    pub fn parse(content: &str) -> Result<LcovFile, LcovParseError> {
        let mut reports: Vec<LcovReport> = vec![];
        let mut file = LcovReport::default();
        let lines = content.split('\n');
        for (i, raw_line) in lines.into_iter().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split(':');
            let ty = parts.next().expect("No type defined");
            match ty {
                "TN" => match parts.next() {
                    Some(name) => file.name = name,
                    None => return Err(LcovParseError::new(i, "No test name defined")),
                },
                "SF" => match parts.next() {
                    Some(path) => {
                        if path.contains(".cargo") || path.contains(".rustup") {
                            // skip libs
                            continue;
                        }
                        file.path = path;
                    }
                    None => return Err(LcovParseError::new(i, "No source path defined")),
                },
                "FN" => {
                    let mut fn_info = parts.next().map(|dat| dat.split(',')).unwrap();
                    let line_num: u32 = fn_info.next().map(|s| s.parse().unwrap()).unwrap();
                    let name = fn_info.next().unwrap();
                    file.fn_ln.insert(name, line_num);
                }
                "FNDA" => {
                    let mut fn_data = parts.next().map(|dat| dat.split(',')).unwrap();
                    let hits: u32 = fn_data.next().map(|s| s.parse().unwrap()).unwrap();
                    let name = fn_data.next().unwrap();
                    file.fn_da.insert(name, hits);
                }
                "FNF" => file.fn_found = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "FNH" => file.fn_hit = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "BRDA" => {
                    let mut br_data = parts.next().map(|dat| dat.split(',')).unwrap();
                    let line = br_data.next().unwrap();
                    let blk = br_data.next().unwrap();
                    let br = br_data.next().unwrap();
                    let hit = br_data.next().unwrap();
                    file.br_data.insert(line, BrData { blk, br, hit });
                }
                "BRF" => file.br_found = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "BRH" => file.br_hit = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "DA" => {
                    let mut line_data = parts.next().map(|dat| dat.split(',')).unwrap();
                    let line_num = line_data.next().unwrap();
                    let hits: u32 = line_data.next().map(|s| s.parse().unwrap()).unwrap();
                    file.ln_data.insert(line_num, hits);
                }
                "LH" => file.ln_hit = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "LF" => file.ln_found = parts.next().map(|s| s.parse().unwrap()).unwrap(),
                "end_of_record" => {
                    if !file.path.is_empty() {
                        reports.push(file);
                    }
                    file = LcovReport::default()
                }
                _ => return Err(LcovParseError::new(i, format!("Unexpected token '{}'", ty))),
            }
        }

        Ok(LcovFile { reports })
    }
}

#[derive(Default)]
pub struct LcovFile<'a> {
    pub reports: Vec<LcovReport<'a>>,
}

#[derive(Default)]
pub struct LcovReport<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub fn_found: u32,
    pub fn_hit: u32,
    pub fn_ln: HashMap<&'a str, u32>,
    pub fn_da: HashMap<&'a str, u32>,
    pub br_hit: u32,
    pub br_found: u32,
    pub br_data: HashMap<&'a str, BrData<'a>>,
    pub ln_hit: u32,
    pub ln_found: u32,
    pub ln_data: HashMap<&'a str, u32>,
}

pub struct BrData<'a> {
    blk: &'a str,
    br: &'a str,
    hit: &'a str,
}

#[derive(Debug)]
pub struct LcovParseError {
    pub line: usize,
    pub message: String,
}

impl LcovParseError {
    fn new<T: Into<String>>(line: usize, message: T) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    static SAMPLE_LCOV_INFO: &str = include_str!("../test/resources/sample.lcov.info");

    #[test]
    fn test_parse() {
        let res = LcovParser::parse(SAMPLE_LCOV_INFO);
        res.unwrap();
    }
}
