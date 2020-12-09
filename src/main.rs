#![deny(clippy::all)]

#[macro_use]
extern crate penrose;

use penrose::{
    client::Client,
    hooks::Hook,
    bindings::MouseEvent,
    contrib::{
        extensions::Scratchpad,
        hooks::{DefaultWorkspace, LayoutSymbolAsRootName},
    },
    draw::{dwm_bar, TextStyle, XCBDraw},
    helpers::{index_selectors, spawn, spawn_for_output},
    layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
    Backward, Config, Forward, Less, More, Result, Selector, WindowManager, XcbConnection,
};
use simplelog::{LevelFilter, SimpleLogger};
//use std::env; // used for getting the HOME env var

const HEIGHT: usize = 20;
const FONT: &str = "hack";
const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
const BLUE: u32 = 0x458588ff;

struct MyClientHook {}
impl Hook for MyClientHook {
    fn new_client(&mut self, wm: &mut WindowManager, c: &mut Client) {
        wm.log(&format!("new client with WM_CLASS='{}'", c.wm_class()));
        wm.log(&format!("new client with WM_NAME='{}'", c.wm_name()));
        match c.wm_name().as_ref() {
            "calibre" => c.set_workspace(6),
            "Firefox Developer Edition" => {
                wm.log(&format!("Matched client name: {}", c.wm_name()));
                wm.log(&format!("Current Workspace: {}", c.workspace()));
                c.set_workspace(7);
                wm.log(&format!("Current Workspace: {}", c.workspace()));
                //wm.client_to_workspace(&Selector::Index(7));
            }
            _ => println!("something else!")
        }
    }
}

fn main() -> Result<()> {
    // -- logging --
    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default())?;
    let mut config = Config::default();

    // -- top level config constants --
    config.workspaces = vec!["1", "2", "3", "4", "5", "6", "7", "8", "messaging"];
    config.floating_classes = &["dmenu", "dunst", "pinentry-gtk-2", "pinentry"];

    config.focused_border = BLUE;
    config.unfocused_border = BLACK;
    // -- hooks --
    let sp = Scratchpad::new("alacritty", 0.8, 0.8);
    sp.register(&mut config);

    config.hooks.push(Box::new(dwm_bar(
        Box::new(XCBDraw::new()?),
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
        &config.workspaces,
    )?));

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
    config.layouts = vec![
        Layout::new("[side]", tiled_layout, side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        Layout::new("[mono]", follow_focus_conf, monocle, n_main, ratio),
        Layout::floating("[----]"),
    ];

    let key_bindings = gen_keybindings! {
        // Program launch
        "M-p" => run_external!("dmenu_run");
        "M-S-p" => run_external!("passmenu");
        "M-Return" => run_external!("alacritty");
        "M-S-l" => run_external!("i3lock-color -c 000000");
        "M-S-Return" => run_external!("vscode");

        // actions
        "M-A-d" => run_internal!(detect_screens);
        "M-slash" => sp.toggle();

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

        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces.len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces.len()) ];
        };
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    config.hooks.push(Box::new(MyClientHook {}));
    config.hooks.push(DefaultWorkspace::new(
        "1",
        "[mono]",
        vec!["alacritty"],
    ));

    // -- init & run --
    let conn = XcbConnection::new()?;
    let mut wm = WindowManager::init(config, &conn);
    wm.grab_keys_and_run(key_bindings, mouse_bindings);

    Ok(())
}
