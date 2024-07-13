use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time,
};

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
                println!("{}", hex::encode(hash_object(hash, true)));
            } else {
                println!("{}", hex::encode(hash_object(arg1, false)));
            }
        }
        "ls-tree" => {
            let action = args.get(2).expect("expected -p, -t or -s");
            let hash = args.get(3).expect("expected hash");

            ls_tree(action, hash);
        }
        "write-tree" => {
            println!("{}", hex::encode(write_tree("./")));
        }
        "commit-tree" => {
            let root_tree_hash = args.get(2).expect("expected commit tree hash").as_str();
            let parent_tree_hash;
            let commit_message;

            if args.get(3).expect("expected -p").as_str() == "-p" {
                if args.get(5).expect("expected -m").as_str() == "-m" {
                    parent_tree_hash = args.get(4).expect("expected a parent tree hash").as_str();
                    commit_message = args.get(6).expect("expected a commit message").as_str();

                    commit_tree(root_tree_hash, parent_tree_hash, commit_message);
                } else {
                    println!("expected -p");
                }
            } else {
                println!("expected -m");
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

fn hash_object(file: &str, write: bool) -> Vec<u8> {
    let contents = fs::read_to_string(file).unwrap();
    let mut complete = format!("blob {}\0", contents.len());
    complete.push_str(&contents);
    let mut hasher = Sha1::new();
    hasher.update(&complete);
    let hash = hasher.finalize();

    if write {
        let hash = hex::encode(hash);
        let mut path = String::from(".git/objects/");
        path.push_str(hash.get(0..2).unwrap());
        let _ = fs::create_dir(&path);

        path.push_str("/");
        path.push_str(hash.get(2..).unwrap());

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(complete.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

        fs::write(path, compressed).unwrap();
    }

    hash.to_vec()
}

/**
 tree <size>\0
 <mode> <name>\0<20_byte_sha>
 <mode> <name>\0<20_byte_sha>
*/
fn ls_tree(flag: &str, hash: &str) {
    let mut path = String::from(".git/objects/");
    path.push_str(hash.get(0..2).unwrap());
    path.push_str("/");
    path.push_str(hash.get(2..).unwrap());

    let gz_blob = fs::read(path).unwrap();
    let gz_blob = ZlibDecoder::new(gz_blob.as_slice());

    let mut tree_type = vec![];
    let mut tree_size = vec![];

    let mut modes = vec![];
    let mut names = vec![];
    let mut hashes = vec![];

    let mut to_fill = 0;
    let mut fill_tree_type = true;
    let mut hash_count = 0;

    for byte in gz_blob.bytes() {
        let val = byte.unwrap();

        if to_fill == 0 {
            match val {
                b' ' => {
                    fill_tree_type = false;
                }
                b'\x00' => {
                    to_fill = 1;
                    modes.push(vec![]);
                    names.push(vec![]);
                    hashes.push(vec![]);
                }
                _ => {
                    if fill_tree_type {
                        tree_type.push(val);
                    } else {
                        tree_size.push(val);
                    }
                }
            }

            continue;
        } else {
            match val {
                b' ' => {
                    to_fill = 2;
                }
                b'\0' => {
                    to_fill = 3;
                }
                _ => match to_fill {
                    1 => {
                        let len = modes.len();
                        let latest = &mut modes[len - 1];
                        latest.push(val);
                    }
                    2 => {
                        let len = names.len();
                        let latest = &mut names[len - 1];
                        latest.push(val);
                    }
                    3 => {
                        let len = hashes.len();
                        let latest = &mut hashes[len - 1];
                        latest.push(val);
                        hash_count += 1;

                        if hash_count == 20 {
                            modes.push(vec![]);
                            names.push(vec![]);
                            hashes.push(vec![]);
                            to_fill = 1;
                            hash_count = 0;
                        }
                    }
                    _ => unreachable!(),
                },
            }
        }
    }

    match flag {
        "--name-only" => {
            let last = names.len() - 1;
            for (i, name) in names.into_iter().enumerate() {
                if i == last {
                    print!("{}", String::from_utf8(name).unwrap());
                } else {
                    println!("{}", String::from_utf8(name).unwrap());
                }
            }
        }
        _ => println!("Invalid flag"),
    }
}

/**
 tree <size>\0
 <mode> <name>\0<20_byte_sha>
 <mode> <name>\0<20_byte_sha>
*/
fn write_tree(path: &str) -> Vec<u8> {
    let mut working_directory = fs::read_dir(path)
        .unwrap()
        .map(|e| {
            let b = e.unwrap().path();
            b.as_os_str().to_owned()
        })
        .collect::<Vec<OsString>>();
    working_directory.sort();

    let working_directory = working_directory
        .iter()
        .map(|e| Path::new(e.as_os_str()))
        .collect::<Vec<&Path>>();

    let ignored = fs::read_to_string(".gitignore").unwrap_or_default();
    let mut ignored = ignored
        .split('\n')
        .filter_map(|e| match e.split_once('#').unwrap_or((e, "")).0 {
            "" => None,
            _ => Path::new(e).canonicalize().ok(),
        })
        .collect::<Vec<PathBuf>>();
    ignored.push(Path::new(".git").canonicalize().unwrap());

    let mut contents = vec![];

    for file in &working_directory {
        if ignored.contains(&file.canonicalize().unwrap()) {
            continue;
        }

        if file.is_file() {
            contents.extend("100644 ".as_bytes().to_vec());
        } else {
            contents.extend("40000 ".as_bytes().to_vec());
        }

        contents.extend(file.file_name().unwrap().as_encoded_bytes().to_vec());
        contents.push(b'\0');

        if file.is_file() {
            let hash = hash_object(
                &String::from_utf8(file.as_os_str().as_encoded_bytes().to_vec()).unwrap(),
                true,
            );
            contents.extend(hash);
        } else {
            let hash = write_tree(
                &String::from_utf8(file.as_os_str().as_encoded_bytes().to_vec()).unwrap(),
            );
            contents.extend(hash);
        }
    }

    let mut complete = format!("tree {}\0", contents.len()).as_bytes().to_vec();
    complete.extend(contents);

    let mut hasher = Sha1::new();
    hasher.update(&complete);
    let hash = hasher.finalize();

    let hex_hash = hex::encode(&hash);
    let mut this_path = String::from(".git/objects/");
    this_path.push_str(&hex_hash[0..2]);
    let _ = fs::create_dir(&this_path);
    this_path.push('/');
    this_path.push_str(&hex_hash[2..]);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&complete).unwrap();
    let compressed = encoder.finish().unwrap();

    fs::write(this_path, compressed).unwrap();

    hash.to_vec()
}

fn commit_tree(root_tree_hash: &str, parent_tree_hash: &str, commit_message: &str) {
    let mut contents = format!("tree {}\n", root_tree_hash);
    contents.push_str(&format!("parent {}\n", parent_tree_hash));

    let timestamp = time::UNIX_EPOCH.elapsed().unwrap().as_secs();

    contents.push_str(&format!(
        "author AmadiMichael <amadimichaeld@gmail.com> {timestamp} +0100\n",
    ));
    contents.push_str(&format!(
        "committer AmadiMichael <amadimichaeld@gmail.com> {timestamp} +0100\n",
    ));
    contents.push_str(&format!("\n{commit_message}\n"));

    let mut complete = format!("commit {}\0", contents.len());
    complete.push_str(&contents);

    let mut hasher = Sha1::new();
    hasher.update(&complete);
    let hash = hasher.finalize();

    let hex_hash = hex::encode(&hash);
    let mut this_path = String::from(".git/objects/");
    this_path.push_str(&hex_hash[0..2]);
    let _ = fs::create_dir(&this_path);
    this_path.push('/');
    this_path.push_str(&hex_hash[2..]);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&complete.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    fs::write(this_path, compressed).unwrap();

    println!("{hex_hash}");
}
