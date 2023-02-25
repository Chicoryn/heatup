use std::convert::Infallible;
use crate::{models::*, schedule::*};

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
