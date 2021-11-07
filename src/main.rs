use std::fmt::Debug;
use std::io::BufRead;
use std::process::exit;
use std::sync::Arc;

use iced_winit::settings::{Settings, WindowConfigurator};
use iced_winit::winit::dpi::{PhysicalSize, Size};
use iced_winit::winit::event_loop::EventLoopWindowTarget;
use iced_winit::winit::window::WindowBuilder;

use menu_item::MenuItem;
use tmenu::{TMenu, TMenuSettings};
use iced::Application;

mod filter;
mod menu_item;
mod styles;
mod tmenu;

#[derive(Debug)]
struct PlaceOnTopConfigurator {}

impl<M> WindowConfigurator<M> for PlaceOnTopConfigurator {
    fn configure_builder(
        &self,
        window_target: &EventLoopWindowTarget<M>,
        mut window_builder: WindowBuilder,
    ) -> WindowBuilder {
        window_builder = window_builder.with_always_on_top(true);
        if let Some(primary) = window_target.primary_monitor() {
            window_builder = window_builder.with_position(primary.position());
            window_builder = window_builder.with_inner_size(primary.size());
        } else if let Some(primary) = window_target.available_monitors().next() {
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
    let mut app_settings = TMenuSettings::default();
    parse_args(std::env::args().collect(), &mut app_settings);

    let verbose = app_settings.verbose;
    if verbose {
        eprintln!("{:?}", app_settings);
    }

    let settings = Settings {
        id: Some("tmenu".to_string()),
        window: iced_winit::settings::Window {
            resizable: false,
            decorations: false,
            transparent: false,
            always_on_top: true,
            ..Default::default()
        },
        flags: app_settings,
        exit_on_close_request: true,
        window_configurator: Some(Arc::new(PlaceOnTopConfigurator {})),
        ..Default::default()
    };

    let renderer_settings = iced_wgpu::Settings {
        antialiasing: Some(iced_wgpu::settings::Antialiasing::MSAAx4),
        ..iced_wgpu::Settings::from_env()
    };
    iced_winit::application::run::<
        iced::Instance<TMenu>,
        <TMenu as Application>::Executor,
        iced_wgpu::window::Compositor,
    >(settings, renderer_settings)
    .unwrap();

    if verbose {
        eprintln!("The end");
    }
}

fn parse_args(args: Vec<String>, state: &mut TMenuSettings) {
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut remaining = &args_ref[1..];

    let mut read_stdin = true;

    loop {
        match remaining {
            ["-i" | "--case-insensitive", r @ ..] => {
                state.case_insensitive = true;
                remaining = r;
            }
            ["-p" | "--match_prefix", r @ ..] => {
                state.filter_by_prefix = true;
                remaining = r;
            }
            ["-f" | "--fuzzy", r @ ..] => {
                state.fuzzy = true;
                remaining = r;
            }
            ["-u" | "--allow-undefined", r @ ..] => {
                state.allow_undefined = true;
                remaining = r;
            }
            ["--", options @ ..] => {
                read_stdin = false;
                options.iter().enumerate().for_each(|(index, s)| {
                    state.available_options.push(MenuItem::create(s, index))
                });
                break;
            }
            ["--verbose", r @ ..] => {
                state.verbose = true;
                remaining = r;
            }
            [option, ..] => {
                eprintln!("Unknown option: {}", option);
                exit(-1);
            }
            [] => {
                break;
            }
        }
    }

    if read_stdin {
        let std_in = std::io::stdin();
        let mut lines = std_in.lock().lines();
        let mut index = 0;
        while let Some(line) = lines.next() {
            match line {
                Ok(line) => state.available_options.push(MenuItem::create(&line, index)),
                Err(e) => {
                    eprintln!("Failed reading stdin: {:?}", e);
                    exit(-1);
                }
            }
            index += 1;
        }
    }

    if state.available_options.is_empty() {
        eprintln!("No options available");
        exit(2);
    }
}
