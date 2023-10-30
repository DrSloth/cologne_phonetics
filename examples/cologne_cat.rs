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
    let mut outbuf = cologne_codes::CologneVec::new();
    let pre = std::time::Instant::now();
    outbuf.read_from_utf8(&cont);
    eprintln!("Took: {:?}", pre.elapsed());
    writeln!(stdout, "{:?}", outbuf).unwrap();
    stdout.write_all(b"\n").unwrap();
    stdout.flush().unwrap();
}

