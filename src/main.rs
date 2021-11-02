#![deny(clippy::all)]

#[macro_use]
extern crate penrose;

#[macro_use]
extern crate tracing;

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
use std::time::{Duration, Instant};
use std::{sync, thread, time};

const HEIGHT: usize = 16;
const FONT: &str = "hack";

struct Pomo {
    active: u64,
    inactive: u64,
}

struct PomodoroBlackList {
    active: bool,
    last_time: Instant,
    time: Pomo,
    kill_list: Vec<&'static str>,
}

impl PomodoroBlackList {
    fn new(kill_list: Vec<&'static str>) -> Box<Self> {
        let active: u64 = 1200;
        let inactive: u64 = 300;
        Box::new(Self {
            active: true,
            time: Pomo { active, inactive },
            last_time: Instant::now(),
            kill_list,
        })
    }
}

impl<X: XConn> Hook<X> for PomodoroBlackList {
    fn new_client(&mut self, wm: &mut WindowManager<X>, c: Xid) -> Result<()> {
        let current = time::Instant::now();

        let elapsed = current
            .checked_duration_since(self.last_time)
            .unwrap()
            .as_secs();
        let client = wm.client(&Selector::WinId(c)).unwrap();

        if self.active && elapsed > self.time.active {
            self.active = false;
            self.last_time = current;
        } else if !self.active && elapsed > self.time.inactive {
            self.active = true;
            self.last_time = current;
        }

        info!("new client with WM_CLASS='{}'", client.wm_class());
        info!("new client with WM_NAME='{}'", client.wm_name());

        let class_and_name = vec![client.wm_class(), client.wm_name()];
        if self.active && self.kill_list.iter().any(|e| class_and_name.contains(e)) {
            wm.kill_client_id(c)?;
        }
        Ok(())
    }
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
    let ws = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    let mut config_builder = Config::default().builder();
    config_builder
        .workspaces(ws.clone())
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
        PomodoroBlackList::new(vec![
            "brave-browser",
            "telegram-desktop",
            "Telegram",
            "Signal",
            "signal",
            "Discord",
            "discord",
            "chromium-browser",
        ]),
        ClientSpawnRules::new(vec![
            SpawnRule::WMName("Firefox Developer Edition", 1),
            SpawnRule::WMName("Discord", 8),
            SpawnRule::WMName("Signal", 8),
            SpawnRule::WMName("Element", 8),
            SpawnRule::WMName("Roam Research", 5),
        ]),
        DefaultWorkspace::new(ws[0], "[mono]", vec!["alacritty"]),
        Box::new(dwm_bar(
            XcbDraw::new()?,
            HEIGHT,
            &TextStyle {
                font: FONT.to_string(),
                point_size: 10,
                fg: white,
                bg: Some(black),
                padding: (2.0, 2.0),
            },
            blue, // highlight
            grey, // empty_ws
            ws.clone(),
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
        Layout::floating("[----]"),
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
        "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
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

        map: { "1", "2", "3", "4", "5", "6", "7", "8", "9" } to index_selectors(9) => {
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
