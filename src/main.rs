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
use std::str::FromStr;

use std::sync::{Arc};
use rand::Rng;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::io::stdout;
use bitcoin::{Address, PrivateKey, PublicKey};
use bitcoin::Network::Bitcoin;
use bitcoin::secp256k1::{Secp256k1, SecretKey};

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
            let t = format!("0 -CPU core 0/{} (0 - speed test 1 core)", num_cpus::get());
            add_v_file(&file_cong, &t);
            vec![t.to_string()]
        }
    };
    //---------------------------------------------------------------------

    let stroka_0_all = &conf[0].to_string();
    let mut num_cores: u8 = first_word(stroka_0_all).to_string().parse::<u8>().unwrap();

    println!("===============================");
    println!("Find Satoshi Public key v0.0.6");
    println!("===============================\n");


    let mut bench = false;
    if num_cores == 0 {
        println!("---------------------");
        println!("  SPEED TESTS 1 core ");
        println!("---------------------");
        bench = true;
        num_cores = 1;
    }
    println!("CPU CORE:{num_cores}/{} \n", num_cpus::get());


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
        task::spawn_blocking(move || {
            process(clone_database, bench, tx);
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

fn process(file_content: Arc<HashSet<String>>, bench: bool, tx: Sender<String>) {
    let mut start = Instant::now();
    let mut speed: u64 = 0;
    let s = Secp256k1::new();

    //в случае хз подставим рабочий секретный ключ
    let def = SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001").expect("error_nevozmogen");
    let mut hex = "".to_string();

    let mut rng = rand::thread_rng();



    loop {
        hex.clear();

        for _i in 0..64 {
            hex.push_str(HEX[rng.gen_range(0..16)])
        }

        let secret_key = SecretKey::from_str(&*hex).unwrap_or(def);
        let private_key_u = PrivateKey::new_uncompressed(secret_key, Bitcoin);
        let public_key_u = PublicKey::from_private_key(&s, &private_key_u);

        if file_content.contains(&public_key_u.to_string()) {
            let address_u = Address::p2pkh(&public_key_u, Bitcoin);
            print_and_save(&address_u.to_string(), &private_key_u.to_string(), &secret_key.display_secret().to_string());
        }

        if bench {
            speed = speed + 1;
            if start.elapsed() >= Duration::from_secs(1) {
                println!("--------------------------------------------------------");
                println!("SPEED {speed}/sek", );
                println!("PUBLIC:{public_key_u}");
                println!("PRIVATE:{private_key_u}");
                println!("HEX:{}", secret_key.display_secret());
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

fn print_and_save(adress: &String, key: &String, secret_key: &String) {
    println!("ADRESS:{}", &adress);
    println!("PrivateKey:{}", &key);
    println!("Secret_key:{}", &secret_key);
    let s = format!("ADRESS:{} PrivateKey:{}\nSecret_key{}\n", &adress, &key, &secret_key);
    add_v_file("BOBLO.txt", &s);
    println!("-----------\n-----------\nСохранено в BOBLO.txt-----------\n-----------\n");
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