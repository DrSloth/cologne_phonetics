use std::io::{Write, Read};

fn main() {
    if let Some(path) = std::env::args().nth(1) {
        run(&mut std::fs::File::open(path).unwrap())
    } else {
        run(&mut std::io::stdin().lock())
    }
}

fn run<R: Read>(r: &mut R) {
    let mut cont = Vec::new();
    r.read_to_end(&mut cont).unwrap();
    let mut stdout = std::io::stdout().lock();
    let mut outbuf = Vec::new();
    let pre = std::time::Instant::now();
    cologne_codes::utf8_to_cologne_codes(&cont, &mut outbuf);
    eprintln!("Took: {:?}", pre.elapsed());
    for code in outbuf {
        write!(stdout, "{}", code).unwrap();
    }
    write!(stdout, "\n").unwrap();
    stdout.flush().unwrap();
}

