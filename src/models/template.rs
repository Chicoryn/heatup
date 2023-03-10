use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use super::CooldownUsage;

lazy_static! {
    static ref TIME: Regex = Regex::new(r"\{time:(?P<at>[\d:]+)\}").unwrap();
    static ref USAGE: Regex = Regex::new(r"\{\{(?P<names>.*?)(<(?P<value>[\d]+)>)?\}\}").unwrap();
    static ref MMSS: Regex = Regex::new(r"(?P<mins>[\d+]+):(?P<secs>[\d]+)").unwrap();
}

pub struct Template {
    usages: Vec<CooldownUsage>
}

impl Template {
    pub fn parse(template: &str) -> Result<Self, String> {
        let mut usages = vec! [];
        let mut uid_counter = 1;

        for line in template.lines() {
            if let Some(time) = TIME.captures(line) {
                let at: u64 = Self::parse_timestamp(&time["at"])?;

                for caps in USAGE.captures_iter(line) {
                    let group_names = caps["names"].split("/").map(|s| s.trim().to_string()).collect::<Vec<_>>();
                    let value = caps.name("value")
                        .map(|value| value.as_str().parse::<f64>())
                        .unwrap_or(Ok(1.0))
                        .map_err(|_| format!("Unrecognized value format -- {}", &caps["value"]))?;
                    let uid = uid_counter;
                    uid_counter += 1;

                    usages.push(CooldownUsage { uid, at, value, group_names });
                }
            }
        }

        Ok(Self { usages })
    }

    fn parse_timestamp(ts: &str) -> Result<u64, String> {
        if let Some(mmss) = MMSS.captures(ts) {
            let mins = mmss["mins"].parse::<u64>().unwrap();
            let secs = mmss["secs"].parse::<u64>().unwrap();

            Ok(60 * mins + secs)
        } else if let Ok(secs) = ts.parse::<u64>() {
            Ok(secs)
        } else {
            Err(format!("Unrecognized time format -- {}", ts))
        }
    }

    pub fn subst_assignments(&self, template: &str, assignments: HashMap<u64, Option<String>>) -> String {
        let mut count = 0;

        template.lines().map(|line| {
            if let Some(_) = TIME.captures(line) {
                let assignments = assignments.clone();
                let mut line = line.to_string();

                while USAGE.is_match(&line) {
                    let uid = self.usages[count].uid;
                    let replace_with = if let Some(Some(display_name)) = &assignments.get(&uid) {
                        display_name.clone()
                    } else {
                        "{spell:0} Panic!".to_string()
                    };

                    line = USAGE.replace(&line, replace_with).to_string();
                    count += 1;
                }

                line
            } else {
                line.to_string()
            }
        }).collect::<Vec<String>>().join("\n")
    }

    pub fn usages(&self) -> &[CooldownUsage] {
        &self.usages
    }
}
