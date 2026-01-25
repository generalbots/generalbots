pub mod abs;
pub mod aggregate;
pub mod basic_math;
pub mod random;
pub mod round;
pub mod trig;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_math_functions(state: &Arc<AppState>, user: UserSession, engine: &mut Engine) {
    abs::abs_keyword(state, user.clone(), engine);
    round::round_keyword(state, user.clone(), engine);
    basic_math::int_keyword(state, user.clone(), engine);
    basic_math::floor_keyword(state, user.clone(), engine);
    basic_math::ceil_keyword(state, user.clone(), engine);
    basic_math::max_keyword(state, user.clone(), engine);
    basic_math::min_keyword(state, user.clone(), engine);
    basic_math::mod_keyword(state, user.clone(), engine);
    basic_math::sgn_keyword(state, user.clone(), engine);
    basic_math::sqrt_keyword(state, user.clone(), engine);
    basic_math::pow_keyword(state, user.clone(), engine);
    random::random_keyword(state, user.clone(), engine);
    trig::sin_keyword(state, user.clone(), engine);
    trig::cos_keyword(state, user.clone(), engine);
    trig::tan_keyword(state, user.clone(), engine);
    trig::log_keyword(state, user.clone(), engine);
    trig::exp_keyword(state, user.clone(), engine);
    trig::pi_keyword(state, user.clone(), engine);
    aggregate::sum_keyword(state, user.clone(), engine);
    aggregate::avg_keyword(state, user, engine);

    debug!("Registered all math functions");
}
