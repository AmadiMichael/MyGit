use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    env, fs,
    io::{Read, Write},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args
        .get(1)
        .expect("expected a command, one of cat-file, hash-object, ls-tree, write-tree")
        .as_str();

    match command {
        "init" => init(),
        "cat-file" => {
            let action = args.get(2).expect("expected -p, -t or -s");
            let hash = args.get(3).expect("expected hash");
            println!("{}", cat_file(action, hash));
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
        "ls-tree" => {
            let hash = args.get(2).expect("expected hash");
            ls_tree(hash);
        }
        "write-tree" => {
            write_tree();
        }
        _ => println!("unknown command: {}", args[1]),
    }
}

fn get_hash_file(hash: &str) -> String {
    let mut path = get_hash_dir(hash);
    path.push_str("/");
    path.push_str(hash.get(2..).unwrap());

    return path;
}

fn get_hash_dir(hash: &str) -> String {
    let mut path = String::from(".git/objects/");
    path.push_str(hash.get(0..2).unwrap());

    return path;
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

    println!("Iniitialized git directory");
}

fn cat_decoded(compressed_blob: &[u8]) -> ZlibDecoder<&[u8]> {
    let decoded = ZlibDecoder::new(compressed_blob);

    return decoded;
}

fn cat_file(flag: &str, hash: &str) -> String {
    let path = get_hash_file(hash);
    // println!("dir: {}", path);

    let mut output = String::new();
    cat_decoded(fs::read(path).unwrap().as_slice())
        .read_to_string(&mut output)
        .unwrap();

    let arr = output.split("\x00").collect::<Vec<&str>>();
    let output = arr[1];

    let prefix = arr[0].split(" ").collect::<Vec<&str>>();
    let hash_type = prefix[0];
    let size = prefix[1];

    match flag {
        "-p" => output.to_owned(),
        "-s" => size.to_string(),
        "-t" => hash_type.to_owned(),
        _ => format!("invalid flag: {}", flag),
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
        let mut path = get_hash_dir(&hash);
        let _ = fs::create_dir_all(&path);

        path.push_str("/");
        path.push_str(hash.get(2..).unwrap());

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(complete.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

        println!("file: {}", path);

        fs::write(path, compressed).unwrap();
    }
}

/**
 tree <size>\0
 <mode> <name>\0<20_byte_sha>
 <mode> <name>\0<20_byte_sha>
*/
fn ls_tree(hash: &str) -> String {
    let path = get_hash_file(hash);
    let compressed_blob = fs::read(path).unwrap();
    let decoded = cat_decoded(compressed_blob.as_slice());

    let mut to_write = 0;

    let mut header = Vec::new();
    let mut mode = Vec::new();
    let mut name = Vec::new();
    let mut sha_hash = Vec::new();

    let mut this_count = 0;

    for byte in decoded.bytes() {
        let val = byte.unwrap();

        if to_write == 0 {
            header.push(val);

            if val == b'\x00' {
                to_write = 1;
                println!("{}", String::from_utf8(header.clone()).unwrap());
            }

            continue;
        }

        if to_write == 1 {
            mode.push(val);
        } else if to_write == 2 {
            name.push(val);
        } else {
            this_count += 1;
            sha_hash.push(val);
        }

        if val == b' ' && to_write != 3 {
            to_write = 2;
        } else if val == b'\x00' && to_write != 3 {
            name.pop();
            to_write = 3;
        } else if this_count == 20 {
            this_count = 0;
            to_write = 1;

            let this_hash = hex::encode(sha_hash.as_slice());
            let str_name = String::from_utf8(name.clone()).unwrap();
            let str_mode = String::from_utf8(mode.clone()).unwrap();

            let file_type = {
                if str_mode.replace(" ", "") == "40000" {
                    "tree"
                } else {
                    "blob"
                }
            };

            println!("{} {} {}   {}", str_mode, file_type, this_hash, str_name);

            mode.clear();
            name.clear();
            sha_hash.clear();
        }
    }

    return String::from_utf8(header.clone())
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>()[0]
        .to_owned();
}

fn write_tree() {}
