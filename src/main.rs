#![cfg_attr(not(any(debug_assertions)), windows_subsystem = "windows")]

mod assets;
mod context;
mod game;
mod menu;
mod model;
mod prelude;
mod render;
mod ui;
mod util;

use crate::context::Context;

use geng::prelude::*;

const OPTIONS_STORAGE: &str = "options";
const FIXED_FPS: f64 = 60.0;

#[derive(clap::Parser)]
struct Opts {
    #[clap(long)]
    log: Option<String>,

    /// Faster loading screen.
    #[clap(long)]
    fast_load: bool,

    #[clap(long)]
    cheat: bool,

    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn parse_args<T: clap::Parser>() -> T {
    match clap::Parser::try_parse_from(batbox::cli::get()) {
        Ok(opts) => opts,
        Err(err) => {
            #[cfg(target_arch = "wasm32")]
            panic!("Failed to parse launch arguments: {}", err);
            #[cfg(not(target_arch = "wasm32"))]
            err.exit();
        }
    }
}

fn main() {
    geng::setup_panic_handler();

    let opts: Opts = parse_args();

    let mut builder = logger::builder();
    builder
        .filter_level(
            if let Some(level) = opts.log.as_deref().or(option_env!("LOG")) {
                match level {
                    "trace" => log::LevelFilter::Trace,
                    "debug" => log::LevelFilter::Debug,
                    "info" => log::LevelFilter::Info,
                    "warn" => log::LevelFilter::Warn,
                    "error" => log::LevelFilter::Error,
                    "off" => log::LevelFilter::Off,
                    _ => panic!("invalid log level string"),
                }
            } else if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
        .filter_module("calloop", log::LevelFilter::Debug)
        .filter_module("discord_presence", log::LevelFilter::Off);
    logger::init_with(builder).expect("failed to init logger");

    log::info!("Running!");

    let mut options = geng::ContextOptions::default();
    options.window.title = "Space Age".to_string();
    options.window.antialias = false;
    options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, |geng| async move {
        let main = geng_main(geng, opts);

        #[cfg(not(target_arch = "wasm32"))]
        let main = async_compat::Compat::new(main);

        if let Err(err) = main.await {
            log::error!("{err:?}");
        }
    });
}

async fn geng_main(geng: Geng, opts: Opts) -> anyhow::Result<()> {
    let loading_assets: Rc<assets::LoadingAssets> =
        geng::asset::Load::load(geng.asset_manager(), &run_dir().join("assets"), &())
            .await
            .context("when loading assets")?;

    let load_everything = load_everything(geng.clone());

    let insta_load = opts.fast_load;

    let loading_screen =
        menu::LoadingScreen::new(&geng, loading_assets, load_everything, insta_load).run();

    let context = loading_screen
        .await
        .ok_or_else(|| anyhow::Error::msg("loading screen failed"))??;

    // Run game
    let state = game::Game::new(context, opts.cheat);
    geng.run_state(state).await;

    Ok(())
}

async fn load_everything(geng: Geng) -> anyhow::Result<Context> {
    let manager = geng.asset_manager();

    let assets: Rc<assets::Assets> =
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("when loading assets")?;

    let context = Context::new(geng, assets);

    Ok(context)
}
