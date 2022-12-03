use anyhow::Context as _;
use std::io::{BufReader, Read as _};
use std::io::Write as _;
use raw_tty::IntoRawMode;
use utf8_chars::BufReadCharsExt;

pub fn open(port: &std::path::Path, baudrate: u32) -> anyhow::Result<()> {
    let mut rx = serialport::new(port.to_string_lossy(), baudrate)
        .timeout(std::time::Duration::from_secs(2))
        .open_native()
        .with_context(|| format!("failed to open serial port `{}`", port.display()))?;
    let mut tx = rx.try_clone_native()?;

    let mut stdin = BufReader::new(std::io::stdin().into_raw_mode()?);
    let mut stdout = std::io::stdout();

    // Spawn a thread for the receiving end because stdio is not portably non-blocking...
    std::thread::spawn(move || loop {
        let mut buf = [0u8; 4098];
        match rx.read(&mut buf) {
            Ok(count) => {
                stdout.write(&buf[..count]).unwrap();
                stdout.flush().unwrap();
            }
            Err(e) => {
                assert!(e.kind() == std::io::ErrorKind::TimedOut);
            }
        }
    });

    loop {
        let mut buf = [0u8; 4];
        let c = stdin.read_char()?.unwrap();
        // EndOfText, On ctrl+c
        if c == char::from(3u8) {
            eprintln!("");
            eprintln!("Exiting.");
            std::process::exit(0);
        }
        let len = c.encode_utf8(&mut buf).len();
        tx.write(&buf[..len])?;
        tx.flush()?;
    }
}
