use iced::application::Title;
use iced::{Settings, Task};
use iced_core::Theme;
use iced_winit::program::DefaultStyle;
use std::fmt::Debug;
use std::process::exit;
use iced_core::window::settings::PlatformSpecific;
use tmenu::TMenu;
use tmenu_settings::TMenuSettings;

use crate::tmenu::{ExitState, MainAction};

mod filter;
mod menu_item;
mod styles;
mod tmenu;
mod tmenu_settings;

// #[derive(Debug)]
// struct PlaceOnTopConfigurator {
//     settings: SettingsWindowConfigurator,
// }
//
// impl<M> iced_winit::window_configurator::WindowConfigurator<M> for PlaceOnTopConfigurator {
//     fn configure_builder(
//         self,
//         window_target: &EventLoopWindowTarget<M>,
//         window_builder: WindowBuilder,
//     ) -> WindowBuilder {
//         let mut window_builder = self
//             .settings
//             .configure_builder(window_target, window_builder);
//         window_builder = window_builder.with_always_on_top(true);
//         if let Some(primary) = window_target
//             .primary_monitor()
//             .or_else(|| window_target.available_monitors().next())
//         {
//             window_builder = window_builder.with_position(primary.position());
//             window_builder = window_builder.with_inner_size(Size::Physical(PhysicalSize {
//                 width: primary.size().width,
//                 height: (30.0 * primary.scale_factor()) as u32,
//             }));
//         }
//
//         window_builder
//     }
// }

fn main() {
    let args = std::env::args().collect();
    eprintln!("all args: {args:?}");
    let app_settings = TMenuSettings::from_args(args);
    let exit_state = app_settings.exit_state.clone();

    let verbose = app_settings.verbose;
    eprintln!("width: {}", app_settings.width.unwrap_or(0));
    if verbose {
        eprintln!("{:?}", app_settings);
    }

    if app_settings.maybe_print_help() {
        exit(0)
    }

    // let window_configurator = PlaceOnTopConfigurator {
    //     settings: SettingsWindowConfigurator {
    //         window: iced_winit::settings::Window {
    //             resizable: false,
    //             decorations: false,
    //             transparent: false,
    //             always_on_top: true,
    //             ..Default::default()
    //         },
    //         id: Some("tmenu".to_string()),
    //         mode: Mode::Windowed,
    //     },
    // };

    let app = iced::application(
        |state: &TMenu| "tmenu".to_string(),
        |state: &mut TMenu, message: MainAction| state.update(message),
        TMenu::view,
    )
    .subscription(TMenu::subscription);

    let app = app.window(iced::window::Settings {
        platform_specific: PlatformSpecific {
            application_id: "tmenu".to_string(),
            override_redirect: false,
        },

        ..iced::window::Settings::default()
    });

    let app = app.decorations(false)
    .window_size(iced_core::Size::new(
        app_settings.width.unwrap_or(1024) as f32,
        30f32,
    ))
    .resizable(false)
    .theme(|_ignore| Theme::Dark);



    eprintln!("start {app_settings:?}");

    let result = app.run_with(move || (TMenu::new(app_settings), Task::done(MainAction::Focus)));

    let exit_code = match result {
        Ok(_) => match exit_state.get() {
            ExitState::Continue => 2,
            ExitState::Exit => 0,
            ExitState::Abort => 1,
        },
        Err(_) => -1,
    };

    if verbose {
        eprintln!("The end {exit_code}");
    }

    std::process::exit(exit_code);
}

struct TMenuTitle;
impl Title<()> for TMenuTitle {
    fn title(&self, state: &()) -> String {
        "TMenu".to_string()
    }
}
