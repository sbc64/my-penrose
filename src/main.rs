#![deny(clippy::all)]

#[macro_use]
extern crate penrose;
#[macro_use]
extern crate tracing;
extern crate foo_config;

use penrose::{
    contrib::hooks::{ClientSpawnRules, DefaultWorkspace, SpawnRule},
    core::{
        bindings::MouseEvent,
        config::Config,
        helpers::index_selectors,
        hooks::Hook,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Selector,
        xconnection::{XConn, Xid},
    },
    draw::{dwm_bar, Color, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw, XcbHooks},
    Backward, Forward, Less, More, Result,
};
use simplelog::{LevelFilter, SimpleLogger};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time;
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};
use std::sync::mpsc::channel;

const HEIGHT: usize = 20;
const FONT: &str = "hack";

struct ConfigTable {
    sound: String,
    duration: Duration,
}

struct PomodoroBlackList {
    active: bool,
    lock_file: sync::Arc<AtomicBool>,
    duration: Duration,
    start_time: Instant,
    kill_list: Vec<&'static str>,
}

impl PomodoroBlackList {
    fn new(kill_list: Vec<&'static str>, duration: Duration) -> Box<Self> {
        let mut path = env::var("XDG_CONFIG_HOME").unwrap();
        path.into_raw_parts();
        let path = path;
        path.trim_matches(pat);
        let path_exists = Path::new(&path).exists();
        let lock_file = sync::Arc::new(AtomicBool::new(path_exists));
        let lock_file_clone = lock_file.clone();

        thread::spawn(move || loop {
            let (tx, rx) = channel();
            let mut watcher = notify::raw_watcher(tx).unwrap();
            let dir = Path::new(&path).parent().unwrap();
            watcher.watch(dir, RecursiveMode::NonRecursive).unwrap();
            match rx.recv() {
                Ok(event) => {
                    if event.path.as_ref().unwrap().ends_with("romodoro.lock") {
                        match event.op {
                            Ok(notify::Op::REMOVE) => {
                                lock_file.store(false, Ordering::Relaxed);
                                info!("REMOVE EVENT: {:?}", event);
                            }
                            Ok(notify::Op::CREATE) => {
                                lock_file.store(true, Ordering::Relaxed);
                                info!("CREATE EVENT: {:?}", event);
                            }
                            Ok(_) => {
                                info!("EVENT: {:?}", event);
                            }
                            Err(event) => info!("Event error: {:?}", event),
                        }
                    }
                }
                Err(e) => info!("watch error: {:?}", e),
            };
        });

        Box::new(Self {
            active: path_exists,
            lock_file: lock_file_clone,
            duration,
            start_time: Instant::now(),
            kill_list,
        })
    }
}

impl<X: XConn> Hook<X> for PomodoroBlackList {
    fn new_client(&mut self, wm: &mut WindowManager<X>, c: Xid) -> Result<()> {
        let completed_time = if self.start_time.elapsed().as_secs() > self.duration.as_secs() {
            true
        } else {
            false
        };

        let lock_file_exists = self.lock_file.load(Ordering::Relaxed);
        if !self.active && lock_file_exists {
            self.duration = get_duration().duration;
            self.start_time = time::Instant::now();
            self.active = true;
        } else if self.active && completed_time {
            self.active = false;
        }

        info!(
            "Active: '{}', Path: '{}', Elapsed: '{:?}', Left: '{:?}'",
            self.active,
            lock_file_exists,
            self.start_time.elapsed(),
            self.duration.as_secs() - self.start_time.elapsed().as_secs()
        );

        let client = wm.client(&Selector::WinId(c)).unwrap();
        info!("new client with WM_CLASS='{}'", client.wm_class());
        info!("new client with WM_NAME='{}'", client.wm_name());

        let class_and_name = vec![client.wm_class(), client.wm_name()];
        if self.active && self.kill_list.iter().any(|e| class_and_name.contains(e)) {
            info!("KILLING '{}'", client.wm_name());
            wm.kill_client_id(c)?;
        }
        Ok(())
    }
}

fn extract_table(table: HashMap<String, foo_config::Value>) -> ConfigTable {
    let sound: String;
    match table.get("sound") {
        Some(entry) => sound = entry.to_string(),
        None => sound = "".to_string(),
    }

    let duration: u64;
    match table.get("duration") {
        Some(entry) => duration = entry.to_string().parse::<u64>().expect("Not a number"),
        None => duration = 0,
    }
    return ConfigTable {
        sound,
        duration: Duration::from_secs(duration),
    };
}

