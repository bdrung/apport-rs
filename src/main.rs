use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::ExitCode;

use flate2::bufread::GzDecoder;
use lexopt::prelude::*;
use regex::Regex;
use rusqlite::{Connection, Transaction};

struct KeyID {
    data: HashMap<String, i64>,
    max_id: i64,
}

impl KeyID {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            max_id: 0,
        }
    }

    fn get(&self, key: &String) -> Option<i64> {
        self.data.get(key).copied()
    }

    fn add(&mut self, key: String) -> i64 {
        self.max_id += 1;
        self.data.insert(key, self.max_id);
        self.max_id
    }
}

trait Path2Package {
    fn open<P: AsRef<Path>>(path: P) -> Self;
    fn create_db(&self);
    fn update_from_contents_file(&mut self, content_path: &str, distro: &str, log_level: u32);
}

struct Path2PackageV2 {
    connection: Connection,
    package_id_cache: KeyID,
}

impl Path2Package for Path2PackageV2 {
    fn open<P: AsRef<Path>>(path: P) -> Self {
        Self {
            connection: Connection::open(path).unwrap(),
            package_id_cache: KeyID::new(),
        }
    }

    fn create_db(&self) {
        let query = "\
            CREATE TABLE packages (
            id integer PRIMARY KEY NOT NULL,
            package text NOT NULL UNIQUE
            );
            CREATE TABLE path_package (
            path TEXT PRIMARY KEY NOT NULL,
            package_id integer NOT NULL,
            FOREIGN KEY (package_id)
                REFERENCES packages(id)
            );
        ";
        self.connection.execute_batch(query).unwrap();
    }

    fn update_from_contents_file(&mut self, content_path: &str, distro: &str, log_level: u32) {
        let transaction = self.connection.transaction().unwrap();
        for pocket in ["-proposed", "", "-security", "-updates"] {
            read_contents_file_v2(
                &transaction,
                &mut self.package_id_cache,
                format!("{}/{}{}-Contents-amd64.gz", content_path, distro, pocket),
                log_level,
            )
            .unwrap();
        }
        transaction.commit().unwrap();
    }
}

fn read_contents_file_v2<P: AsRef<Path>>(
    transaction: &Transaction,
    package_id_cache: &mut KeyID,
    filename: P,
    log_level: u32,
) -> std::io::Result<()> {
    let query = "INSERT INTO packages VALUES (?, ?)";
    let mut package_statement = transaction.prepare(query).unwrap();
    let query = "INSERT INTO path_package VALUES (?, ?) ON CONFLICT(path) DO UPDATE SET package_id=excluded.package_id";
    let mut path_package_statement = transaction.prepare(query).unwrap();

    let file = File::open(filename)?;
    let br = std::io::BufReader::new(file);
    let reader = std::io::BufReader::new(GzDecoder::new(br));
    let path_exclude_re = Regex::new(r"^:|(boot|var|usr/(include|src|[^/]+/include|share/(doc|gocode|help|icons|locale|man|texlive)))/").unwrap();
    let mut lines = 0;
    let mut processed = 0;
    for line in reader.lines() {
        lines += 1;
        let line = line?;
        if path_exclude_re.is_match(&line) {
            continue;
        }
        if let Some((path, column2)) = line.rsplit_once(|c| c == '\t' || c == ' ') {
            let section_package1 = match column2.split_once(',') {
                Some((first, _)) => first,
                None => column2,
            };
            let package: String = match section_package1.rsplit_once('/') {
                Some((_, second)) => second.into(),
                None => section_package1.into(),
            };
            let package_id = match package_id_cache.get(&package) {
                Some(id) => id,
                None => {
                    let package_id = package_id_cache.add(package.clone());
                    if log_level >= LOG_LEVEL_DEBUG {
                        println!(
                            "INSERT INTO packages VALUES ({}, '{}')",
                            package_id, package
                        );
                    }
                    package_statement.execute((package_id, package)).unwrap();
                    package_id
                }
            };
            path_package_statement
                .execute((path.trim_end(), package_id))
                .unwrap();
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Malformed line: '{}'", line),
            ));
        }
        processed += 1;
    }
    if log_level >= LOG_LEVEL_INFO {
        println!(
            "Added paths to database: {}/{} ({:.1} %)",
            processed,
            lines,
            (100 * processed) as f64 / lines as f64
        );
    }
    Ok(())
}

struct Path2PackageV3 {
    connection: Connection,
    package_id_cache: KeyID,
    directory_id_cache: KeyID,
}

impl Path2Package for Path2PackageV3 {
    fn open<P: AsRef<Path>>(path: P) -> Self {
        Self {
            connection: Connection::open(path).unwrap(),
            package_id_cache: KeyID::new(),
            directory_id_cache: KeyID::new(),
        }
    }

