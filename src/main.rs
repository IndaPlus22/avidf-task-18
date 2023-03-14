//With influnce from the work of others as my initial solution didnt work!


use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;

use std::io::{self, BufRead, Write, Seek};
use std::io::BufReader;

use std::time::{Instant};

use clap::Parser;

use std::process;


#[derive(Parser, Clone)]
pub struct Cli {
    pattern: Vec<String>,
}
#[derive(Debug)]
pub struct Element{
    index: usize,
    key: usize,
}

impl Element{
    pub fn create(data: String, len: &usize, byte_index: usize) -> Element{
        let split_data = data.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        let hashed_word: usize = hashlib::hash_string(&split_data[0]);
        Element{
            key: (hashed_word % len),
            index: byte_index,
        }
    }
}

fn main() {

    let args = Cli::parse();


    if args.pattern.len() < 1 {
        println!("err; no valid argument");
        process::exit(1);
    }

    match args.pattern[0].as_str() {
        "find" => {
            if args.pattern.len() > 1{
                find(&args.pattern[1]);
            }else{
                
            }
        },
        "load" => {

            let fname = Path::new("korpus.zip");
            let file = fs::File::open(&fname).unwrap();
            let reader = BufReader::new(file);

            //extract from the zip 
            let mut archive = zip::ZipArchive::new(reader).unwrap();
            zip::ZipArchive::extract(&mut archive, Path::new("")).unwrap();

            let start = Instant::now();
            if args.pattern.len() > 1 && args.pattern[1].eq("debug"){
                println!("Debugging...");
                create_magic_file(true);
            }else{
                create_magic_file(false);
            }
            let time = start.elapsed();
            println!("It took: {:?} to generate the magic file", time);
        },
        _ => (),
    }
}



fn find(in_word: &String){

    let word = in_word.to_lowercase();

    //Get/make bufreader
    let in_magic = Path::new("magic_file.txt");
    let in_index = Path::new("index_file.txt");
    let in_korpus = Path::new("korpus.txt");
    let mut magic_file;
    let mut index_file;
    let mut korpus_file;

    match File::open(in_magic) {
        Ok(f) => {
            magic_file = io::BufReader::new(f);  
                
        },

        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };

    match File::open(in_index) {
        Ok(f) => {
            index_file = io::BufReader::new(f);
                
        },
        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };

    match File::open(in_korpus) {
        Ok(f) => {
            korpus_file = io::BufReader::new(f);
               
        },
        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };

    let index = hashlib::hash_string(&word) % 116502101;

   

    let mut in_bin = vec![];

    let start = Instant::now();

        magic_file.seek_relative((index - 1)  as i64).unwrap();
        magic_file.read_until(b'%', &mut in_bin).unwrap();
        
       
        if in_bin[0] == 37{
            in_bin = vec![];
            magic_file.read_until(b'%', &mut in_bin).unwrap();
        }

        let mut buf = String::from_utf8_lossy(&in_bin[0..in_bin.len()-1]).parse::<usize>().unwrap();
        

        index_file.seek_relative(buf as i64).unwrap();

        let mut in_word:String = "".to_string();
        index_file.read_line(&mut in_word).unwrap();


        while !word.eq(&in_word[..word.len()]){

            let after = index_file.stream_position().unwrap();
            index_file.seek_relative(0 as i64 - after as i64).unwrap();
            in_bin = vec![];
            magic_file.read_until(b'%', &mut in_bin).unwrap();
            buf = String::from_utf8_lossy(&in_bin[0..in_bin.len()-1]).parse::<usize>().unwrap();
            index_file.seek_relative(buf as i64).unwrap();
            in_word = "".to_string();
            index_file.read_line(&mut in_word).unwrap();
        }



        let indexes: Vec<String> = in_word[word.len()..].split_whitespace()
        .map(|s| s.to_string()).collect::<Vec<String>>();

        let duration = start.elapsed();
        println!("Duration of getting indexes: {:?}", duration);


        let offset = 12i64;
        let mut collection: Vec<String> = vec![];


        for ind in &indexes{
            let i = ind.parse::<usize>().unwrap_or(0);
            korpus_file.seek_relative(i as i64 - offset);

            let mut in_buffer = vec![0; (offset*2i64) as usize];
            korpus_file.read(&mut in_buffer).unwrap();
            
            let in_value = String::from_utf8_lossy(&in_buffer).to_string();
            let after = korpus_file.stream_position().unwrap();

            korpus_file.seek_relative(0 as i64 - after as i64).unwrap();
            collection.push(in_value);
        }

    let duration = start.elapsed();

    println!("The word is: {:#?}", &in_word[..word.len()]);

    println!("Number of found instances: {}", collection.len());

    for mut out in collection{
        out = out.replace("\n", " ");   
        println!("The context is: {}", out)
    }

    println!("The duration for the process: {:?}", duration);

}





