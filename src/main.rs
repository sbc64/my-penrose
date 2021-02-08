#![deny(clippy::all)]

#[macro_use]
extern crate penrose;

use penrose::{
    contrib::{
        extensions::Scratchpad,
        hooks::{
            DefaultWorkspace,
            LayoutSymbolAsRootName,
            SpawnRule,
            ClientSpawnRules,
        },
        layouts::paper,
    },
    draw::{dwm_bar, TextStyle},
    core::{
        client::Client,
        config::Config,
        helpers::index_selectors,
        hooks::Hook,
        layout::{bottom_stack, side_stack, Layout, LayoutConf, monocle},
        manager::WindowManager,
        ring::Selector,
        xconnection::XConn,
        bindings::MouseEvent,
    },
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw, XcbConnection, XcbHooks},
    Backward, Forward, Less, More, Result,
};

use simplelog::{LevelFilter, SimpleLogger};
use std::{sync, thread, time};
use std::time::{Duration, Instant};

const HEIGHT: usize = 20;
const FONT: &str = "hack";
const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
const BLUE: u32 = 0x458588ff;

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
            time: Pomo {
                active,
                inactive,
            },
            last_time: Instant::now(),
            kill_list
        })
    }
}

impl<X: XConn> Hook<X> for PomodoroBlackList {
    fn new_client(&mut self, wm: &mut WindowManager<X>, c: &mut Client) -> Result<()> {
        let current = time::Instant::now();

        let elapsed = current.checked_duration_since(self.last_time)
                .unwrap()
                .as_secs();

        wm.log(&format!(
            "Elapsed: {}", elapsed
        ));
        wm.log(&format!(
            "Active: {}", self.active
        ));

        if self.active && elapsed > self.time.active {
            self.active = false;
            self.last_time = current;
        } else if !self.active && elapsed > self.time.inactive {
            self.active = true;
            self.last_time = current;
        }

        wm.log(&format!("new client with WM_CLASS='{}'", c.wm_class()));
        wm.log(&format!("new client with WM_NAME='{}'", c.wm_name()));

        let class_and_name = vec![c.wm_class().as_ref(), c.wm_name().as_ref()];
        if self.active &&
            self.kill_list.iter().any(|e| class_and_name.contains(e))
        {
            wm.kill_client_id(c.id())?
        }
        Ok(())
    }
}

fn main() -> Result<()> {
   // -- logging --
    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default())
        .expect("Failed to init logging");

    // -- top level config constants --
    let workspaces = vec!["1", "2", "3", "4", "5", "6", "7", "8", "messaging"];
    let mut config_builder = Config::default().builder();
    config_builder
        .workspaces(workspaces.clone())
        .floating_classes(vec!["dmenu", "dunst", "pinentry-gtk-2", "pinentry"])
        .focused_border("#cc241d")?
        .unfocused_border("#3c3836")?;

    // -- hooks --
    let mut hooks: XcbHooks = vec![];
    hooks.push(PomodoroBlackList::new(vec![
        "brave-browser",
        "telegram-desktop",
        "Telegram",
        "Signal",
        "signal",
        "Discord",
        "discord",
        "chromium-browser",
    ]));
    hooks.push(ClientSpawnRules::new(vec![
        SpawnRule::WMName("Firefox Developer Edition" , 1),
        SpawnRule::WMName("Discord", 8),
        SpawnRule::WMName("Signal", 8),
        SpawnRule::WMName("Element", 8),
        SpawnRule::WMName("Roam Research", 5),
    ]));
    hooks.push(DefaultWorkspace::new(
        workspaces[0],
        "[mono]",
        vec!["alacritty"],
    ));

    // -- layouts --
    let mut bar = dwm_bar(
        XcbDraw::new()?,
        HEIGHT,
        &TextStyle {
            font: FONT.to_string(),
            point_size: 11,
            fg: WHITE.into(),
            bg: Some(BLACK.into()),
            padding: (2.0, 2.0),
        },
        BLUE, // highlight
        GREY, // empty_ws
        workspaces,
    )?;

    let config = config_builder
        .workspaces(vec!["main"])
        .build()
        .unwrap();

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
        "M-A-period" => run_internal!(cycle_screen, Forward);
        "M-A-comma" => run_internal!(cycle_screen, Backward);
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
    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    bar.startup(&mut wm);
    wm.grab_keys_and_run(key_bindings, mouse_bindings)?;
    Ok(())
}