    fn create_db(&self) {
        let query = "\
        CREATE TABLE packages (
          id integer PRIMARY KEY NOT NULL,
          package text NOT NULL UNIQUE
        );
        CREATE TABLE directories (
          id integer PRIMARY KEY NOT NULL,
          directory text NOT NULL UNIQUE
        );
        CREATE TABLE directory_name_package (
          directory_id integer NOT NULL,
          name string NOT NULL,
          package_id integer NOT NULL,
          PRIMARY KEY (directory_id, name),
          FOREIGN KEY (directory_id) REFERENCES directories(id)
          FOREIGN KEY (package_id) REFERENCES packages(id)
        );
        ";
        self.connection.execute_batch(query).unwrap();
    }

    fn update_from_contents_file(&mut self, contents_path: &str, distro: &str, log_level: u32) {
        let transaction = self.connection.transaction().unwrap();
        for pocket in ["-proposed", "", "-security", "-updates"] {
            read_contents_file_v3(
                &transaction,
                &mut self.package_id_cache,
                &mut self.directory_id_cache,
                format!("{}/{}{}-Contents-amd64.gz", contents_path, distro, pocket),
                log_level,
            )
            .unwrap();
        }
        transaction.commit().unwrap();
    }
}

fn read_contents_file_v3<P: AsRef<Path>>(
    transaction: &Transaction,
    package_id_cache: &mut KeyID,
    directory_id_cache: &mut KeyID,
    filename: P,
    log_level: u32,
) -> std::io::Result<()> {
    let query = "INSERT INTO packages VALUES (?, ?)";
    let mut package_statement = transaction.prepare(query).unwrap();
    let query = "INSERT INTO directories VALUES(?, ?)";
    let mut directories_statement = transaction.prepare(query).unwrap();
    let query = "INSERT INTO directory_name_package VALUES (?, ?, ?) ON CONFLICT(directory_id, name) DO UPDATE SET package_id=excluded.package_id";
    let mut directory_name_package_statement = transaction.prepare(query).unwrap();

    let file = File::open(filename)?;
    let br = std::io::BufReader::new(file);
    let reader = std::io::BufReader::new(GzDecoder::new(br));
    let path_exclude_re = Regex::new(r"^:|(boot|var|usr/(include|src|[^/]+/include|share/(doc|gocode|help|icons|locale|man|texlive)))/").unwrap();
    let mut lines = 0;
    let mut processed = 0;
    for line in reader.lines() {
        lines += 1;
        let line = line?;
        if path_exclude_re.is_match(&line) {
            continue;
        }
        if let Some((path, column2)) = line.rsplit_once(|c| c == '\t' || c == ' ') {
            let section_package1 = match column2.split_once(',') {
                Some((first, _)) => first,
                None => column2,
            };
            let package: String = match section_package1.rsplit_once('/') {
                Some((_, second)) => second.into(),
                None => section_package1.into(),
            };
            let path = path.trim_end();

            let mut matches = path.match_indices('/');
            let mut index: usize = 0;
            for _ in 1..6 {
                match matches.next() {
                    Some((i, _)) => index = i,
                    None => break,
                }
            }
            let (directory, _) = path.split_at(index);
            let (_, name) = path.split_at(index + 1);
            let directory: String = directory.into();

            let directory_id = match directory_id_cache.get(&directory) {
                Some(id) => id,
                None => {
                    let directory_id = directory_id_cache.add(directory.clone());
                    if log_level >= LOG_LEVEL_DEBUG {
                        println!(
                            "INSERT INTO directories VALUES ({}, '{}')",
                            directory_id, directory
                        );
                    }
                    directories_statement
                        .execute((directory_id, directory))
                        .unwrap();
                    directory_id
                }
            };
            let package_id = match package_id_cache.get(&package) {
                Some(id) => id,
                None => {
                    let package_id = package_id_cache.add(package.clone());
                    if log_level >= LOG_LEVEL_DEBUG {
                        println!(
                            "INSERT INTO packages VALUES ({}, '{}')",
                            package_id, package
                        );
                    }
                    package_statement.execute((package_id, package)).unwrap();
                    package_id
                }
            };
            if log_level >= LOG_LEVEL_DEBUG {
                println!(
                    "INSERT INTO directory_name_package VALUES ({}, '{}', {})",
                    directory_id, name, package_id
                );
            }
            directory_name_package_statement
                .execute((directory_id, name, package_id))
                .unwrap();
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Malformed line: '{}'", line),
            ));
        }
        processed += 1;
    }
    if log_level >= LOG_LEVEL_INFO {
        println!(
            "Added paths to database: {}/{} ({:.1} %)",
            processed,
            lines,
            (100 * processed) as f64 / lines as f64
        );
    }
    Ok(())
}

fn create_db_v1(connection: &Connection) {
    let query = "\
    CREATE TABLE path_package(
        path TEXT PRIMARY KEY UNIQUE NOT NULL,
        package TEXT NOT NULL);
";
    connection.execute(query, ()).unwrap();
}