fn create_magic_file(debug: bool){

    create_index_file(debug);

    let in_path = Path::new("index_file.txt");
    let out_path = Path::new("magic_file.txt");
    
    let in_data = fs::read_to_string(in_path).expect("Unable to read file");
    let lines:Vec<String> = in_data.split("\n").map(|s| s.to_string().replace("\r", "")).collect::<Vec<String>>();


    if debug {
        for i in &lines{
            println!("File contains: {}", i);
        }
    }

    let mut bytes: usize = 0;
    

    
    for i in &lines{
        bytes += i.len();
    }

    let mut new_data: Vec<Element> = vec![];
    let mut current_bytes: usize = 0;

    for u in &lines{
        new_data.push(Element::create(u.to_string()
        , &bytes, current_bytes));

        current_bytes += u.len() + 1;
    }

    
    new_data.sort_by_key(|a| a.key);

    let file = File::create(out_path).unwrap();
    let mut file_writer = io::LineWriter::new(file);
    let mut bytes_file: usize = 1;


    
    for i in 0..new_data.len(){
        
            while new_data[i].key > bytes_file{

                file_writer.write("\n".as_bytes()).unwrap();
                bytes_file += 1;
            }
        
        let mut new_out:String = new_data[i].index.to_string();
        
        new_out += "%";
        file_writer.write(new_out.as_bytes()).unwrap();
        bytes_file += new_out.len();
    }

    file_writer.flush().unwrap();

    println!("Amount of lines {}", bytes);
    

    if debug {
        println!("{}", bytes);
        let mut collisions: Vec<usize> = vec![];
        println!("Getting collisions");

        for _i in 0..bytes{
            collisions.push(0);
        }

        for u in new_data{
            collisions[u.key] = collisions[u.key]+1;
        }


        let mut collision_counter = 0;

        for j in 0..collisions.len(){

            if !&collisions[j].eq(&1) && !&collisions[j].eq(&0){

                println!("Number is: {} at index {}", collisions[j], j);
                collision_counter += collisions[j];
            }
        }
        println!("Number of collisons: {}", collision_counter);
        println!("{}", bytes);
        
    }


}


fn create_index_file(debug: bool){

    let out_val = Path::new("index_file.txt");

    if debug{
        println!("Reading input...");
    }


    let fname = std::path::Path::new("token.zip");
    let file = fs::File::open(&fname).unwrap();
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let file = archive.by_index(0).unwrap();
    let mut in_lines = io::BufReader::new(file);


    let lines:Vec<String> = in_lines.lines().map(|s| s.unwrap().to_string().replace("\r", "")).collect::<Vec<String>>();

    
    if debug{
        for i in &lines{
            println!("File contains: {}", i);
        }
    }

    let mut new_word: String = "".to_string();
    let mut current_word: &str = &lines[0].split_whitespace().collect::<Vec<&str>>()[0];
    let mut posistions: String = "".to_string();

    for line in &lines{
        if line.len() < 1{
            break;
        }
        let words: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();

        if !current_word.eq(words[0]){

            new_word += current_word;
            new_word += &posistions;
            new_word += "\n";


            current_word = words[0];
            posistions = "".to_string();

        }
        posistions += &(" ".to_owned() + words[1]);
    }

    new_word += current_word;
    new_word += &posistions;




    if debug{
        println!("New data is: \n{}", new_word);
    }

    write_file(out_val, new_word);

}

pub fn write_file(path: &Path, new_word: String){

    fs::write(path, new_word).expect("Unable to write file");

}

