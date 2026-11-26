#![recursion_limit = "256"]

mod api;
mod app;
#[cfg(feature = "ssr")]
mod asset_hashes;
mod community;
mod components;
#[cfg(any(feature = "ssr", test))]
mod contact;
mod contact_fields;
mod creator;
#[cfg(any(feature = "ssr", test))]
mod edge;
mod search;
#[cfg(feature = "ssr")]
mod server;
#[cfg(any(feature = "ssr", test))]
mod telemetry;
mod todos;

#[cfg(feature = "ssr")]
#[worker::event(fetch)]
async fn fetch(
    req: worker::Request,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<worker::web_sys::Response> {
    edge::dispatch(req, env, ctx).await
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
