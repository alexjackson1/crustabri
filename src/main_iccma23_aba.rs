use crusti_app_helper::App;
use std::ffi::OsString;

mod app;

fn main() {
    let app = app::common::create_app_helper();
    let fake_params = translate_args_os_params();
    app.launch_app_with_args(fake_params);
}

fn translate_args_os_params() -> Vec<OsString> {
    let real_args = std::env::args_os().skip(1).collect::<Vec<OsString>>();
    let new_args: Box<dyn Iterator<Item = OsString>> = if real_args.is_empty() {
        Box::new(std::iter::once("authors".to_string().into()))
    } else if real_args == ["--problems"] {
        Box::new(std::iter::once("problems".to_string().into()))
    } else {
        let fake_app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("unknown app name"))
            .arg(app::common::input_args())
            .args(&app::common::problem_args());
        fake_app.get_matches();
        Box::new(
            std::iter::once("solve".to_string().into())
                .chain(real_args.into_iter())
                .chain(
                    ["--reader", "iccma23_aba", "--logging-level", "off"]
                        .iter()
                        .map(|s| s.into()),
                ),
        )
    };
    std::iter::once(std::env::args_os().next().unwrap())
        .chain(new_args)
        .collect()
}
