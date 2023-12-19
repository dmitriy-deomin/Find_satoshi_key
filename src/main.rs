mod ice_library;

extern crate num_cpus;

use std::{
    collections::HashSet,
    io,
    fs::{OpenOptions},
    fs::File,
    io::Write,
    time::Instant,
    time::Duration,
    io::{BufRead, BufReader},
    path::Path,
};
use std::sync::{Arc};
use rand::Rng;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::io::stdout;
use tokio::task;

const BACKSPACE: char = 8u8 as char;
//Список для рандом
const  HEX:[&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];

#[tokio::main]
async fn main() {
    let file_cong = "conf_find_key.txt";
    //Чтение настроек, и если их нет создадим
    //-----------------------------------------------------------------
    let conf = match lines_from_file(&file_cong) {
        Ok(text) => { text }
        Err(_) => {
            let t = format!("0 -CPU core 0/{} (0 - speed test 1 core)\n\
            *,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,* -custom find digit(0123456789ABCDEF)", num_cpus::get());
            add_v_file(&file_cong, &t);
            lines_from_file(&file_cong).expect("ERROR READ CONFIG FILE")
        }
    };
    //---------------------------------------------------------------------

    let mut num_cores: u8 = first_word(&conf[0].to_string()).to_string().parse::<u8>().unwrap();
    let custom_digit = first_word(&conf[1].to_string()).to_string().parse::<String>().unwrap();

    println!("===============================");
    println!("Find Satoshi Public key v0.0.7");
    println!("===============================\n");


    let mut bench = false;
    if num_cores == 0 {
        println!("---------------------");
        println!("  SPEED TESTS 1 core ");
        println!("---------------------");
        bench = true;
        num_cores = 1;
    }
    print!("CPU CORE:{num_cores}/{} \n", num_cpus::get());
    print!("CUSTOM DIGIT:{custom_digit}\n");


    let file_content = match lines_from_file("pub_key.txt") {
        Ok(file) => { file }
        Err(_) => {
            let dockerfile = include_str!("pub_key.txt");
            add_v_file("pub_key.txt", dockerfile);
            lines_from_file("pub_key.txt").expect("erro_ne poluchilos chto to")
        }
    };

    let mut database = HashSet::new();
    for addres in file_content.iter() {
        database.insert(addres.to_string());
    }

    println!("LOAD:{:?} PUBLIC KEY\n", database.len());
    let database = Arc::new(database);

    let (tx, rx) = mpsc::channel();

    for _ in 0..num_cores {
        let clone_database = database.clone();
        let tx = tx.clone();
        let custom_digit = custom_digit.clone();
        task::spawn_blocking(move || {
            process(clone_database, bench, tx,custom_digit);
        });
    }

    //отображает инфу в однy строку(обновляемую)
    let mut stdout = stdout();
    for received in rx {
        let list: Vec<&str> = received.split(",").collect();
        let mut speed = list[0].to_string().parse::<u64>().unwrap();
        speed = speed * num_cores as u64;
        print!("{}\rSPEED:{}/s {:.20}...", BACKSPACE, speed, list[1].to_string());
        stdout.flush().unwrap();
    }
}

fn process(file_content: Arc<HashSet<String>>, bench: bool, tx: Sender<String>, custom_digit: String) {
    let mut start = Instant::now();
    let mut speed: u64 = 0;

    let mut rng = rand::thread_rng();

    let ice_library = ice_library::IceLibrary::new();
    ice_library.init_secp256_lib();

    let list_custom: Vec<&str> = custom_digit.split(",").collect();
    //проверим что длинна правельная
    if list_custom.len() != 64 { println!("ERROR LEN HEX:{}!=64", list_custom.len()) }

    let mut hex ="".to_string();
    loop {
        hex.clear();

        for i in 0..64 {
            if list_custom[i] == "*" {
                hex.push_str(HEX[rng.gen_range(0..16)])
            } else {
                hex.push_str(&list_custom[i].to_string());
            }
        };

        let public_key_u =ice_library.privatekey_to_publickey(hex.as_str());

        if file_content.contains(&public_key_u) {
           let address_u = ice_library.privatekey_to_address(hex.as_str());
            print_and_save( address_u,&public_key_u, &hex);
        }

        if bench {
            speed = speed + 1;
            if start.elapsed() >= Duration::from_secs(1) {
                let address_u = ice_library.privatekey_to_address(hex.as_str());
                println!("--------------------------------------------------------");
                println!("SPEED {speed}/sek", );
                println!("ADDRESS:{address_u}", );
                println!("PUBLIC:{public_key_u}");
                println!("HEX:{}", hex);
                println!("--------------------------------------------------------");
                start = Instant::now();
                speed = 0;
            }
        } else {
            speed = speed + 1;
            if start.elapsed() >= Duration::from_secs(1) {
                tx.send(format!("{speed},{public_key_u}", ).to_string()).unwrap();
                start = Instant::now();
                speed = 0;
            }
        }
    }
}

fn print_and_save(address:String,key: &String, secret_key: &String) {
    println!("ADDRES:{}", &address);
    println!("Public:{}", &key);
    println!("Secret_key:{}", &secret_key);
    let s = format!("ADDRESS:{}\nPUBLIC:{}\nHEX{}\n", address,&key, &secret_key);
    add_v_file("FOUND_SATOSHI.txt", &s);
    println!("\nСохранено в FOUND_SATOSHI.txt");
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn add_v_file(name: &str, data: &str) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("cannot open file")
        .write(data.as_bytes())
        .expect("write failed");
}

fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}