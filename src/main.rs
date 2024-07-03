use flate2::read::GzDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => init(),
        "cat-file" => cat_file(&args[2], &args[3]),
        _ => println!("unknown command: {}", args[1]),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "refs/heads/main\n").unwrap();

    println!("Iniitialized git directory");
}

fn cat_file(flag: &str, hash: &str) {
    match flag {
        "-p" => {
            let mut path = String::from(".git/objects/");
            path.push_str(hash.get(0..2).unwrap());
            path.push_str("/");
            path.push_str(hash.get(2..).unwrap());

            println!("dir: {}", path);

            let gz_blob = fs::read(path).unwrap();
            let gz_blob = GzDecoder::new(gz_blob.as_slice());

            println!("Header: {:?}", gz_blob.header());
        }
        _ => print!("invalid flag: {}", flag),
    }
}