fn get_duration() -> ConfigTable {
    let mut config_path: String = env::var("XDG_CONFIG_HOME").unwrap();
    config_path.push_str("/romodoro");
    let mut settings = foo_config::Config::default();
    settings
        .merge(foo_config::File::with_name(config_path.as_ref()))
        .unwrap()
        .merge(foo_config::Environment::with_prefix("ROMODORO"))
        .unwrap();

    let begin_work = settings
        .get_table("begin_work")
        .expect("no begin_work table");
    extract_table(begin_work)
}

fn main() -> Result<()> {
    let black = Color::from(0x282828ff);
    let grey = Color::new_from_hex(0x3c3836ff);
    let blue = Color::new_from_hex(0x458588ff);
    let white = Color::new_from_hex(0xebdbb2ff);

    // -- logging --
    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default())
        .expect("Failed to init logging");

    // -- top level config constants --
    let workspaces = vec!["1", "2", "3", "4", "5"];
    let mut config_builder = Config::default().builder();
    config_builder
        .workspaces(workspaces.clone())
        .border_px(0)
        .bar_height(HEIGHT as u32)
        .floating_classes(vec![
            "dmenu",
            "dunst",
            "pinentry-gtk-2",
            "pinentry",
            "polybar",
            "rofi",
        ])
        .focused_border(blue.as_rgb_hex_string())?
        .unfocused_border(grey.as_rgb_hex_string())?;

    // -- hooks --
    let hooks: XcbHooks = vec![
        PomodoroBlackList::new(
            vec![
                "brave-browser",
                "telegram-desktop",
                "Telegram",
                "Signal",
                "signal",
                "chromium-browser",
                "New Tab - Brave",
                "Discord",
                "discord",
                "element",
                "Element",
            ],
            get_duration().duration,
        ),
        ClientSpawnRules::new(vec![
            SpawnRule::WMName("Firefox Developer Edition", 1),
            SpawnRule::WMName("Discord", 4),
            SpawnRule::WMName("Signal", 4),
            SpawnRule::WMName("Element", 4),
            SpawnRule::WMName("Roam Research", 3),
        ]),
        DefaultWorkspace::new(workspaces[0], "[mono]", vec!["alacritty"]),
        Box::new(dwm_bar(
            XcbDraw::new()?,
            HEIGHT,
            &TextStyle {
                font: FONT.to_string(),
                point_size: 12,
                fg: white,
                bg: Some(black),
                padding: (2.0, 2.0),
            },
            blue, // highlight
            grey, // empty_ws
            workspaces.clone(),
        )?),
    ];

    // -- layouts --
    let follow_focus_conf = LayoutConf {
        floating: false,
        gapless: true,
        follow_focus: true,
        allow_wrapping: true,
    };
    let tiled_layout = LayoutConf {
        floating: false,
        gapless: true,
        follow_focus: false,
        allow_wrapping: true,
    };
    let n_main = 1;
    let ratio = 0.6;
    config_builder.layouts(vec![
        Layout::new("[side]", tiled_layout, side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        Layout::new("[mono]", follow_focus_conf, monocle, n_main, ratio),
        Layout::floating("[floa]"),
    ]);

    // -- key-bindings --bindings
    let key_bindings = gen_keybindings! {
        // Program launch
        "M-p" => run_external!("dmenu_run");
        "M-S-p" => run_external!("passmenu");
        "M-Return" => run_external!("alacritty");
        "M-S-l" => run_external!("i3lock-color -c 000000");
        "M-S-Return" => run_external!("vscode");

        // actions
        "M-A-d" => run_internal!(detect_screens);

        // client management
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-C-bracketleft" => run_internal!(client_to_screen, &Selector::Index(0));
        "M-C-bracketright" => run_internal!(client_to_screen, &Selector::Index(1));
        "M-b" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-S-c" => run_internal!(kill_client);

        // workspace management
        "M-Tab" => run_internal!(toggle_workspace);
        "M-period" => run_internal!(cycle_screen, Forward);
        "M-comma" => run_internal!(cycle_screen, Backward);
        "M-S-period" => run_internal!(drag_workspace, Forward);
        "M-S-comma" => run_internal!(drag_workspace, Backward);

        // Layout & window management
        "M-t" => run_internal!(cycle_layout, Forward);
        "M-S-t" => run_internal!(cycle_layout, Backward);
        "M-A-Up" => run_internal!(update_max_main, More);
        "M-A-Down" => run_internal!(update_max_main, Less);
        "M-A-Right" => run_internal!(update_main_ratio, More);
        "M-A-Left" => run_internal!(update_main_ratio, Less);
        "M-A-C-Escape" => run_internal!(exit);

        map: { "1", "2", "3", "4", "5"} to index_selectors(5) => {
            "M-{}" => focus_workspace (REF);
            "M-S-{}" => client_to_workspace (REF);
        };
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    // -- init & run --
    let config = config_builder.build().unwrap();
    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    wm.grab_keys_and_run(key_bindings, mouse_bindings)
}
