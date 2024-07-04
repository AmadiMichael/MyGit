use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;

fn main() {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => init(),
        "cat-file" => {
            let action = args.get(2).expect("expected -p, -t or -s");
            let hash = args.get(3).expect("expected hash");
            cat_file(action, hash);
        }
        "hash-object" => {
            let arg1 = args.get(2).expect("expected -w or file to hash");

            if arg1 == "-w" {
                let hash = args.get(3).expect("expected hash");
                hash_object(hash, true);
            } else {
                hash_object(arg1, false);
            }
        }
        _ => println!("unknown command: {}", args[1]),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

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
        "-p" => print!("{}", output),
        "-s" => println!("{}", size),
        "-t" => println!("{}", hash_type),
        _ => println!("invalid flag: {}", flag),
    }
}

fn hash_object(file: &str, write: bool) {
    let contents = fs::read_to_string(file).unwrap();
    let mut complete = format!("blob {}\0", contents.len());
    complete.push_str(&contents);
    let mut hasher = Sha1::new();
    hasher.update(&complete);
    let hash = hasher.finalize();
    let hash = hex::ToHex::encode_hex::<String>(&hash.to_vec());

    println!("{}", hash);

    if write {
        let mut path = String::from(".git/objects/");
        path.push_str(hash.get(0..2).unwrap());
        let _ = fs::create_dir_all(&path);

        path.push_str("/");
        path.push_str(hash.get(2..).unwrap());

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(complete.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

        println!("{}", path);
        println!("{:?}", compressed);

        fs::write(path, compressed).unwrap();
    }

    println!("{:?}", hash);
}
