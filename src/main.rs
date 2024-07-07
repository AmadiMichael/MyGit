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
                println!("{}", hash_object(hash, true));
            } else {
                println!("{}", hash_object(arg1, false));
            }
        }
        "ls-tree" => {
            let hash = args.get(2).expect("expected hash");
            ls_tree(hash);
        }
        "write-tree" => {
            println!("{}", write_tree("./"));
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

    let binding = fs::read(path).unwrap();
    let mut decoded = cat_decoded(binding.as_slice());
    let decoded2 = cat_decoded(binding.as_slice());

    let mut hash_type = Vec::new();
    let mut size = Vec::new();
    let mut find_size = false;
    for b in decoded2.bytes() {
        let val = b.unwrap();

        if val == b' ' {
            find_size = true;
            continue;
        } else if val == b'\x00' {
            break;
        }

        if find_size {
            size.push(val);
        } else {
            hash_type.push(val);
        }
    }
    let hash_type = String::from_utf8(hash_type).unwrap();
    let size = String::from_utf8(size).unwrap();

    let mut output = String::new();
    if hash_type == "blob" {
        decoded.read_to_string(&mut output).unwrap();

        let arr = output.split("\x00").collect::<Vec<&str>>();
        output = arr[1].to_owned();
    } else {
        output = String::new();
    }

    match flag {
        "-p" => {
            ls_tree(hash);
            output
        }
        "-s" => size.to_string(),
        "-t" => hash_type.to_owned(),
        _ => format!("invalid flag: {}", flag),
    }
}

fn hash_object(file: &str, write: bool) -> String {
    let contents = fs::read_to_string(file).unwrap();
    let mut complete = format!("blob {}\0", contents.len());
    complete.push_str(&contents);
    let mut hasher = Sha1::new();
    hasher.update(&complete);
    let hash = hasher.finalize();
    let hash = hex::ToHex::encode_hex::<String>(&hash.to_vec());

    if write {
        let mut path = get_hash_dir(&hash);
        let _ = fs::create_dir(&path);

        path.push_str("/");
        path.push_str(hash.get(2..).unwrap());

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(complete.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

        // println!("file: {}", path);

        fs::write(path, compressed).unwrap();
    }

    return hash;
}

/**
 tree <size>\0
 <mode> <name>\0<20_byte_sha>
 <mode> <name>\0<20_byte_sha>
*/
fn ls_tree(hash: &str) {
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
            if val == b' ' {
                if String::from_utf8(header.clone()).unwrap() != "tree"
                    && String::from_utf8(header.clone()).unwrap() != "commit"
                {
                    println!("Not a tree hash type");
                    break;
                }
            }

            header.push(val);

            if val == b'\x00' {
                to_write = 1;
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
            mode.pop();
            to_write = 2;
        } else if val == b'\x00' && to_write != 3 {
            name.pop();
            to_write = 3;
        } else if this_count == 20 {
            this_count = 0;
            to_write = 1;

            let this_hash = hex::encode(sha_hash.as_slice());
            let str_name = String::from_utf8(name.clone()).unwrap();
            let mut str_mode = String::from_utf8(mode.clone()).unwrap();

            let file_type = {
                if str_mode.replace(" ", "") == "40000" {
                    str_mode = "040000".to_owned();
                    "tree"
                } else {
                    "blob"
                }
            };

            println!("{} {} {}    {}", str_mode, file_type, this_hash, str_name);

            mode.clear();
            name.clear();
            sha_hash.clear();
        }
    }
}

/**
 tree <size>\0
 <mode> <name>\0<20_byte_sha>
 <mode> <name>\0<20_byte_sha>
*/
fn write_tree(path: &str) -> String {
    let mut content = Vec::<u8>::new();
    let mut working_area = fs::read_dir(&path)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<String>>();
    working_area.sort();

    let ignored = fs::read_to_string(".gitignore");

    let mut ignored = match ignored {
        Ok(ref files) => files
            .split("\n")
            .map(|e| {
                let mut r = e;
                if e.starts_with('/') {
                    r = e.split_at(1).1;
                } else if e.ends_with("/") {
                    r = e.split_at(e.len() - 1).0;
                }
                r
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    ignored.push(".git");

    for file in working_area {
        if ignored.contains(&file.as_str()) {
            continue;
        }

        let is_dir = fs::metadata(&file).unwrap().file_type().is_dir();

        let (mode, hash) = {
            if is_dir {
                let mut binding = file.clone();
                binding.push('/');
                ("40000".to_owned(), write_tree(&binding))
            } else {
                let mut this_path = path.to_owned();
                this_path.push_str(&file);
                // println!("this_path {}", this_path);

                ("100644".to_owned(), hash_object(&this_path, true))
            }
        };

        content.append(&mut format!("{} {}\x00", mode, file).as_bytes().to_vec());
        content.append(&mut hex::decode(hash).unwrap());
    }

    // Hash it
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let hash = hasher.finalize();

    // append prefix of file type and size
    let mut to_encode = format!("tree {}", content.len()).as_bytes().to_vec();
    to_encode.push(b'\x00');
    to_encode.append(&mut content);

    // encode and write to file
    let hex_hash = hex::encode(&hash);
    let mut path = get_hash_dir(&hex_hash);
    let _ = fs::create_dir(&path);
    path.push_str("/");
    path.push_str(hex_hash.get(2..).unwrap());

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&to_encode).unwrap();
    let compressed = e.finish().unwrap();

    fs::write(path, compressed).unwrap();

    return hex_hash;
}
