#![allow(dead_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::wildcard_enum_match_arm)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        deprecated
    )
)]

use std::fs;
use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};

mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}
use demo::{AnObject, Gender, TestUnion, User};

#[derive(Parser)]
#[command(name = "demo")]
#[command(about = "Fluorite demo with interop testing support")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Write sample data to JSON files
    Write {
        /// Output directory for JSON files
        #[arg(short, long, default_value = "./fixtures")]
        output: String,
    },
    /// Read and validate JSON files
    Read {
        /// Input directory containing JSON files
        #[arg(short, long, default_value = "./fixtures")]
        input: String,
    },
}

fn create_sample_user_male() -> User {
    let first_name = "John".to_string();
    let last_name = "Doe".to_string();
    let gender = Gender::Male;
    let age = 30;
    let active = true;
    let info = Some(serde_json::json!({
        "hobbies": ["reading", "coding"],
        "score": 95.5
    }));

    User {
        first_name,
        last_name,
        gender,
        age,
        active,
        info,
    }
}

fn create_sample_user_female() -> User {
    let first_name = "Jane".to_string();
    let last_name = "Smith".to_string();
    let gender = Gender::Female;
    let age = 25;
    let active = false;
    let info = None;

    User {
        first_name,
        last_name,
        gender,
        age,
        active,
        info,
    }
}

fn create_sample_plain_string() -> TestUnion {
    TestUnion::PlainString
}

fn create_sample_an_object() -> TestUnion {
    TestUnion::AnObject(AnObject::new("Test field value".to_owned()))
}

fn write_sample_data(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    // Write User samples
    let user_male = create_sample_user_male();
    let user_female = create_sample_user_female();

    let user_male_json = serde_json::to_string_pretty(&user_male)?;
    let user_female_json = serde_json::to_string_pretty(&user_female)?;

    let mut file = fs::File::create(path.join("user_male.json"))?;
    file.write_all(user_male_json.as_bytes())?;

    let mut file = fs::File::create(path.join("user_female.json"))?;
    file.write_all(user_female_json.as_bytes())?;

    // Write TestUnion samples
    let plain_string = create_sample_plain_string();
    let an_object = create_sample_an_object();

    let plain_string_json = serde_json::to_string_pretty(&plain_string)?;
    let an_object_json = serde_json::to_string_pretty(&an_object)?;

    let mut file = fs::File::create(path.join("union_plain_string.json"))?;
    file.write_all(plain_string_json.as_bytes())?;

    let mut file = fs::File::create(path.join("union_an_object.json"))?;
    file.write_all(an_object_json.as_bytes())?;

    println!("Sample data written to {}", output_dir);
    Ok(())
}

fn read_and_validate(input_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(input_dir);

    let files = [
        "user_male.json",
        "user_female.json",
        "union_plain_string.json",
        "union_an_object.json",
    ];

    for filename in files.iter() {
        let file_path = path.join(filename);
        if !file_path.exists() {
            eprintln!("File not found: {}", file_path.display());
            continue;
        }

        let content = fs::read_to_string(&file_path)?;

        if filename.starts_with("user") {
            let user: User = serde_json::from_str(&content)?;
            println!("Validated User: {} {}", user.first_name, user.last_name);
            println!("  Gender: {:?}, Active: {}", user.gender, user.active);
            if let Some(ref info) = user.info {
                println!("  Info: {:?}", info);
            }
        } else if filename.starts_with("union") {
            let union: TestUnion = serde_json::from_str(&content)?;
            match union {
                TestUnion::PlainString => {
                    println!("Validated TestUnion::PlainString");
                }
                TestUnion::AnObject(ref obj) => {
                    println!("Validated TestUnion::AnObject");
                    println!("  Field A: {}", obj.field_a);
                }
            }
        }
    }

    Ok(())
}

fn run_default_demo() {
    let first_name = "f".to_string();
    let last_name = "l".to_string();
    let gender = Gender::Male;
    let age = 10;
    let active = true;
    let info = None;

    let user = User {
        first_name,
        last_name,
        gender,
        age,
        active,
        info,
    };
    println!("user: {:?}", user);

    let o = TestUnion::AnObject(AnObject::new("test".to_owned()));
    println!("object enum: {:?}", o);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Write { output }) => {
            if let Err(e) = write_sample_data(&output) {
                eprintln!("Error writing sample data: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Read { input }) => {
            if let Err(e) = read_and_validate(&input) {
                eprintln!("Error reading sample data: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            run_default_demo();
        }
    }
}
