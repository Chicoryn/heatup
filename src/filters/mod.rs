mod index;
mod solve;

use self::index::*;
use self::solve::*;
use warp::Filter;

pub fn all() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    solve().or(index())
}
