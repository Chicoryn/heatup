use crate::handlers;
use warp::Filter;

pub fn solve() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("solve")
        .and(warp::post())
        .and(warp::body::content_length_limit(16_384))
        .and(warp::body::json())
        .and_then(handlers::solve)
}
