use lazy_static::lazy_static;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub struct Cooldown {
    #[serde(alias = "displayName")]
    display_name: String,

    #[serde(alias = "cooldown")]
    cooldown: f64,

    #[serde(alias = "groupNames")]
    group_names: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SolvePayload {
    template: String,
    cooldowns: Vec<Cooldown>
}

#[derive(Deserialize, Serialize)]
pub struct Note {
    #[serde(skip_serializing_if = "String::is_empty")]
    error: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    output: String,
}

#[tokio::main]
async fn main() {
    warp::serve(filters::all())
        .run(([0, 0, 0, 0], 8080))
        .await;
}

mod filters {
    use super::handlers;
    use warp::Filter;

    pub fn all() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        solve().or(index())
    }

    fn index() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get().and(warp::fs::dir("./static"))
    }

    fn solve() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("solve")
            .and(warp::post())
            .and(warp::body::content_length_limit(16_384))
            .and(warp::body::json())
            .and_then(handlers::solve)
    }
}

mod handlers {
    use std::convert::Infallible;
    use super::{SolvePayload, Note, Template, schedule::Schedule};

    pub async fn solve(payload: SolvePayload) -> Result<impl warp::Reply, Infallible> {
        Ok(Template::parse(&payload.template).map_or_else(|err_msg| {
            warp::reply::json(&Note {
                error: err_msg.to_string(),
                output: "".to_string()
            })
        }, |template| {
            let assignments = Schedule::new(&template.usages(), &payload.cooldowns).assignments();

            warp::reply::json(&Note {
                error: "".to_string(),
                output: template.subst_assignments(&payload.template, assignments),
            })
        }))
    }
}

#[derive(Debug)]
struct Template {
    usages: Vec<CooldownUsage>
}

#[derive(Debug)]
pub struct CooldownUsage {
    uid: u64,
    at: u64,
    value: f64,
    group_names: Vec<String>,
}

impl Template {
    fn parse(template: &str) -> Result<Self, String> {
        lazy_static! {
            static ref TIME: Regex = Regex::new(r"\{time:([\d:]+)\}").unwrap();
            static ref USAGE: Regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        }

        let mut usages = vec! [];
        let mut uid_counter = 1;

        for line in template.lines() {
            if let Some(time) = TIME.captures(line) {
                let at: u64 = Self::parse_timestamp(&time[1])?;

                for caps in USAGE.captures_iter(line) {
                    let group_names = caps[1].split("/").map(|s| s.trim().to_string()).collect::<Vec<String>>();
                    let value = 1.0;
                    let uid = uid_counter;
                    uid_counter += 1;

                    usages.push(CooldownUsage { uid, at, value, group_names });
                }
            }
        }

        Ok(Self { usages })
    }

    fn parse_timestamp(ts: &str) -> Result<u64, String> {
        lazy_static! {
            static ref MMSS: Regex = Regex::new(r"(?P<mins>[\d+]+):(?P<secs>[\d]+)").unwrap();
        }

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
        lazy_static! {
            static ref TIME: Regex = Regex::new(r"\{time:([\d:]+)\}").unwrap();
            static ref USAGE: Regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        }
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

mod schedule {
    use good_lp::*;
    use super::{Cooldown, CooldownUsage};
    use std::collections::HashMap;

    pub struct Schedule<'a> {
        usages: &'a [CooldownUsage],
        cooldowns: &'a [Cooldown],
    }

    impl<'a> Schedule<'a> {
        pub fn new(usages: &'a [CooldownUsage], cooldowns: &'a [Cooldown]) -> Self {
            Self { usages, cooldowns }
        }

        pub fn assignments(self) -> HashMap<u64, Option<String>> {
            let (vars, usage_vars) = self.variables();
            let mut model = vars
                .maximise(self.utilization(&usage_vars))
                .using(default_solver);

            model = self.at_most_one_per_usage(&usage_vars).fold(model, |model, c| model.with(c));

            for (i, cooldown) in self.cooldowns.iter().enumerate() {
                model = self.at_most_one_per_time_slice(
                    &usage_vars.iter().map(|vars| vars[i]).collect::<Vec<_>>(),
                    cooldown.cooldown
                ).fold(model, |model, c| model.with(c));
            }

            model.solve()
                .map_or_else(
                    |_err| HashMap::new(),
                    |solution| {
                        self.usages.iter()
                            .zip(usage_vars.iter())
                            .map(|(usage, vars)| {
                                (
                                    usage.uid,
                                    vars.iter()
                                        .position(|v| solution.value(*v) > 0.99)
                                        .map(|i| self.cooldowns[i].display_name.clone()),
                                )
                            })
                            .collect::<HashMap<u64, _>>()
                    }
                )
        }

        fn variables(&self) -> (ProblemVariables, Vec<Vec<Variable>>) {
            let mut vars = variables!();
            let usage_vars = self.usages.iter().map(|usage| {
                self.cooldowns.iter().map(|cd| {
                    if cd.group_names.iter().any(|group_name| usage.group_names.contains(&group_name)) {
                        vars.add(variable().binary().name(&cd.display_name))
                    } else {
                        vars.add(variable().binary().max(0.0).name(&cd.display_name))
                    }
                }).collect::<Vec<Variable>>()
            }).collect::<Vec<_>>();

            (vars, usage_vars)
        }

        fn utilization(&self, usage_vars: &[Vec<Variable>]) -> Expression {
            self.usages.iter()
                .zip(usage_vars.iter())
                .flat_map(|(usage, vars)| {
                    vars.iter().map(|var| usage.value * *var)
                })
                .sum::<Expression>()
        }

        fn at_most_one_per_usage<'b>(&'b self, usage_vars: &'b [Vec<Variable>]) -> impl Iterator<Item=Constraint> + 'b {
            usage_vars.iter().map(|cooldowns| {
                cooldowns.iter().sum::<Expression>().leq(1.0)
            })
        }

        fn at_most_one_per_time_slice<'b>(&'b self, usages: &'b [Variable], time_secs: f64) -> impl Iterator<Item=Constraint> + 'b {
            self.usages.iter().enumerate().map(move |(i, usage_i)| {
                let mut expr: Expression = 0.into();

                expr += usages[i];
                for (j, usage_j) in self.usages.iter().enumerate().skip(i + 1) {
                    if (usage_i.at.abs_diff(usage_j.at) as f64) < time_secs {
                        expr += usages[j];
                    }
                }

                expr.leq(1.0)
            })
        }
    }
}
