use super::*;

pub(crate) fn run() {
    match parse_launch_mode() {
        Ok(LaunchMode::Cli(path)) => {
            if let Err(error) = run_cli(path) {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        Ok(LaunchMode::Gui(initial_path)) => {
            let config_path = default_config_path();
            let config = AppConfig::load(&config_path);
            let initial_dir = choose_initial_directory(initial_path, &config);
            run_gui(config, config_path, initial_dir);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

enum LaunchMode {
    Gui(Option<PathBuf>),
    Cli(PathBuf),
}

fn parse_launch_mode() -> Result<LaunchMode, String> {
    let mut args = std::env::args_os().skip(1);

    match args.next() {
        None => Ok(LaunchMode::Gui(None)),
        Some(first) => {
            let first_text = first.to_string_lossy().into_owned();
            if first_text == "--gui" {
                Ok(LaunchMode::Gui(args.next().map(PathBuf::from)))
            } else if first_text == "--cli" {
                let path = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| "Missing path after --cli".to_owned())?;
                Ok(LaunchMode::Cli(path))
            } else {
                Ok(LaunchMode::Cli(PathBuf::from(first)))
            }
        }
    }
}

fn choose_initial_directory(initial_path: Option<PathBuf>, config: &AppConfig) -> PathBuf {
    initial_path
        .as_deref()
        .and_then(resolve_existing_directory)
        .or_else(|| {
            config
                .last_workdir
                .as_deref()
                .and_then(resolve_existing_directory)
        })
        .or_else(|| std::env::current_dir().ok())
        .or_else(|| discover_roots().into_iter().next())
        .unwrap_or_else(|| PathBuf::from(r"C:\"))
}

#[cfg(target_os = "windows")]
fn detach_own_console_window() {
    type Hwnd = *mut core::ffi::c_void;

    unsafe extern "system" {
        fn GetConsoleProcessList(process_list: *mut u32, process_count: u32) -> u32;
        fn GetConsoleWindow() -> Hwnd;
        fn ShowWindow(window: Hwnd, command: i32) -> i32;
    }

    const SW_HIDE: i32 = 0;

    let mut processes = [0u32; 2];
    let attached = unsafe { GetConsoleProcessList(processes.as_mut_ptr(), processes.len() as u32) };
    if attached > 1 {
        return;
    }

    let console = unsafe { GetConsoleWindow() };
    if !console.is_null() {
        unsafe {
            ShowWindow(console, SW_HIDE);
        }
    }
}

fn run_cli(path: PathBuf) -> Result<(), String> {
    let entries = load_entries(&path)?;

    println!("Loaded {} entries from {}", entries.len(), path.display());

    for (index, entry) in entries.iter().take(CLI_PREVIEW_ROWS).enumerate() {
        println!(
            "[{}] key={} value={}",
            index + 1,
            format_bytes(&entry.key_bytes),
            format_bytes(&entry.value_bytes)
        );
    }

    if entries.len() > CLI_PREVIEW_ROWS {
        println!(
            "... {} more entries omitted",
            entries.len() - CLI_PREVIEW_ROWS
        );
    }

    Ok(())
}

fn run_gui(config: AppConfig, config_path: PathBuf, initial_dir: PathBuf) {
    #[cfg(target_os = "windows")]
    detach_own_console_window();

    let title = I18n::new(config.language)
        .text(TextKey::WindowTitle)
        .to_owned();

    Application::new().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("ctrl-c", CopySelectedItem, None),
            KeyBinding::new("cmd-c", CopySelectedItem, None),
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-a", SelectAll, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("ctrl-v", Paste, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
        ]);

        let bounds = Bounds::centered(None, size(px(1480.0), px(940.0)), cx);
        let config = config.clone();
        let config_path = config_path.clone();
        let initial_dir = initial_dir.clone();
        let title = title.clone();

        let window = cx
            .open_window(
                WindowOptions {
                    focus: true,
                    titlebar: Some(TitlebarOptions {
                        title: Some(title.into()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| {
                    let config = config.clone();
                    let config_path = config_path.clone();
                    let initial_dir = initial_dir.clone();
                    cx.new(|cx| LevelDbBrowserApp::new(config, config_path, initial_dir, cx))
                },
            )
            .unwrap();

        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
        cx.on_action(move |_: &CopySelectedItem, cx| {
            let _ = view.update(cx, |this, cx| {
                this.copy_selected_item_to_clipboard(cx);
            });
        });

        cx.activate(true);
    });
}
