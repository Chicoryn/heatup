use crate::models::*;
use good_lp::*;
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