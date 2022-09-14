use crate::repo::RpmEntry;
use anyhow::Result;
use std::cmp::min;

pub enum Flag {
    LE,
    LT,
    EQ,
    GT,
    GE,
}

fn split_string_to_alpha_and_numeric_sections(str: &String) -> Result<Vec<String>> {
    let mut tmp = String::new();
    // flag: 0 - empty
    //       1 - alpha
    //       2 - numeric
    let mut flag = 0;
    let mut result: Vec<String> = Vec::new();
    for ch in str.chars() {
        if ch.is_alphabetic() {
            match flag {
                1 => tmp = tmp + &ch.to_string(),
                2 => {
                    result.push(tmp.clone());
                    tmp.clear();
                    flag = 1;
                    tmp = tmp + &ch.to_string()
                }
                _ => {
                    flag = 1;
                    tmp = tmp + &ch.to_string()
                }
            }
        } else if ch.is_numeric() {
            match flag {
                1 => {
                    result.push(tmp.clone());
                    tmp.clear();
                    flag = 2;
                    tmp = tmp + &ch.to_string()
                }
                2 => tmp = tmp + &ch.to_string(),
                _ => {
                    flag = 2;
                    tmp = tmp + &ch.to_string()
                }
            }
        } else if flag > 0 {
            result.push(tmp.clone());
            tmp.clear();
            flag = 0;
        }
    }
    if flag > 0 {
        result.push(tmp.clone());
    }
    Ok(result)
}

