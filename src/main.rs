use flate2::bufread::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

fn main() {
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
    let mut path = String::from(".git/objects/");
    path.push_str(hash.get(0..2).unwrap());
    path.push_str("/");
    path.push_str(hash.get(2..).unwrap());

    println!("dir: {}", path);

    let gz_blob = fs::read(path).unwrap();
    let mut gz_blob = ZlibDecoder::new(gz_blob.as_slice());

    let mut output = String::new();
    gz_blob.read_to_string(&mut output).unwrap();

    let arr = output.split("\x00").collect::<Vec<&str>>();
    let output = arr[1];

    let prefix = arr[0].split(" ").collect::<Vec<&str>>();
    let hash_type = prefix[0];
    let size = prefix[1];

    match flag {
        "-p" => println!("{}", output),
        "-s" => println!("{}", size),
        "-t" => println!("{}", hash_type),
        _ => print!("invalid flag: {}", flag),
    }
}
