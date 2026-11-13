use pocket_relay_desktop_blaze_networking::{BlazeRouterBuilder, router::handler};

use crate::blaze::session::SessionLink;

mod association_lists;
mod auth;
mod game_manager;
mod game_reporting;
mod messaging;
mod stats;
mod user_sessions;
mod util;

/// Function which creates and sets up the router that directs incoming
/// packets to different handling functions
pub fn router() -> BlazeRouterBuilder<SessionLink> {
    BlazeRouterBuilder::new()
        .apply(route_authentication)
        .apply(route_game_manager)
        .apply(route_stats)
        .apply(route_util)
        .apply(route_messaging)
        .apply(route_association_lists)
        .apply(route_game_reporting)
        .apply(route_user_sessions)
}

fn route_authentication(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::authentication as a;
    use auth::*;

    #[rustfmt::skip]
    builder.routes(
        a::COMPONENT,
        [
            (a::LOGOUT, handler(handle_logout)),
            (a::SILENT_LOGIN, handler(handle_silent_login)),
            (a::ORIGIN_LOGIN, handler(handle_origin_login)),
            (a::LOGIN, handler(handle_login)),
            (a::LOGIN_PERSONA, handler(handle_login_persona)),
            (a::LIST_USER_ENTITLEMENTS_2, handler(handle_list_user_entitlements_2)),
            (a::CREATE_ACCOUNT, handler(handle_create_account)),
            (a::PASSWORD_FORGOT, handler(handle_forgot_password)),
            (a::GET_LEGAL_DOCS_INFO, handler(handle_get_legal_docs_info)),
            (a::GET_TERMS_OF_SERVICE_CONTENT, handler(handle_get_terms_of_service_content)),
            (a::GET_PRIVACY_POLICY_CONTENT, handler(handle_get_privacy_policy_content)),
            (a::GET_AUTH_TOKEN, handler(handle_get_auth_token)),
        ],
    );
}

fn route_game_manager(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::game_manager as g;
    use game_manager::*;

    #[rustfmt::skip]
    builder.routes(
        g::COMPONENT,
        [
            (g::CREATE_GAME, handler(handle_create_game)),
            (g::ADVANCE_GAME_STATE, handler(handle_advance_game_state)),
            (g::SET_GAME_SETTINGS, handler(handle_set_game_settings)),
            (g::SET_GAME_ATTRIBUTES, handler(handle_set_attributes)),
            (g::REMOVE_PLAYER, handler(handle_remove_player)),
            (g::UPDATE_MESH_CONNECTION, handler(handle_update_mesh_connection)),
            (g::ADD_ADMIN_PLAYER, handler(handle_add_admin_player)),
            (g::START_MATCHMAKING, handler(handle_start_matchmaking)),
            (g::CANCEL_MATCHMAKING, handler(handle_cancel_matchmaking)),
            (g::GET_GAME_DATA_FROM_ID, handler(handle_get_game_data)),
            (g::JOIN_GAME, handler(handle_join_game)),
            (g::REPLAY_GAME, handler(handle_replay_game)),
        ],
    );
}

fn route_stats(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::stats as s;
    use stats::*;

    #[rustfmt::skip]
    builder.routes(
        s::COMPONENT,
        [
            (s::GET_LEADERBOARD_ENTITY_COUNT, handler(handle_get_leaderboard_entity_count)),
            (s::GET_LEADERBOARD, handler(handle_get_leaderboard)),
            (s::GET_CENTERED_LEADERBOARD, handler(handle_get_centered_leaderboard)),
            (s::GET_FILTERED_LEADERBOARD, handler(handle_get_filtered_leaderboard)),
            (s::GET_LEADERBOARD_GROUP, handler(handle_get_leaderboard_group)),
        ],
    );
}

fn route_util(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::util as u;
    use util::*;

    #[rustfmt::skip]
    builder.routes(
        u::COMPONENT,
        [
            (u::PRE_AUTH, handler(handle_pre_auth)),
            (u::POST_AUTH, handler(handle_post_auth)),
            (u::PING, handler(handle_ping)),
            (u::FETCH_CLIENT_CONFIG, handler(handle_fetch_client_config)),
            (u::SUSPEND_USER_PING, handler(handle_suspend_user_ping)),
            (u::USER_SETTINGS_SAVE, handler(handle_user_settings_save)),
            (u::GET_TELEMETRY_SERVER, handler(handle_get_telemetry_server)),
            (u::GET_TICKER_SERVER, handler(handle_get_ticker_server)),
            (u::USER_SETTINGS_LOAD_ALL, handler(handle_user_settings_load_all)),
            (u::SET_CLIENT_METRICS, handler(handle_set_client_metrics)),
        ],
    );
}

fn route_user_sessions(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::user_sessions as u;
    use user_sessions::*;

    #[rustfmt::skip]
    builder.routes(
        u::COMPONENT,
        [
            (u::RESUME_SESSION, handler(handle_resume_session)),
            (u::UPDATE_NETWORK_INFO, handler(handle_update_network_info)),
            (u::UPDATE_HARDWARE_FLAGS, handler(handle_update_hardware_flags)),
            (u::LOOKUP_USER, handler(handle_lookup_user)),
        ],
    );
}

fn route_messaging(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::messaging as m;
    use messaging::*;

    #[rustfmt::skip]
    builder.routes(
        m::COMPONENT,
        [
            (m::FETCH_MESSAGES, handler(handle_fetch_messages)),
        ],
    );
}

fn route_game_reporting(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::game_reporting as g;
    use game_reporting::*;

    #[rustfmt::skip]
    builder.routes(
        g::COMPONENT,
        [
            (g::SUBMIT_OFFLINE_GAME_REPORT, handler(handle_submit_offline_game_report)),
        ],
    );
}

fn route_association_lists(builder: &mut BlazeRouterBuilder<SessionLink>) {
    use super::components::association_lists as a;
    use association_lists::*;

    #[rustfmt::skip]
    builder.routes(
        a::COMPONENT,
        [
            (a::GET_LISTS, handler(handle_get_lists)),
        ],
    );
}