fn is_alphabetic_string(str: &String) -> Result<bool> {
    for ch in str.chars() {
        if !ch.is_alphabetic() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn is_equal_section(x: &String, y: &String) -> Result<bool> {
    match (is_alphabetic_string(&x)?, is_alphabetic_string(&y)?) {
        (true, true) => Ok(x == y),
        (false, false) => match (x.parse::<u32>(), y.parse::<u32>()) {
            (Ok(x_num), Ok(y_num)) => Ok(x_num == y_num),
            (_, _) => Ok(false),
        },
        (_, _) => Ok(false),
    }
}

fn section_compare(x: &String, y: &String, op: Flag) -> Result<bool> {
    match (is_alphabetic_string(&x)?, is_alphabetic_string(&y)?) {
        // Both of the sections are alphabetic.
        // Compare like strcmp function.
        (true, true) => match op {
            Flag::LT => Ok(x.cmp(&y).is_lt()),
            Flag::LE => Ok(x.cmp(&y).is_le()),
            Flag::EQ => Ok(x.cmp(&y).is_eq()),
            Flag::GE => Ok(x.cmp(&y).is_ge()),
            Flag::GT => Ok(x.cmp(&y).is_gt()),
        },
        // If one of the sections is a number, while the other is alphabetic,
        // the numeric elements is considered newer.
        // (alphabetic, numeric)
        (true, false) => match op {
            Flag::LT | Flag::LE => Ok(true),
            _ => Ok(false),
        },
        // (numeric, alphabetic)
        (false, true) => match op {
            Flag::GE | Flag::GT => Ok(true),
            _ => Ok(false),
        },
        // Both of the sections are numbers.
        (false, false) => match (x.parse::<i32>(), y.parse::<i32>()) {
            (Ok(x_num), Ok(y_num)) => match op {
                Flag::LT => Ok(x_num < y_num),
                Flag::LE => Ok(x_num <= y_num),
                Flag::EQ => Ok(x_num == y_num),
                Flag::GE => Ok(x_num >= y_num),
                Flag::GT => Ok(x_num > y_num),
            },
            (_, _) => Ok(true),
        },
    }
}

fn label_compare(x: &String, y: &String, op: Flag) -> Result<bool> {
    let x_vec = split_string_to_alpha_and_numeric_sections(x)?;
    let y_vec = split_string_to_alpha_and_numeric_sections(y)?;
    let limit = min(x_vec.len(), y_vec.len());
    for i in 0..limit {
        if is_equal_section(&x_vec[i], &y_vec[i])? {
            if i == limit - 1 {
                match op {
                    Flag::LT | Flag::GT => {
                        if x_vec.len() == y_vec.len() {
                            return Ok(false);
                        }
                    }
                    _ => continue,
                }
            }
        } else {
            return section_compare(&x_vec[i], &y_vec[i], op);
        }
    }
    if x_vec.len() != y_vec.len() {
        match op {
            Flag::LT | Flag::LE => return Ok(x_vec.len() < y_vec.len()),
            Flag::EQ => return Ok(false),
            Flag::GE | Flag::GT => return Ok(x_vec.len() > y_vec.len()),
        }
    }
    Ok(true)
}

pub fn version_compare(x: &RpmEntry, y: &RpmEntry, op: Flag) -> Result<bool> {
    match (x.get_epoch(), y.get_epoch()) {
        (Some(e1), Some(e2)) => {
            if e1 == e2 {
                match (x.get_ver(), y.get_ver()) {
                    (Some(v1), Some(v2)) => {
                        if label_compare(v1, v2, Flag::EQ)? {
                            match (x.get_rel(), y.get_rel()) {
                                (Some(r1), Some(r2)) => label_compare(r1, r2, op),
                                (Some(_), None) => match op {
                                    Flag::LT | Flag::LE | Flag::EQ => Ok(false),
                                    Flag::GE | Flag::GT => Ok(true),
                                },
                                (None, Some(_)) => match op {
                                    Flag::GE | Flag::GT | Flag::EQ => Ok(false),
                                    Flag::LT | Flag::LE => Ok(true),
                                },
                                (None, None) => match op {
                                    Flag::GE | Flag::LE | Flag::EQ => Ok(true),
                                    Flag::LT | Flag::GT => Ok(false),
                                },
                            }
                        } else {
                            label_compare(v1, v2, op)
                        }
                    }
                    (_, _) => Ok(true),
                }
            } else {
                match op {
                    Flag::LE | Flag::LT => Ok(e1 < e2),
                    Flag::GT | Flag::GE => Ok(e1 > e2),
                    Flag::EQ => Ok(false),
                }
            }
        }
        (Some(e1), None) => match op {
            Flag::LE => Ok(e1 <= 0),
            Flag::LT => Ok(e1 < 0),
            Flag::EQ => Ok(e1 == 0),
            Flag::GT => Ok(e1 > 0),
            Flag::GE => Ok(e1 >= 0),
        },
        (None, Some(e2)) => match op {
            Flag::LE => Ok(e2 >= 0),
            Flag::LT => Ok(e2 > 0),
            Flag::EQ => Ok(e2 == 0),
            Flag::GT => Ok(e2 < 0),
            Flag::GE => Ok(e2 <= 0),
        },
        (None, None) => match (x.get_ver(), y.get_ver()) {
            (Some(v1), Some(v2)) => {
                if label_compare(v1, v2, Flag::EQ)? {
                    match (x.get_rel(), y.get_rel()) {
                        (Some(r1), Some(r2)) => label_compare(r1, r2, op),
                        (Some(_), None) => match op {
                            Flag::LT | Flag::LE | Flag::EQ => Ok(false),
                            Flag::GE | Flag::GT => Ok(true),
                        },
                        (None, Some(_)) => match op {
                            Flag::GE | Flag::GT | Flag::EQ => Ok(false),
                            Flag::LT | Flag::LE => Ok(true),
                        },
                        (None, None) => match op {
                            Flag::GE | Flag::LE | Flag::EQ => Ok(true),
                            Flag::LT | Flag::GT => Ok(false),
                        },
                    }
                } else {
                    label_compare(v1, v2, op)
                }
            }
            (_, _) => Ok(true),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() -> Result<()> {
        let e1 = RpmEntry {
            name: "QAQ".to_string(),
            flags: Some("EQ".to_string()),
            epoch: None,
            ver: Some("1.2-1".to_string()),
            rel: None,
        };
        let e2 = RpmEntry {
            name: "TAT".to_string(),
            flags: Some("EQ".to_string()),
            epoch: None,
            ver: Some("1.2-1".to_string()),
            rel: None,
        };
        assert_eq!(version_compare(&e1, &e2, Flag::LE)?, true);
        Ok(())
    }

    #[test]
    fn test_label_compare() -> Result<()> {
        assert_eq!(
            label_compare(&"1.0010".to_string(), &"1.9".to_string(), Flag::LT)?,
            false
        );
        assert_eq!(
            label_compare(&"1.0010".to_string(), &"1.9".to_string(), Flag::EQ)?,
            false
        );
        assert_eq!(
            label_compare(&"1.0010".to_string(), &"1.9".to_string(), Flag::GE)?,
            true
        );
        assert_eq!(
            label_compare(&"1.05".to_string(), &"1.5".to_string(), Flag::EQ)?,
            true
        );
        assert_eq!(
            label_compare(&"1.05".to_string(), &"1.5".to_string(), Flag::GT)?,
            false
        );
        assert_eq!(
            label_compare(&"1.05".to_string(), &"1.5".to_string(), Flag::LE)?,
            true
        );
        assert_eq!(
            label_compare(&"1.0".to_string(), &"1".to_string(), Flag::GT)?,
            true
        );
        assert_eq!(
            label_compare(&"2.50".to_string(), &"2.5".to_string(), Flag::GE)?,
            true
        );
        assert_eq!(
            label_compare(&"fc4".to_string(), &"fc.4".to_string(), Flag::EQ)?,
            true
        );
        assert_eq!(
            label_compare(&"FC5".to_string(), &"fc4".to_string(), Flag::LT)?,
            true
        );
        assert_eq!(
            label_compare(&"2a".to_string(), &"2.5".to_string(), Flag::LT)?,
            true
        );
        assert_eq!(
            label_compare(&"2.5.0".to_string(), &"2.5".to_string(), Flag::GT)?,
            true
        );
        Ok(())
    }
}
