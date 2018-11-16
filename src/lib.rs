//! A Lua application framework for Rust libraries.
extern crate actix;
extern crate actix_lua;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate tera;
extern crate rlua;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate serde_urlencoded;
extern crate rlua_serde;
extern crate uuid;
extern crate comrak;
extern crate rust_sodium;
extern crate base64;
extern crate config;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate fern;
extern crate colored;
extern crate log_panics;
extern crate select;
#[macro_use]
extern crate serde_derive;
extern crate git2;
extern crate regex;
extern crate tantivy;

use std::sync::Arc;
use actix::prelude::*;
use actix_lua::LuaActorBuilder;
use actix_web::{server as actix_server, App};
use tera::{Tera};
use rlua::prelude::*;
use std::collections::HashMap;

pub mod bindings;
pub mod logger;

mod app_state {
    pub struct AppState {
        pub lua: ::actix::Addr<::actix_lua::LuaActor>,
        pub tera: ::std::sync::Arc<::tera::Tera>,
    }
}

fn create_vm(tera: Arc<Tera>, init_path: &str, settings: HashMap<String, String>) -> Result<Lua, LuaError> {
    let lua = unsafe { Lua::new_with_debug() };

    lua.exec::<_, ()>(r#"
        -- The debug library is unpredictable in some cases,
        -- so we only include the safe parts.

        -- Modify the table itself instead of setting the
        -- global field, because debug can also be required.

        local to_remove = {}

        for k, _ in pairs(debug) do
            if  k ~= "traceback"
            and k ~= "getinfo"
            then
                table.insert(to_remove, k)
            end
        end

        for _, k in ipairs(to_remove) do
            debug[k] = nil
        end
    "#, None)?;

    bindings::tera::init(&lua, tera)?;
    bindings::yaml::init(&lua)?;
    bindings::json::init(&lua)?;
    bindings::uuid::init(&lua)?;
    bindings::markdown::init(&lua)?;
    bindings::client::init(&lua)?;
    bindings::crypto::init(&lua)?;
    bindings::stringset::init(&lua)?;
    bindings::time::init(&lua)?;
    bindings::fs::init(&lua)?;
    bindings::select::init(&lua)?;
    bindings::git::init(&lua)?;
    bindings::regex::init(&lua)?;
    bindings::tantivy::init(&lua)?;

    // torchbear crashes if there's no log binding
    //if cfg!(feature = "log_bindings") {
        bindings::log::init(&lua)?;
    //}

    // torchbear global table
    {
        let tb_table = lua.create_table()?;
        tb_table.set("settings", settings)?;
        tb_table.set("init_filename", init_path)?;
        lua.globals().set("torchbear", tb_table)?;
    }

    // Lua Bridge
    lua.exec::<_, ()>(r#"
        xpcall(function ()

            local init_f, err = loadfile(torchbear.init_filename)
            if not init_f then error(err) end

            local handler = init_f()

            if handler then
                torchbear.handler = handler
            end
            
        end, function (msg)
            local trace = debug.traceback(msg, 3)
            log.error(trace)
        end)        

        if not torchbear.handler then
            log.error("No handler specified")
        end
    "#, None)?;

    Ok(lua)
}

pub struct ApplicationBuilder {
    log_settings: logger::Settings,
}

impl ApplicationBuilder {
    pub fn new () -> Self {
        Self {
            log_settings: logger::Settings{
                level: logger::LevelFilter::Info,
                everything: false,
            }
        }
    }

    pub fn log_level (&mut self, level: logger::Level) -> &mut Self {
        self.log_settings.level = level.to_level_filter(); self
    }

    pub fn log_everything (&mut self, b: bool) -> &mut Self {
        self.log_settings.everything = b; self
    }

    pub fn start (&mut self) {

        let mut settings = config::Config::new();

        match settings.merge(config::File::with_name("Settings.toml")) {
            Err(err) => {
                if let config::ConfigError::Foreign(err) = &err {
                    use std::io::{Error as IoErr, ErrorKind};

                    if let Some(err) = err.downcast_ref::<IoErr>() {
                        if let ErrorKind::NotFound = err.kind() {
                            println!("Error: torchbear needs an app to run. Change to the directory containing your application and run torchbear again.");
                            std::process::exit(1);
                        };
                    };
                };

                println!("Error opening Settings.toml: {}", err);
                std::process::exit(1);
            },
            _ => ()
        };
        settings.merge(config::Environment::with_prefix("torchbear")).unwrap();

        let hashmap = settings.try_into::<HashMap<String, String>>().unwrap();

        fn get_or (map: &HashMap<String, String>, key: &str, val: &str) -> String {
            map.get(key).map(|s| s.to_string()).unwrap_or(String::from(val))
        }

        let templates_path = get_or(&hashmap, "templates_path", "templates/**/*");
        let host = get_or(&hashmap, "host", "0.0.0.0:3000");
        let init_path = get_or(&hashmap, "init", "init.lua");
        let log_path = get_or(&hashmap, "log_path", "log");
        
        logger::init(::std::path::Path::new(&log_path), self.log_settings.clone());
        //log_panics::init();

        let sys = actix::System::new("torchbear");
        let tera = Arc::new(compile_templates!(&templates_path));

        let vm = create_vm(tera.clone(), &init_path, hashmap).unwrap();

        let addr = Arbiter::start(move |_| {
            let lua_actor = LuaActorBuilder::new()
                .on_handle_with_lua(include_str!("handlers/web_server.lua"))
                .build_with_vm(vm)
                .unwrap();
            lua_actor
        });

        actix_server::new(move || {
            App::with_state(app_state::AppState { lua: addr.clone(), tera: tera.clone() })
                .default_resource(|r| r.with(bindings::server::handler))
        }).bind(&host)
            .unwrap()
            .start();

        let _ = sys.run();
    }
}