fn read_contents_file_v1<P: AsRef<Path>>(
    transaction: &Transaction,
    filename: P,
    log_level: u32,
) -> std::io::Result<()> {
    let query = "INSERT INTO path_package VALUES (?, ?) ON CONFLICT(path) DO UPDATE SET package=excluded.package;";
    let mut statement = transaction.prepare(query).unwrap();

    let file = File::open(filename)?;
    let br = std::io::BufReader::new(file);
    let reader = std::io::BufReader::new(GzDecoder::new(br));
    let path_exclude_re = Regex::new(r"^:|(boot|var|usr/(include|src|[^/]+/include|share/(doc|gocode|help|icons|locale|man|texlive)))/").unwrap();
    let mut lines = 0;
    let mut processed = 0;
    for line in reader.lines() {
        lines += 1;
        let line = line?;
        if path_exclude_re.is_match(&line) {
            continue;
        }
        if let Some((path, column2)) = line.rsplit_once(|c| c == '\t' || c == ' ') {
            let section_package1 = match column2.split_once(',') {
                Some((first, _)) => first,
                None => column2,
            };
            let package = match section_package1.rsplit_once('/') {
                Some((_, second)) => second,
                None => section_package1,
            };
            if log_level >= LOG_LEVEL_DEBUG {
                println!(
                    "INSERT INTO path_package VALUES ('{}', '{}');",
                    path.trim_end(),
                    package
                );
            }
            statement.execute((path.trim_end(), package)).unwrap();
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Malformed line: '{}'", line),
            ));
        }
        processed += 1;
    }
    if log_level >= LOG_LEVEL_INFO {
        println!(
            "Added paths to database: {}/{} ({:.1} %)",
            processed,
            lines,
            (100 * processed) as f64 / lines as f64
        );
    }
    Ok(())
}

fn main_v1<P: AsRef<Path>>(db_name: P, content_path: &str, distro: &str, log_level: u32) {
    let mut connection = Connection::open(db_name).unwrap();
    create_db_v1(&connection);
    let transaction = connection.transaction().unwrap();
    for pocket in ["-proposed", "", "-security", "-updates"] {
        read_contents_file_v1(
            &transaction,
            format!("{}/{}{}-Contents-amd64.gz", content_path, distro, pocket),
            log_level,
        )
        .unwrap();
    }
    transaction.commit().unwrap();
}

pub const LOG_LEVEL_WARNING: u32 = 5;
pub const LOG_LEVEL_INFO: u32 = 7;
pub const LOG_LEVEL_DEBUG: u32 = 8;

#[derive(Debug)]
struct Args {
    cache_dir: String,
    log_level: u32,
    release: String,
    version: i32,
}

fn print_help() {
    let executable = std::env::args().next().unwrap();
    println!(
        "Usage:
    {executable} [-v|--debug] [-c DIR] [-r DISTRO] [-j|--jammy]

Optional arguments:
  -c, --cache=DIR  Cache directory that contains the Contents files.
  -r, --release=R  Release (e.g. noble or jammy)
  -j, --jammy      Short for --release=jammy
  -V, --version=V  SQLite database implementation version/variant (1-3)
  -v, --verbose    Verbose output
  --debug          Debug output
  -h, --help       print help message",
    );
}

fn parse_args() -> Result<Args, lexopt::Error> {
    let mut cache_dir = "contents_cache".into();
    let mut log_level = LOG_LEVEL_WARNING;
    let mut version = 1;
    let mut release = "noble".into();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('c') | Long("cache") => {
                cache_dir = parser.value()?.string()?;
            }
            Long("debug") => {
                log_level = LOG_LEVEL_DEBUG;
            }
            Short('h') | Long("help") => {
                print_help();
                std::process::exit(0);
            }
            Short('j') | Long("jammy") => {
                release = "jammy".into();
            }
            Short('r') | Long("release") => {
                release = parser.value()?.string()?;
            }
            Short('v') | Long("verbose") => {
                if log_level <= LOG_LEVEL_INFO {
                    log_level = LOG_LEVEL_INFO;
                }
            }
            Short('V') | Long("version") => {
                version = parser.value()?.parse::<i32>()?;
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        cache_dir,
        log_level,
        release,
        version,
    })
}

fn main() -> ExitCode {
    let executable = std::env::args().next().unwrap();
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}: Error: {}", executable, e);
            return ExitCode::from(2);
        }
    };
    let db_name = format!("contents-{}_v{}.sqlite3", args.release, args.version);
    // TODO
    let _ = std::fs::remove_file(&db_name);
    if args.log_level >= LOG_LEVEL_INFO {
        println!("Creating {}...", db_name);
    }
    if args.version == 1 {
        main_v1(db_name, &args.cache_dir, &args.release, args.log_level);
    } else if args.version == 2 {
        let mut path2package = Path2PackageV2::open(&db_name);
        path2package.create_db();
        path2package.update_from_contents_file(&args.cache_dir, &args.release, args.log_level);
        if args.log_level >= LOG_LEVEL_INFO {
            println!(
                "Entries in packages table: {}",
                path2package.package_id_cache.max_id
            );
        }
    } else if args.version == 3 {
        let mut path2package = Path2PackageV3::open(&db_name);
        path2package.create_db();
        path2package.update_from_contents_file(&args.cache_dir, &args.release, args.log_level);
        if args.log_level >= LOG_LEVEL_INFO {
            println!(
                "Entries in packages table: {}\nEntries in directories table: {}",
                path2package.package_id_cache.max_id, path2package.directory_id_cache.max_id
            );
        }
    }
    ExitCode::SUCCESS
}
