use std::fmt::Debug;
use std::process::exit;

use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::winit::dpi::{PhysicalSize, Size};
use iced_winit::winit::event_loop::EventLoopWindowTarget;
use iced_winit::winit::window::WindowBuilder;

use tmenu::TMenu;
use tmenu_settings::TMenuSettings;

use crate::tmenu::ExitState;
use iced_winit::Mode;

mod filter;
mod menu_item;
mod styles;
mod tmenu;
mod tmenu_settings;

#[derive(Debug)]
struct PlaceOnTopConfigurator {
    settings: SettingsWindowConfigurator,
}

impl<M> iced_winit::window_configurator::WindowConfigurator<M> for PlaceOnTopConfigurator {
    fn configure_builder(
        self,
        window_target: &EventLoopWindowTarget<M>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder {
        let mut window_builder = self
            .settings
            .configure_builder(window_target, window_builder);
        window_builder = window_builder.with_always_on_top(true);
        if let Some(primary) = window_target
            .primary_monitor()
            .or_else(|| window_target.available_monitors().next())
        {
            window_builder = window_builder.with_position(primary.position());
            window_builder = window_builder.with_inner_size(Size::Physical(PhysicalSize {
                width: primary.size().width,
                height: (30.0 * primary.scale_factor()) as u32,
            }));
        }

        window_builder
    }
}

fn main() {
    let app_settings = TMenuSettings::from_args(std::env::args().collect());
    let exit_state = app_settings.exit_state.clone();

    let verbose = app_settings.verbose;
    if verbose {
        eprintln!("{:?}", app_settings);
    }

    if app_settings.maybe_print_help() {
        exit(0)
    }

    let window_configurator = PlaceOnTopConfigurator {
        settings: SettingsWindowConfigurator {
            window: iced_winit::settings::Window {
                resizable: false,
                decorations: false,
                transparent: false,
                always_on_top: true,
                ..Default::default()
            },
            id: Some("tmenu".to_string()),
            mode: Mode::Windowed,
        },
    };

    let renderer_settings = iced_wgpu::Settings {
        antialiasing: Some(iced_wgpu::settings::Antialiasing::MSAAx4),
        ..iced_wgpu::Settings::from_env()
    };
    iced_winit::application::run_with_window_configurator::<
        TMenu,
        iced_futures::executor::ThreadPool,
        iced_wgpu::window::Compositor,
        _,
    >(app_settings, renderer_settings, window_configurator, true)
    .unwrap();

    if verbose {
        eprintln!("The end");
    }

    std::process::exit(match exit_state.get() {
        ExitState::Abort => 1,
        _ => 0,
    })
}
