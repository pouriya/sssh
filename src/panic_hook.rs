use backtrace::Backtrace;
use clap::crate_name;
use std::{env, fmt::Write, mem, panic};

pub fn setup() {
    panic::set_hook(Box::new(|panic_info| {
        let name = crate_name!();
        let command = env::args().map(String::from).collect::<Vec<_>>().join(" ");
        let command = if command.trim().is_empty() {
            String::new()
        } else {
            format!(" when running `{}`", command)
        };
        let authors = env!("CARGO_PKG_AUTHORS").replace(':', ", ");
        let version = env!("APPLICATION_VERSION");
        let github_base_url = "https://github.com/pouriya/sssh";
        let cause = match (
            panic_info.payload().downcast_ref::<&str>(),
            panic_info.payload().downcast_ref::<String>(),
        ) {
            (Some(s), _) => s.to_string(),
            (_, Some(s)) => s.to_string(),
            (None, None) => "Unknown".to_string(),
        };
        let location = panic_info
            .location()
            .map(|location| {
                format!(
                    " at [{}]({github_base_url}/blob/{version}/{}#L{})",
                    location.file(),
                    location.file(),
                    location.line()
                )
            })
            .unwrap_or_default();
        let backtrace = backtrace_string();
        let os_info = os_info::get().to_string();
        print!(
            r#"{name} had a problem and crashed. To help us diagnose the problem you can send us a crash report.
Please submit an issue at `{github_base_url}/issues/new?title=Crash+Report+v{version}`.
If you are not familiar with GitHub, please email with the subject of `{name} Crash Report v{version}` to `{authors}`.
In order to improve `{name}`, we rely on people to submit reports.


Please include the following report in GitHub issue or email:



# Crash report
Panic{command}{location}.

### Cause
```text
{cause}
```

#### Backtrace
```text{backtrace}
```

#### OS info
{os_info}

## How to reproduce
If the bug is reproducible, please tell us how did it happen (write it here).

#### Logs
```text
Note that you can rerun `{name}` with `-v` flag and include debug logging here.
```
"#,
        );
    }))
}

// Copied from [human-panic](https://github.com/rust-cli/human-panic/blob/v1.1.1/src/report.rs#L59)
fn backtrace_string() -> String {
    const SKIP_FRAMES_NUM: usize = 7;
    const HEX_WIDTH: usize = mem::size_of::<usize>() + 2;
    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
    let mut backtrace = String::new();
    for (idx, frame) in Backtrace::new()
        .frames()
        .iter()
        .skip(SKIP_FRAMES_NUM)
        .enumerate()
    {
        let ip = frame.ip();
        let _ = write!(backtrace, "\n{idx:4}: {ip:HEX_WIDTH$?}");
        let symbols = frame.symbols();
        if symbols.is_empty() {
            let _ = write!(backtrace, " - <unresolved>");
            continue;
        }
        for (idx, symbol) in symbols.iter().enumerate() {
            if idx != 0 {
                let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
            }
            if let Some(name) = symbol.name() {
                let _ = write!(backtrace, " - {name}");
            } else {
                let _ = write!(backtrace, " - <unknown>");
            }
            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                let _ = write!(
                    backtrace,
                    "\n{:3$}at {}:{}",
                    "",
                    file.display(),
                    line,
                    NEXT_SYMBOL_PADDING
                );
            }
        }
    }
    backtrace
}
