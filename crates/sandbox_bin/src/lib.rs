// pub mod config;

use clap::builder::styling::{AnsiColor, Color, Style};

/// Cargo-like terminal color style
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
        )
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
}

use shadow_rs::shadow;
shadow!(build);

pub fn print_build() {
    let label = Style::new()
        .bold()
        .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)));
    println!("{label}version{label:#}: {}", build::PKG_VERSION);
    println!(
        "{label}commit{label:#}: {}, {}, {}",
        build::SHORT_COMMIT,
        build::BRANCH,
        build::COMMIT_DATE,
    );
    if !build::GIT_STATUS_FILE.trim().is_empty() {
        print!("{}", build::GIT_STATUS_FILE)
    }
    println!("{label}build_os{label:#}: {}", build::BUILD_OS);
    println!(
        "{label}build_channel{label:#}: {}, {}",
        build::RUST_CHANNEL,
        build::BUILD_RUST_CHANNEL
    );
    println!("{label}rust_version{label:#}: {}", build::RUST_VERSION);
    println!("{label}cargo_version{label:#}: {}", build::CARGO_VERSION);
}
